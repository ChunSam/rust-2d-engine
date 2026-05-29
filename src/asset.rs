#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc::channel;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc::Receiver;
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};

pub type AssetId = u64;

static NEXT_ASSET_ID: AtomicU64 = AtomicU64::new(1);

fn alloc_id() -> AssetId {
    NEXT_ASSET_ID.fetch_add(1, Ordering::Relaxed)
}

// ─── Handle<T> ────────────────────────────────────────────────────────────────

/// Typed, lightweight reference to a loaded asset.
///
/// Clone is O(1) (Arc pointer copy). Stores the canonical path so the renderer
/// can resolve the GPU texture without an extra AssetServer lookup.
pub struct Handle<T> {
    pub(crate) id: AssetId,
    pub(crate) path: Arc<str>,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Handle<T> {
    pub fn id(&self) -> AssetId {
        self.id
    }

    /// 이 핸들이 가리키는 파일 경로.
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            path: Arc::clone(&self.path),
            _marker: PhantomData,
        }
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for Handle<T> {}

impl<T> std::hash::Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Handle({}, {:?})", self.id, &*self.path)
    }
}

// ─── ImageAsset ───────────────────────────────────────────────────────────────

/// CPU-side decoded image (RGBA8). Cheap to clone (data behind Arc).
#[derive(Clone)]
pub struct ImageAsset {
    pub data: Arc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

/// 에셋 브라우저에서 표시할 이미지 항목 정보.
#[derive(Debug, Clone)]
pub struct ImageEntry {
    pub path: String,
    pub id: AssetId,
    pub width: u32,
    pub height: u32,
}

// ─── ScriptAsset ─────────────────────────────────────────────────────────────

/// CPU-side Rhai 스크립트 에셋.
pub struct ScriptAsset {
    pub source: String,
    pub ast: rhai::AST,
}

// ─── AssetLoadState ───────────────────────────────────────────────────────────

/// 에셋 로드 결과 상태. `AssetServer::load_state()`로 조회한다.
#[derive(Debug, Clone, PartialEq)]
pub enum AssetLoadState {
    /// 비동기 로드 진행 중. 마젠타 폴백 텍스처가 표시된다.
    Loading,
    /// 정상 로드됨.
    Loaded,
    /// 로드 실패 (파일 없음, 디코딩 오류 등). 마젠타 폴백 텍스처로 대체된 상태.
    Failed(String),
}

// ─── 비동기 로드 결과 (네이티브 채널용) ───────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
struct AsyncImageResult {
    id: AssetId,
    path: String,
    asset: ImageAsset,
    state: AssetLoadState,
}

// ─── WASM 비동기 큐 ───────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WASM_ASYNC_QUEUE: RefCell<std::collections::VecDeque<(AssetId, String, (ImageAsset, AssetLoadState))>>
        = RefCell::new(std::collections::VecDeque::new());
}

// ─── AssetServer ──────────────────────────────────────────────────────────────

/// 에셋 관리자 — 이미지 로드·캐싱·핫 리로딩.
///
/// ECS World에 Resource로 삽입해 사용하거나 `App::load_image`를 통해 간접적으로 접근한다.
/// 네이티브 빌드에서는 존재하는 파일 경로를 canonical path로 정규화해 캐시 키로 사용한다.
/// 존재하지 않는 경로는 입력 문자열을 그대로 보존해 기존 fallback 동작을 유지하고, WASM에서는
/// URL/상대경로 의미를 보존하기 위해 정규화하지 않는다.
///
/// # 핫 리로딩
/// 파일이 변경되면 `poll_reloads()`가 변경된 경로 목록을 반환한다.
/// `App`이 매 프레임 이를 호출해 GPU 텍스처를 재업로드한다.
///
/// # 예시
/// ```rust,no_run
/// # use engine::App;
/// let mut app = App::new();
/// let handle = app.load_image("assets/player.png");
/// // 로드 실패 여부 확인
/// # use engine::asset::AssetLoadState;
/// # let assets = app.world.resource::<engine::AssetServer>().unwrap();
/// // if assets.load_state(&handle) == AssetLoadState::Failed { ... }
/// ```
pub struct AssetServer {
    images: HashMap<AssetId, ImageAsset>,
    image_load_states: HashMap<AssetId, AssetLoadState>,
    path_to_id: HashMap<Arc<str>, AssetId>,
    scripts: HashMap<AssetId, ScriptAsset>,
    script_path_to_id: HashMap<Arc<str>, AssetId>,
    atlases: HashMap<AssetId, crate::atlas::TextureAtlas>,
    atlas_path_to_id: HashMap<Arc<str>, AssetId>,
    #[cfg(not(target_arch = "wasm32"))]
    reload_rx: Option<Receiver<PathBuf>>,
    #[cfg(not(target_arch = "wasm32"))]
    _watcher: Option<RecommendedWatcher>,
    // 비동기 로드용 채널 (네이티브 전용)
    #[cfg(not(target_arch = "wasm32"))]
    async_tx: std::sync::mpsc::SyncSender<AsyncImageResult>,
    #[cfg(not(target_arch = "wasm32"))]
    async_rx: std::sync::mpsc::Receiver<AsyncImageResult>,
}

