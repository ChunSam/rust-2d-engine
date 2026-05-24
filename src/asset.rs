use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc::channel;
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

// ─── ScriptAsset ─────────────────────────────────────────────────────────────

/// CPU-side Rhai 스크립트 에셋.
pub struct ScriptAsset {
    pub source: String,
    pub ast: rhai::AST,
}

// ─── AssetServer ──────────────────────────────────────────────────────────────

/// 에셋 관리자 — 이미지 로드·캐싱·핫 리로딩.
///
/// ECS World에 Resource로 삽입해 사용하거나 `App::load_image`를 통해 간접적으로 접근한다.
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
/// // Sprite::with_handle(handle) 로 사용
/// ```
pub struct AssetServer {
    images: HashMap<AssetId, ImageAsset>,
    path_to_id: HashMap<Arc<str>, AssetId>,
    scripts: HashMap<AssetId, ScriptAsset>,
    script_path_to_id: HashMap<Arc<str>, AssetId>,
    atlases: HashMap<AssetId, crate::atlas::TextureAtlas>,
    atlas_path_to_id: HashMap<Arc<str>, AssetId>,
    reload_rx: Option<Receiver<PathBuf>>,
    #[cfg(not(target_arch = "wasm32"))]
    _watcher: Option<RecommendedWatcher>,
}

impl AssetServer {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
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
                Ok(w) => {
                    return Self {
                        images: HashMap::new(),
                        path_to_id: HashMap::new(),
                        scripts: HashMap::new(),
                        script_path_to_id: HashMap::new(),
                        atlases: HashMap::new(),
                        atlas_path_to_id: HashMap::new(),
                        reload_rx: Some(rx),
                        _watcher: Some(w),
                    }
                }
                Err(e) => {
                    log::warn!("파일 감시 초기화 실패 (핫 리로딩 비활성): {e}");
                }
            }
        }
        Self {
            images: HashMap::new(),
            path_to_id: HashMap::new(),
            scripts: HashMap::new(),
            script_path_to_id: HashMap::new(),
            atlases: HashMap::new(),
            atlas_path_to_id: HashMap::new(),
            reload_rx: None,
            #[cfg(not(target_arch = "wasm32"))]
            _watcher: None,
        }
    }

    /// 이미지를 로드해 핸들을 반환한다. 같은 경로를 다시 호출하면 캐시된 핸들을 반환한다.
    pub fn load_image(&mut self, path: impl AsRef<Path>) -> Handle<ImageAsset> {
        let key: Arc<str> = path.as_ref().to_string_lossy().as_ref().into();
        if let Some(&id) = self.path_to_id.get(&key) {
            return Handle {
                id,
                path: key,
                _marker: PhantomData,
            };
        }
        let id = alloc_id();
        let asset = decode_image(&key);
        self.images.insert(id, asset);
        self.path_to_id.insert(Arc::clone(&key), id);
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

    /// 현재 캐시된 이미지 에셋 수를 반환한다.
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    /// CPU-side 이미지 데이터를 반환한다.
    pub fn get_image(&self, handle: &Handle<ImageAsset>) -> Option<&ImageAsset> {
        self.images.get(&handle.id)
    }

    /// 스크립트를 로드해 핸들을 반환한다. 같은 경로 재호출 시 캐시된 핸들 반환.
    pub fn load_script(&mut self, path: impl AsRef<Path>) -> Handle<ScriptAsset> {
        let key: Arc<str> = path.as_ref().to_string_lossy().as_ref().into();
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
        let key: Arc<str> = path.as_ref().to_string_lossy().as_ref().into();
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
        let rx = match &self.reload_rx {
            Some(r) => r,
            None => return Vec::new(),
        };
        let mut seen: Vec<String> = Vec::new();
        while let Ok(path) = rx.try_recv() {
            if let Some(s) = path.to_str() {
                let key: Arc<str> = s.into();
                let s_str = s.to_string();
                let is_known =
                    self.path_to_id.contains_key(&key) || self.script_path_to_id.contains_key(&key);
                if is_known && !seen.contains(&s_str) {
                    seen.push(s_str);
                }
            }
        }
        for path_str in &seen {
            let key: Arc<str> = path_str.as_str().into();
            if let Some(&id) = self.path_to_id.get(&key) {
                self.images.insert(id, decode_image(path_str));
            }
            if let Some(&id) = self.script_path_to_id.get(&key) {
                self.scripts.insert(id, compile_script_file(path_str));
            }
        }
        seen
    }
}

impl Default for AssetServer {
    fn default() -> Self {
        Self::new()
    }
}

// ─── 내부 헬퍼 ────────────────────────────────────────────────────────────────

fn decode_image(path: &str) -> ImageAsset {
    match std::fs::read(path) {
        Ok(bytes) => match image::load_from_memory(&bytes) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                ImageAsset {
                    data: Arc::new(rgba.into_raw()),
                    width: w,
                    height: h,
                }
            }
            Err(e) => {
                log::error!("이미지 디코딩 실패 '{path}': {e}");
                magenta_fallback()
            }
        },
        Err(e) => {
            log::error!("이미지 파일 읽기 실패 '{path}': {e}");
            magenta_fallback()
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