pub(crate) fn asset_key(path: impl AsRef<Path>) -> Arc<str> {
    let path = path.as_ref();
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(canonical) = path.canonicalize() {
            return canonical.to_string_lossy().as_ref().into();
        }
    }
    path.to_string_lossy().as_ref().into()
}

impl AssetServer {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (async_tx, async_rx) = std::sync::mpsc::sync_channel::<AsyncImageResult>(128);
            let (tx, rx) = channel::<PathBuf>();
            let watcher_result =
                notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                    if let Ok(event) = res {
                        match event.kind {
                            EventKind::Modify(_) | EventKind::Create(_) => {
                                for path in event.paths {
                                    let _ = tx.send(path);
                                }
                            }
                            _ => {}
                        }
                    }
                });
            match watcher_result {
                Ok(w) => Self {
                    images: HashMap::new(),
                    image_load_states: HashMap::new(),
                    path_to_id: HashMap::new(),
                    scripts: HashMap::new(),
                    script_path_to_id: HashMap::new(),
                    atlases: HashMap::new(),
                    atlas_path_to_id: HashMap::new(),
                    reload_rx: Some(rx),
                    _watcher: Some(w),
                    async_tx,
                    async_rx,
                },
                Err(e) => {
                    log::warn!("파일 감시 초기화 실패 (핫 리로딩 비활성): {e}");
                    let (async_tx2, async_rx2) =
                        std::sync::mpsc::sync_channel::<AsyncImageResult>(128);
                    Self {
                        images: HashMap::new(),
                        image_load_states: HashMap::new(),
                        path_to_id: HashMap::new(),
                        scripts: HashMap::new(),
                        script_path_to_id: HashMap::new(),
                        atlases: HashMap::new(),
                        atlas_path_to_id: HashMap::new(),
                        reload_rx: None,
                        _watcher: None,
                        async_tx: async_tx2,
                        async_rx: async_rx2,
                    }
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        Self {
            images: HashMap::new(),
            image_load_states: HashMap::new(),
            path_to_id: HashMap::new(),
            scripts: HashMap::new(),
            script_path_to_id: HashMap::new(),
            atlases: HashMap::new(),
            atlas_path_to_id: HashMap::new(),
        }
    }

    /// 이미지를 로드해 핸들을 반환한다. 같은 경로를 다시 호출하면 캐시된 핸들을 반환한다.
    ///
    /// 로드 실패 시 마젠타(1×1) 폴백 텍스처로 대체되며 `load_state()`로 결과를 확인할 수 있다.
    pub fn load_image(&mut self, path: impl AsRef<Path>) -> Handle<ImageAsset> {
        let key = asset_key(path.as_ref());
        if let Some(&id) = self.path_to_id.get(&key) {
            return Handle {
                id,
                path: key,
                _marker: PhantomData,
            };
        }
        let id = alloc_id();
        self.path_to_id.insert(Arc::clone(&key), id);

        #[cfg(not(target_arch = "wasm32"))]
        {
            let (asset, state) = decode_image_with_state(&key);
            self.images.insert(id, asset);
            self.image_load_states.insert(id, state);
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.images.insert(id, magenta_fallback());
            self.image_load_states.insert(id, AssetLoadState::Loading);
            spawn_image_fetch_wasm(id, key.to_string());
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(ref mut w) = self._watcher {
            let _ = w.watch(path.as_ref(), RecursiveMode::NonRecursive);
        }
        Handle {
            id,
            path: key,
            _marker: PhantomData,
        }
    }

    /// 핸들의 로드 상태를 반환한다.
    ///
    /// 존재하지 않는 핸들이면 `AssetLoadState::Failed`를 반환한다.
    pub fn load_state(&self, handle: &Handle<ImageAsset>) -> AssetLoadState {
        self.image_load_states
            .get(&handle.id)
            .cloned()
            .unwrap_or_else(|| AssetLoadState::Failed("unknown handle".into()))
    }

    /// 로드에 실패한 이미지 핸들 목록을 반환한다 (디버그용).
    pub fn failed_images(&self) -> Vec<AssetId> {
        self.image_load_states
            .iter()
            .filter_map(|(&id, state)| matches!(state, AssetLoadState::Failed(_)).then_some(id))
            .collect()
    }

    /// 현재 캐시된 이미지 에셋 수를 반환한다.
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    /// CPU-side 이미지 데이터를 반환한다.
    pub fn get_image(&self, handle: &Handle<ImageAsset>) -> Option<&ImageAsset> {
        self.images.get(&handle.id)
    }

    /// id로 이미지 에셋을 직접 조회한다 (에셋 브라우저용).
    pub fn get_image_by_id(&self, id: AssetId) -> Option<&ImageAsset> {
        self.images.get(&id)
    }

    /// 현재 로드된 이미지 에셋 목록을 반환한다 (에셋 브라우저용).
    pub fn image_list(&self) -> Vec<ImageEntry> {
        self.path_to_id
            .iter()
            .filter_map(|(path, &id)| {
                self.images.get(&id).map(|img| ImageEntry {
                    path: path.to_string(),
                    id,
                    width: img.width,
                    height: img.height,
                })
            })
            .collect()
    }

    /// 스크립트를 로드해 핸들을 반환한다. 같은 경로 재호출 시 캐시된 핸들 반환.
    pub fn load_script(&mut self, path: impl AsRef<Path>) -> Handle<ScriptAsset> {
        let key = asset_key(path.as_ref());
        if let Some(&id) = self.script_path_to_id.get(&key) {
            return Handle {
                id,
                path: key,
                _marker: PhantomData,
            };
        }
        let id = alloc_id();
        let asset = compile_script_file(&key);
        self.scripts.insert(id, asset);
        self.script_path_to_id.insert(Arc::clone(&key), id);
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(ref mut w) = self._watcher {
            let _ = w.watch(path.as_ref(), RecursiveMode::NonRecursive);
        }
        Handle {
            id,
            path: key,
            _marker: PhantomData,
        }
    }

    /// 스크립트 에셋을 id로 조회한다 (ScriptingSystem 내부용).
    pub fn get_script_by_id(&self, id: AssetId) -> Option<&ScriptAsset> {
        self.scripts.get(&id)
    }

    /// 텍스처 아틀라스를 로드하고 핸들을 반환한다. 같은 경로를 다시 호출하면 캐시된 핸들을 반환한다.
    ///
    /// 내부적으로 이미지(`Handle<ImageAsset>`)도 함께 로드한다.
    pub fn load_atlas(
        &mut self,
        path: impl AsRef<Path>,
        cols: u32,
        rows: u32,
    ) -> Handle<crate::atlas::TextureAtlas> {
        let key = asset_key(path.as_ref());
        if let Some(&id) = self.atlas_path_to_id.get(&key) {
            return Handle {
                id,
                path: key,
                _marker: PhantomData,
            };
        }
        let img_handle = self.load_image(path.as_ref());
        let id = alloc_id();
        let atlas = crate::atlas::TextureAtlas {
            handle: img_handle,
            cols,
            rows,
        };
        self.atlases.insert(id, atlas);
        self.atlas_path_to_id.insert(Arc::clone(&key), id);
        Handle {
            id,
            path: key,
            _marker: PhantomData,
        }
    }

    /// 아틀라스 핸들로 `TextureAtlas` 데이터를 조회한다.
    pub fn get_atlas(
        &self,
        handle: &Handle<crate::atlas::TextureAtlas>,
    ) -> Option<&crate::atlas::TextureAtlas> {
        self.atlases.get(&handle.id)
    }

    /// 변경된 파일 경로 목록을 반환하고 내부 CPU 캐시를 갱신한다.
    ///
    /// `App`이 매 프레임 이를 호출하고, 반환된 경로들에 대해 GPU 텍스처를 재업로드한다.
    pub fn poll_reloads(&mut self) -> Vec<String> {
        #[cfg(target_arch = "wasm32")]
        {
            Vec::new()
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let rx = match &self.reload_rx {
                Some(r) => r,
                None => return Vec::new(),
            };
            let mut seen: Vec<String> = Vec::new();
            while let Ok(path) = rx.try_recv() {
                let key = asset_key(&path);
                let key_str = key.to_string();
                let is_known =
                    self.path_to_id.contains_key(&key) || self.script_path_to_id.contains_key(&key);
                if is_known && !seen.contains(&key_str) {
                    seen.push(key_str);
                }
            }
            for path_str in &seen {
                let key: Arc<str> = path_str.as_str().into();
                if let Some(&id) = self.path_to_id.get(&key) {
                    let (asset, state) = decode_image_with_state(path_str);
                    self.images.insert(id, asset);
                    self.image_load_states.insert(id, state);
                }
                if let Some(&id) = self.script_path_to_id.get(&key) {
                    self.scripts.insert(id, compile_script_file(path_str));
                }
            }
            seen
        }
    }

    /// 이미지를 백그라운드에서 비동기로 로드한다.
    ///
    /// 즉시 마젠타 폴백 텍스처가 등록된 핸들을 반환한다. 로딩 완료 전까지
    /// `load_state()`는 `AssetLoadState::Loading`을 반환한다.
    ///
    /// - 네이티브: `std::thread::spawn` 백그라운드 스레드에서 로드
    /// - WASM: `wasm_bindgen_futures::spawn_local` + `fetch` API
    pub fn load_image_async(&mut self, path: impl AsRef<Path>) -> Handle<ImageAsset> {
        let key = asset_key(path.as_ref());
        // 캐시 확인 — 이미 로드됐거나 로딩 중이면 기존 핸들 반환
        if let Some(&id) = self.path_to_id.get(&key) {
            return Handle {
                id,
                path: key,
                _marker: PhantomData,
            };
        }
        let id = alloc_id();
        // 마젠타 폴백 + Loading 상태로 즉시 등록
        self.images.insert(id, magenta_fallback());
        self.image_load_states.insert(id, AssetLoadState::Loading);
        self.path_to_id.insert(Arc::clone(&key), id);

        #[cfg(not(target_arch = "wasm32"))]
        {
            let tx = self.async_tx.clone();
            let path_str = key.to_string();
            std::thread::spawn(move || {
                let (asset, state) = decode_image_with_state(&path_str);
                let _ = tx.send(AsyncImageResult {
                    id,
                    path: path_str,
                    asset,
                    state,
                });
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            spawn_image_fetch_wasm(id, key.to_string());
        }

        Handle {
            id,
            path: key,
            _marker: PhantomData,
        }
    }

    /// 완료된 비동기 로드 결과를 처리해 내부 캐시를 갱신한다.
    ///
    /// `App`이 매 프레임 호출한다. 완료된 에셋의 경로 목록을 반환한다.
    pub(crate) fn poll_async_completions(&mut self) -> Vec<(String, ImageAsset)> {
        let mut completed = Vec::new();

        #[cfg(not(target_arch = "wasm32"))]
        while let Ok(result) = self.async_rx.try_recv() {
            let asset = result.asset;
            self.images.insert(result.id, asset.clone());
            self.image_load_states.insert(result.id, result.state);
            completed.push((result.path, asset));
        }

        #[cfg(target_arch = "wasm32")]
        WASM_ASYNC_QUEUE.with(|q| {
            while let Some((id, path, (asset, state))) = q.borrow_mut().pop_front() {
                self.images.insert(id, asset.clone());
                self.image_load_states.insert(id, state);
                completed.push((path, asset));
            }
        });

        completed
    }

    /// 현재 `AssetLoadState::Loading` 상태인 이미지 수를 반환한다.
    pub fn async_loading_count(&self) -> usize {
        self.image_load_states
            .values()
            .filter(|s| matches!(s, AssetLoadState::Loading))
            .count()
    }
}

impl Default for AssetServer {
    fn default() -> Self {
        Self::new()
    }
}

// ─── 내부 헬퍼 ────────────────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
fn decode_image_with_state(path: &str) -> (ImageAsset, AssetLoadState) {
    match std::fs::read(path) {
        Ok(bytes) => match image::load_from_memory(&bytes) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                (
                    ImageAsset {
                        data: Arc::new(rgba.into_raw()),
                        width: w,
                        height: h,
                    },
                    AssetLoadState::Loaded,
                )
            }
            Err(e) => {
                let msg = format!("이미지 디코딩 실패 '{path}': {e}");
                log::error!("{msg}");
                (magenta_fallback(), AssetLoadState::Failed(msg))
            }
        },
        Err(e) => {
            let msg = format!("이미지 파일 읽기 실패 '{path}': {e}");
            log::error!("{msg}");
            (magenta_fallback(), AssetLoadState::Failed(msg))
        }
    }
}

fn compile_script_file(path: &str) -> ScriptAsset {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            log::error!("스크립트 파일 읽기 실패 '{path}': {e}");
            String::new()
        }
    };
    let engine = rhai::Engine::new();
    let ast = engine.compile(&source).unwrap_or_else(|e| {
        log::error!("스크립트 컴파일 실패 '{path}': {e}");
        engine.compile("").unwrap()
    });
    ScriptAsset { source, ast }
}

fn magenta_fallback() -> ImageAsset {
    ImageAsset {
        data: Arc::new(vec![255, 0, 255, 255]),
        width: 1,
        height: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_asset_key_preserves_input_path() {
        let key = asset_key("__definitely_missing_asset__.png");
        assert_eq!(&*key, "__definitely_missing_asset__.png");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn existing_native_paths_are_canonicalized_for_cache_keys() {
        let dir = std::env::current_dir()
            .unwrap()
            .join("target")
            .join(format!("asset-key-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("image.png");
        std::fs::write(&path, b"not a png").unwrap();

        let relative = path.strip_prefix(std::env::current_dir().unwrap()).unwrap();
        let absolute = path.canonicalize().unwrap();
        let mut server = AssetServer::new();

        let a = server.load_image(relative);
        let b = server.load_image(&absolute);

        assert_eq!(a.id(), b.id());
        assert_eq!(a.path(), absolute.to_string_lossy());

        std::fs::remove_file(&path).ok();
        std::fs::remove_dir_all(&dir).ok();
    }
}

// ─── WASM fetch 헬퍼 ─────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
async fn fetch_image_wasm(url: &str) -> (ImageAsset, AssetLoadState) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let window = match web_sys::window() {
        Some(w) => w,
        None => {
            let msg = format!("fetch 실패 '{url}': window 없음");
            log::error!("{msg}");
            return (magenta_fallback(), AssetLoadState::Failed(msg));
        }
    };

    let resp_value = match JsFuture::from(window.fetch_with_str(url)).await {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("fetch 실패 '{url}': {:?}", e);
            log::error!("{msg}");
            return (magenta_fallback(), AssetLoadState::Failed(msg));
        }
    };

    let resp: web_sys::Response = match resp_value.dyn_into() {
        Ok(r) => r,
        Err(_) => {
            let msg = format!("fetch 응답 변환 실패 '{url}'");
            return (magenta_fallback(), AssetLoadState::Failed(msg));
        }
    };

    if !resp.ok() {
        let msg = format!("fetch HTTP 오류 '{url}': {}", resp.status());
        log::error!("{msg}");
        return (magenta_fallback(), AssetLoadState::Failed(msg));
    }

    let array_buffer_promise = match resp.array_buffer() {
        Ok(p) => p,
        Err(e) => {
            let msg = format!("array_buffer() 실패 '{url}': {:?}", e);
            return (magenta_fallback(), AssetLoadState::Failed(msg));
        }
    };

    let array_buffer = match JsFuture::from(array_buffer_promise).await {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("응답 읽기 실패 '{url}': {:?}", e);
            return (magenta_fallback(), AssetLoadState::Failed(msg));
        }
    };

    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let bytes = uint8_array.to_vec();

    match image::load_from_memory(&bytes) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            (
                ImageAsset {
                    data: Arc::new(rgba.into_raw()),
                    width: w,
                    height: h,
                },
                AssetLoadState::Loaded,
            )
        }
        Err(e) => {
            let msg = format!("이미지 디코딩 실패 '{url}': {e}");
            log::error!("{msg}");
            (magenta_fallback(), AssetLoadState::Failed(msg))
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn spawn_image_fetch_wasm(id: AssetId, path: String) {
    wasm_bindgen_futures::spawn_local(async move {
        let result = fetch_image_wasm(&path).await;
        WASM_ASYNC_QUEUE.with(|q| q.borrow_mut().push_back((id, path, result)));
    });
}
