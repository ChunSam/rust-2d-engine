use std::collections::HashMap;
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

// WASM: GPU 초기화는 async(WebGPU Promise 기반)이므로 thread_local로 결과를 전달한다.
#[cfg(target_arch = "wasm32")]
thread_local! {
    static PENDING_GPU: std::cell::RefCell<Option<(
        crate::renderer::GpuContext,
        Arc<winit::window::Window>,
    )>> = std::cell::RefCell::new(None);
}

use glam::Vec2;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

use crate::{
    asset::{AssetServer, Handle, ImageAsset},
    camera::Camera,
    debug_ui::DebugUi,
    ecs::{Entity, Events, System, World},
    hierarchy::HierarchySystem,
    input::{GamepadState, InputState},
    prefab::Tag,
    reflect::ReflectValue,
    renderer::{
        DrawRect, GpuContext, PostProcessConfig, PostProcessRenderer, SpriteRenderer, TextQueue,
        TextRenderer, UiQueue,
    },
    resources::{
        DebugDraw, DebugDrawQueue, DebugRect, DisplayScaleFactor, FontData, GameState,
        PendingResize, ShouldQuit, ViewportSize, WindowConfig,
    },
    scene::{Scene, SceneChange, SceneCmd},
};

// ─── 에디터 Undo/Redo ────────────────────────────────────────────────────────

/// Inspector에서 실행 취소 가능한 작업 목록.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
enum EditorCmd {
    MoveEntity {
        entity: Entity,
        old_pos: glam::Vec2,
        new_pos: glam::Vec2,
    },
    CreateEntity {
        entity: Entity,
    },
    DeleteEntity {
        tag: Option<String>,
        transform: Option<crate::components::Transform>,
        sprite: Option<crate::components::Sprite>,
    },
}

#[cfg(not(target_arch = "wasm32"))]
struct EditorHistory {
    undo: Vec<EditorCmd>,
    redo: Vec<EditorCmd>,
}

#[cfg(not(target_arch = "wasm32"))]
impl EditorHistory {
    fn new() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    fn push(&mut self, cmd: EditorCmd) {
        self.undo.push(cmd);
        self.redo.clear();
    }

    fn undo(&mut self, world: &mut World, selected: &mut Option<Entity>) {
        let Some(cmd) = self.undo.pop() else { return };
        match &cmd {
            EditorCmd::MoveEntity {
                entity, old_pos, ..
            } => {
                if let Some(t) = world.get_mut::<crate::components::Transform>(*entity) {
                    t.position = *old_pos;
                }
                *selected = Some(*entity);
            }
            EditorCmd::CreateEntity { entity } => {
                world.despawn(*entity);
                *selected = None;
            }
            EditorCmd::DeleteEntity {
                tag,
                transform,
                sprite,
            } => {
                let e = world.spawn();
                if let Some(tr) = transform {
                    world.add_component(e, tr.clone());
                }
                if let Some(sp) = sprite {
                    world.add_component(e, sp.clone());
                }
                if let Some(t) = tag {
                    world.add_component(e, Tag(t.clone()));
                }
                *selected = Some(e);
            }
        }
        self.redo.push(cmd);
    }

    fn redo(&mut self, world: &mut World, selected: &mut Option<Entity>) {
        let Some(cmd) = self.redo.pop() else { return };
        match &cmd {
            EditorCmd::MoveEntity {
                entity, new_pos, ..
            } => {
                if let Some(t) = world.get_mut::<crate::components::Transform>(*entity) {
                    t.position = *new_pos;
                }
                *selected = Some(*entity);
            }
            EditorCmd::CreateEntity { entity } => {
                // 엔티티가 이미 despawn 됐으므로 새로 스폰 (id가 달라짐 — 허용)
                let e = world.spawn();
                world.add_component(e, crate::components::Transform::default());
                world.add_component(e, Tag("New Entity".into()));
                *selected = Some(e);
                // redo stack의 cmd를 업데이트할 수 없으므로 이 분기는 새 entity로 처리
                drop(cmd);
                return;
            }
            EditorCmd::DeleteEntity { .. } => {
                if let Some(sel) = *selected {
                    world.despawn(sel);
                    *selected = None;
                }
            }
        }
        self.undo.push(cmd);
    }
}

/// Gizmo 위치를 격자(grid) 단위로 스냅한다 (네이티브 전용).
#[cfg(not(target_arch = "wasm32"))]
fn snap_to_grid(pos: glam::Vec2, snap_size: f32) -> glam::Vec2 {
    glam::Vec2::new(
        (pos.x / snap_size).round() * snap_size,
        (pos.y / snap_size).round() * snap_size,
    )
}

/// 엔진 진입점.
///
/// # 사용법
/// ```rust,no_run
/// # use engine::App;
/// let mut app = App::new();
/// app.world.spawn();
/// // app.add_system(MySystem);
/// app.run();
/// ```
pub struct App {
    /// ECS 세계 (엔티티·컴포넌트·리소스)
    pub world: World,

    systems: Vec<Box<dyn System>>,
    /// 시스템별 라벨/순서/그룹 메타데이터. systems와 동일한 인덱스로 평행 보관.
    system_meta: Vec<crate::ecs::schedule::SystemMeta>,
    /// compute_order가 계산한 실행 순서 (인덱스 목록).
    exec_order: Vec<usize>,
    /// system_meta가 변경됐을 때 true — 다음 프레임에 재계산.
    schedule_dirty: bool,
    /// 비활성화된 SystemSet 라벨. 해당 set의 시스템은 실행을 건너뛴다.
    disabled_sets: std::collections::HashSet<crate::ecs::schedule::SystemLabel>,
    /// (씬, 해당 씬이 등록한 시스템 수). Push/Pop 시 시스템 복원에 사용.
    scene_stack: Vec<(Box<dyn Scene>, usize)>,
    window: Option<Arc<Window>>,
    gpu: Option<GpuContext>,
    sprite_renderer: Option<SpriteRenderer>,
    /// 스프라이트 pass 직후 텍스트를 덮어쓴다. GPU 초기화 이후 Some으로 채워진다.
    text_renderer: Option<TextRenderer>,
    /// PostProcessConfig 리소스가 enabled=true일 때 활성화된다.
    post_renderer: Option<PostProcessRenderer>,
    /// AmbientLight 리소스가 등록된 동안 활성화되는 라이팅 렌더러.
    #[cfg(not(target_arch = "wasm32"))]
    lighting_renderer: Option<crate::renderer::lighting::LightingRenderer>,
    /// FadeTransition 리소스가 alpha > 0 일 때 마지막 패스로 실행되는 페이드 렌더러.
    #[cfg(not(target_arch = "wasm32"))]
    fade_renderer: Option<crate::renderer::fade::FadeRenderer>,
    /// 라이팅 패스가 씬을 먼저 그릴 중간 텍스처.
    #[cfg(not(target_arch = "wasm32"))]
    scene_texture_for_lighting: Option<(wgpu::Texture, wgpu::TextureView)>,
    last_frame: Option<Instant>,
    /// GPU 초기화 전에 등록된 텍스처 경로를 보관한다. resumed()에서 실제로 로드한다.
    pending_textures: Vec<String>,
    /// 매 프레임 종료 시 이벤트 큐를 비우는 클로저 목록.
    event_flushers: Vec<Box<dyn Fn(&mut World)>>,
    /// reload_scene 시 이벤트 리소스를 재삽입하는 클로저 목록.
    event_initializers: Vec<Box<dyn Fn(&mut World)>>,
    /// gilrs 게임패드 컨텍스트. 초기화 실패 시 None (게임패드 없이 동작).
    #[cfg(not(target_arch = "wasm32"))]
    gilrs: Option<gilrs::Gilrs>,
    /// egui 렌더러 (wgpu 백엔드).
    egui_renderer: Option<egui_wgpu::Renderer>,
    /// winit ↔ egui 이벤트 변환기.
    egui_state: Option<egui_winit::State>,
    /// update() 에서 tessellate 한 결과를 render() 까지 전달하는 임시 버퍼.
    egui_output: Option<(Vec<egui::ClippedPrimitive>, egui::TexturesDelta, f32)>,
    /// Inspector 패널에서 현재 선택된 엔티티.
    inspector_selected: Option<Entity>,
    /// Gizmo 드래그 중인지 여부.
    gizmo_dragging: bool,
    /// 드래그 시작 시 (엔티티 position - 커서 월드 좌표) 오프셋.
    gizmo_drag_offset: glam::Vec2,
    /// Inspector 씬 저장 경로 (네이티브 전용).
    editor_save_path: String,
    /// 마지막 씬 저장 결과 메시지.
    editor_save_status: Option<String>,
    /// 마지막 씬 로드 결과 메시지.
    editor_load_status: Option<String>,
    /// Inspector 현재 탭 인덱스 (0: Entities, 1: Assets).
    inspector_tab: u8,
    /// 에디터 실행 취소/다시 실행 히스토리.
    #[cfg(not(target_arch = "wasm32"))]
    cmd_history: EditorHistory,
    /// Gizmo 드래그 시작 시 엔티티 위치 (undo 기록용).
    #[cfg(not(target_arch = "wasm32"))]
    gizmo_drag_start_pos: Option<glam::Vec2>,
    /// 컴포넌트 추가 팩토리 맵 (네이티브 전용). 타입 이름 → World에 컴포넌트를 추가하는 클로저.
    #[cfg(not(target_arch = "wasm32"))]
    component_factories: HashMap<String, Box<dyn Fn(&mut World, Entity) + Send + Sync>>,
    /// "Add Component" 드롭다운에서 현재 선택된 컴포넌트 이름 (네이티브 전용).
    #[cfg(not(target_arch = "wasm32"))]
    add_component_selected: String,
    /// Gizmo 드래그 Grid Snap 활성화 여부 (네이티브 전용).
    #[cfg(not(target_arch = "wasm32"))]
    snap_enabled: bool,
    /// Gizmo 드래그 Grid Snap 격자 크기 (픽셀, 네이티브 전용).
    #[cfg(not(target_arch = "wasm32"))]
    snap_size: f32,
}

impl App {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let gilrs = gilrs::Gilrs::new().ok();
        let mut world = World::new();
        world.insert_resource(InputState::default());
        world.insert_resource(GamepadState::default());
        world.insert_resource(GameState::Playing);
        world.insert_resource(ShouldQuit(false));
        world.insert_resource(WindowConfig::default());
        world.insert_resource(ViewportSize::default());
        world.insert_resource(PendingResize::default());
        world.insert_resource(Camera::default());
        world.insert_resource(TextQueue::default());
        world.insert_resource(UiQueue::default());
        world.insert_resource(DebugDrawQueue::default());
        world.insert_resource(DebugDraw::new());
        world.insert_resource(crate::resources::SelectedEntity::default());
        world.insert_resource(crate::resources::ProfilerData::default());
        world.insert_resource(SceneChange::default());
        world.insert_resource(AssetServer::new());
        // 엔진 내장 컴포넌트를 Reflect 레지스트리에 자동 등록 (이름 포함)
        world.register_reflect_named::<crate::components::Transform>("Transform");
        world.register_reflect_named::<crate::components::Sprite>("Sprite");
        world.register_reflect_named::<crate::prefab::Tag>("Tag");
        // 엔진 내장 컴포넌트를 clone_entity 복제 레지스트리에 등록
        world.register_clone::<crate::components::Transform>();
        world.register_clone::<crate::components::Sprite>();
        world.register_clone::<crate::components::RenderLayer>();
        world.register_clone::<crate::prefab::Tag>();
        world.register_clone::<crate::animation::player::AnimationPlayer>();
        world.register_clone::<crate::timer::Timer>();

        #[cfg(not(target_arch = "wasm32"))]
        let mut app = Self {
            world,
            systems: Vec::new(),
            system_meta: Vec::new(),
            exec_order: Vec::new(),
            schedule_dirty: true,
            disabled_sets: std::collections::HashSet::new(),
            scene_stack: Vec::new(),
            window: None,
            gpu: None,
            sprite_renderer: None,
            text_renderer: None,
            post_renderer: None,
            lighting_renderer: None,
            fade_renderer: None,
            scene_texture_for_lighting: None,
            last_frame: None,
            pending_textures: Vec::new(),
            event_flushers: Vec::new(),
            event_initializers: Vec::new(),
            gilrs,
            egui_renderer: None,
            egui_state: None,
            egui_output: None,
            inspector_selected: None,
            gizmo_dragging: false,
            gizmo_drag_offset: glam::Vec2::ZERO,
            editor_save_path: "saved_scene.ron".into(),
            editor_save_status: None,
            editor_load_status: None,
            inspector_tab: 0,
            cmd_history: EditorHistory::new(),
            gizmo_drag_start_pos: None,
            component_factories: HashMap::new(),
            add_component_selected: String::new(),
            snap_enabled: false,
            snap_size: 16.0,
        };
        #[cfg(not(target_arch = "wasm32"))]
        app.register_default_components();
        #[cfg(not(target_arch = "wasm32"))]
        return app;

        #[cfg(target_arch = "wasm32")]
        Self {
            world,
            systems: Vec::new(),
            system_meta: Vec::new(),
            exec_order: Vec::new(),
            schedule_dirty: true,
            disabled_sets: std::collections::HashSet::new(),
            scene_stack: Vec::new(),
            window: None,
            gpu: None,
            sprite_renderer: None,
            text_renderer: None,
            post_renderer: None,
            last_frame: None,
            pending_textures: Vec::new(),
            event_flushers: Vec::new(),
            event_initializers: Vec::new(),
            egui_renderer: None,
            egui_state: None,
            egui_output: None,
            inspector_selected: None,
            gizmo_dragging: false,
            gizmo_drag_offset: glam::Vec2::ZERO,
            editor_save_path: "saved_scene.ron".into(),
            editor_save_status: None,
            editor_load_status: None,
            inspector_tab: 0,
        }
    }

    /// 기본 컴포넌트 팩토리를 등록한다 (네이티브 전용).
    #[cfg(not(target_arch = "wasm32"))]
    fn register_default_components(&mut self) {
        self.register_component("Sprite", |world, e| {
            world.add_component(e, crate::components::Sprite::default());
        });
        self.register_component("RenderLayer", |world, e| {
            world.add_component(e, crate::components::RenderLayer::default());
        });
        self.register_component("ParticleEmitter", |world, e| {
            world.add_component(e, crate::particle::ParticleEmitter::default());
        });
    }

    /// 컴포넌트 팩토리를 등록한다. "Add Component" 드롭다운에 해당 이름이 표시된다.
    ///
    /// 클로저는 엔티티에 원하는 컴포넌트(들)를 추가해야 한다.
    ///
    /// ```rust,no_run
    /// # use engine::App;
    /// # let mut app = App::new();
    /// app.register_component("MyComp", |world, entity| {
    ///     // world.add_component(entity, MyComp::default());
    /// });
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub fn register_component(
        &mut self,
        name: impl Into<String>,
        factory: impl Fn(&mut World, Entity) + Send + Sync + 'static,
    ) {
        self.component_factories
            .insert(name.into(), Box::new(factory));
    }

    /// 이벤트 타입 `E` 를 등록한다.
    ///
    /// - `Events::<E>` 리소스를 World에 삽입한다.
    /// - 매 프레임 종료 시 자동으로 `flush()` 가 호출된다.
    /// - `reload_scene()` 호출 시에도 이벤트 리소스가 재삽입된다.
    pub fn register_event<E: 'static>(&mut self) {
        self.world.insert_resource(Events::<E>::default());
        self.event_flushers.push(Box::new(|world: &mut World| {
            if let Some(events) = world.resource_mut::<Events<E>>() {
                events.flush();
            }
        }));
        self.event_initializers.push(Box::new(|world: &mut World| {
            world.insert_resource(Events::<E>::default());
        }));
    }

    /// 시스템을 등록한다. 매 프레임 등록 순서대로 실행된다.
    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
        self.system_meta
            .push(crate::ecs::schedule::SystemMeta::default());
        self.schedule_dirty = true;
    }

    /// 라벨/순서/그룹 설정과 함께 시스템을 등록한다.
    pub fn add_system_labeled<S: System + 'static>(
        &mut self,
        system: S,
        config: crate::ecs::schedule::SystemConfig,
    ) {
        self.systems.push(Box::new(system));
        self.system_meta.push(config.into());
        self.schedule_dirty = true;
    }

    /// SystemSet 활성/비활성. 비활성 set의 시스템은 매 프레임 실행에서 제외된다.
    pub fn set_enabled(
        &mut self,
        set: crate::ecs::schedule::SystemLabel,
        enabled: bool,
    ) {
        if enabled {
            self.disabled_sets.remove(set);
        } else {
            self.disabled_sets.insert(set);
        }
    }

    /// PNG 텍스처를 로드 대기열에 추가한다.
    ///
    /// GPU가 준비되기 전(`run()` 호출 전)에도 안전하게 호출할 수 있다.
    /// 실제 GPU 업로드는 `resumed()` 시점에 일괄 처리된다.
    pub fn load_texture(&mut self, path: impl Into<String>) {
        self.pending_textures.push(path.into());
    }

    /// AssetServer를 통해 이미지를 로드하고 `Handle<ImageAsset>`을 반환한다.
    ///
    /// - CPU-side 이미지 데이터와 파일 감시를 등록한다.
    /// - GPU 텍스처 업로드는 `resumed()` 이후 또는 즉시(GPU 준비된 경우) 처리된다.
    /// - 같은 경로를 다시 호출하면 캐시된 핸들이 반환된다.
    pub fn load_image(&mut self, path: impl Into<String>) -> Handle<ImageAsset> {
        let path = path.into();
        self.pending_textures.push(path.clone());
        self.world
            .resource_mut::<AssetServer>()
            .expect("AssetServer 리소스 누락")
            .load_image(&path)
    }

    /// 텍스처 아틀라스를 로드하고 `Handle<TextureAtlas>`를 반환한다.
    ///
    /// - `cols × rows` 균일 그리드로 분할된 단일 이미지 파일을 아틀라스로 등록한다.
    /// - 내부적으로 이미지를 CPU(AssetServer)와 GPU(SpriteRenderer) 양쪽에 로드한다.
    /// - 같은 경로를 다시 호출하면 캐시된 핸들을 반환한다.
    pub fn load_atlas(
        &mut self,
        path: impl Into<String>,
        cols: u32,
        rows: u32,
    ) -> Handle<crate::atlas::TextureAtlas> {
        let path = path.into();
        self.pending_textures.push(path.clone());
        self.world
            .resource_mut::<AssetServer>()
            .expect("AssetServer 없음")
            .load_atlas(&path, cols, rows)
    }

    /// 스크립트 파일을 로드하고 핸들을 반환한다.
    pub fn load_script(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Handle<crate::asset::ScriptAsset> {
        self.world
            .resource_mut::<AssetServer>()
            .expect("AssetServer 없음")
            .load_script(path)
    }

    /// ECS 월드를 초기화하고 기본 리소스를 재삽입한다.
    ///
    /// 씬 전환 시 엔티티·컴포넌트를 전부 지우고 싶을 때 사용한다.
    /// 시스템은 유지되므로 필요하면 `add_system`으로 새로 등록한다.
    pub fn reload_scene(&mut self) {
        self.world = World::new();
        self.world.insert_resource(InputState::default());
        self.world.insert_resource(GamepadState::default());
        self.world.insert_resource(GameState::Playing);
        self.world.insert_resource(ShouldQuit(false));
        self.world.insert_resource(WindowConfig::default());
        self.world.insert_resource(ViewportSize::default());
        self.world.insert_resource(PendingResize::default());
        self.world.insert_resource(Camera::default());
        self.world.insert_resource(TextQueue::default());
        self.world.insert_resource(UiQueue::default());
        self.world.insert_resource(DebugDrawQueue::default());
        self.world.insert_resource(DebugDraw::new());
        self.world
            .insert_resource(crate::resources::SelectedEntity::default());
        self.world
            .insert_resource(crate::resources::ProfilerData::default());
        self.world.insert_resource(SceneChange::default());
        self.world.insert_resource(AssetServer::new());
        // 등록된 이벤트 리소스 재삽입
        let inits = std::mem::take(&mut self.event_initializers);
        for init in &inits {
            init(&mut self.world);
        }
        self.event_initializers = inits;
        // Reflect 레지스트리 재등록 (World 재생성으로 초기화되었으므로)
        self.world
            .register_reflect_named::<crate::components::Transform>("Transform");
        self.world
            .register_reflect_named::<crate::components::Sprite>("Sprite");
        self.world
            .register_reflect_named::<crate::prefab::Tag>("Tag");
        // clone_entity 복제 레지스트리 재등록
        self.world.register_clone::<crate::components::Transform>();
        self.world.register_clone::<crate::components::Sprite>();
        self.world
            .register_clone::<crate::components::RenderLayer>();
        self.world.register_clone::<crate::prefab::Tag>();
        self.world
            .register_clone::<crate::animation::player::AnimationPlayer>();
        self.world.register_clone::<crate::timer::Timer>();
        self.inspector_selected = None;
        self.editor_save_status = None;
    }

    /// 씬을 즉시 전환한다. `run()` 호출 전·후 모두 사용 가능하다.
    ///
    /// 현재 씬 스택을 전부 종료하고 월드를 리셋한 뒤 `scene`을 진입시킨다.
    pub fn set_scene(&mut self, scene: Box<dyn Scene>) {
        self.apply_scene_cmd(SceneCmd::Replace(scene));
    }

    // ── DebugDraw 도형 → DrawRect 변환 ──────────────────────────────────────

    /// `DebugShape`를 `UiQueue`에 `DrawRect` 목록으로 변환한다.
    fn debug_shape_to_draw_rects(shape: crate::resources::DebugShape, q: &mut UiQueue) {
        use crate::resources::DebugShape;
        const Z: f32 = 999.0;

        // 선분 근사 헬퍼: 두 점 사이를 thickness×thickness 점들로 채운다.
        let mut push_line =
            |start: glam::Vec2, end: glam::Vec2, color: [f32; 4], thickness: f32| {
                let delta = end - start;
                let len = delta.length();
                if len < 0.001 {
                    return;
                }
                let steps = (len / thickness.max(0.5)).ceil() as usize;
                let half = thickness / 2.0;
                for i in 0..=steps {
                    let t = i as f32 / steps.max(1) as f32;
                    let pos = start + delta * t;
                    q.push(
                        DrawRect::new(pos.x - half, pos.y - half, thickness, thickness, color)
                            .with_z(Z),
                    );
                }
            };

        match shape {
            DebugShape::Rect { min, max, color } => {
                let t = 1.5_f32;
                let w = max.x - min.x;
                let h = max.y - min.y;
                // 위
                q.push(DrawRect::new(min.x, min.y, w, t, color).with_z(Z));
                // 아래
                q.push(DrawRect::new(min.x, max.y - t, w, t, color).with_z(Z));
                // 왼쪽
                q.push(DrawRect::new(min.x, min.y, t, h, color).with_z(Z));
                // 오른쪽
                q.push(DrawRect::new(max.x - t, min.y, t, h, color).with_z(Z));
            }
            DebugShape::Line {
                start,
                end,
                color,
                thickness,
            } => {
                push_line(start, end, color, thickness);
            }
            DebugShape::Circle {
                center,
                radius,
                color,
            } => {
                let n = 24u32;
                for i in 0..n {
                    let a0 = (i as f32 / n as f32) * std::f32::consts::TAU;
                    let a1 = ((i + 1) as f32 / n as f32) * std::f32::consts::TAU;
                    let p0 = center + glam::Vec2::new(a0.cos(), a0.sin()) * radius;
                    let p1 = center + glam::Vec2::new(a1.cos(), a1.sin()) * radius;
                    push_line(p0, p1, color, 1.5);
                }
            }
            DebugShape::Cross { pos, size, color } => {
                let half = size / 2.0;
                push_line(
                    pos - glam::Vec2::X * half,
                    pos + glam::Vec2::X * half,
                    color,
                    1.5,
                );
                push_line(
                    pos - glam::Vec2::Y * half,
                    pos + glam::Vec2::Y * half,
                    color,
                    1.5,
                );
            }
        }
    }

    // ── 씬 전환 처리 ─────────────────────────────────────────────────────────

    /// systems와 system_meta 길이를 맞추고 schedule_dirty를 표시한다.
    /// 씬 on_enter 또는 truncate 이후 반드시 호출.
    fn reconcile_meta(&mut self) {
        use crate::ecs::schedule::SystemMeta;
        if self.system_meta.len() < self.systems.len() {
            self.system_meta
                .resize(self.systems.len(), SystemMeta::default());
        } else if self.system_meta.len() > self.systems.len() {
            self.system_meta.truncate(self.systems.len());
        }
        self.schedule_dirty = true;
    }

    fn apply_scene_cmd(&mut self, cmd: SceneCmd) {
        match cmd {
            SceneCmd::Replace(mut new_scene) => {
                for (mut scene, _) in self.scene_stack.drain(..).rev() {
                    scene.on_exit(&mut self.world);
                }
                self.systems.clear();
                self.reconcile_meta(); // systems.clear() 후 meta 동기화
                self.reload_scene();
                let before = self.systems.len();
                new_scene.on_enter(&mut self.world, &mut self.systems);
                let owned = self.systems.len() - before;
                self.scene_stack.push((new_scene, owned));
                self.reconcile_meta(); // on_enter 후 씬이 직접 push한 시스템 흡수
            }
            SceneCmd::Push(mut new_scene) => {
                let before = self.systems.len();
                new_scene.on_enter(&mut self.world, &mut self.systems);
                let owned = self.systems.len() - before;
                self.scene_stack.push((new_scene, owned));
                self.reconcile_meta(); // on_enter 후 씬이 직접 push한 시스템 흡수
            }
            SceneCmd::Pop => {
                if let Some((mut scene, owned)) = self.scene_stack.pop() {
                    scene.on_exit(&mut self.world);
                    let new_len = self.systems.len().saturating_sub(owned);
                    self.systems.truncate(new_len);
                    self.reconcile_meta(); // truncate 후 meta 동기화
                }
            }
        }
    }

    /// 이벤트 루프를 시작한다. 창이 닫힐 때까지 블로킹된다.
    #[allow(unused_mut)]
    pub fn run(mut self) {
        let event_loop = EventLoop::new().expect("이벤트 루프 생성 실패");
        #[cfg(not(target_arch = "wasm32"))]
        event_loop.run_app(&mut self).expect("이벤트 루프 오류");
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::EventLoopExtWebSys;
            event_loop.spawn_app(self);
        }
    }

    // ── 내부 메서드 ─────────────────────────────────────────────────────────

    fn update(&mut self, dt: f32) {
        self.world.clear_change_tracking();
        // GPU 실제 크기는 물리 픽셀이고, 게임 좌표계는 논리 픽셀이다.
        // Retina/HiDPI에서 이 둘을 분리해야 스프라이트와 UI가 절반 크기로 보이지 않는다.
        if let Some(gpu) = &self.gpu {
            let scale_factor = self
                .window
                .as_ref()
                .map(|w| w.scale_factor() as f32)
                .unwrap_or(1.0)
                .max(1.0);
            self.world.insert_resource(ViewportSize {
                width: gpu.config.width as f32 / scale_factor,
                height: gpu.config.height as f32 / scale_factor,
            });
            self.world.insert_resource(DisplayScaleFactor(scale_factor));
        }

        // egui 프레임 시작
        let egui_ctx: Option<egui::Context> = {
            let window = self.window.as_ref();
            let state = self.egui_state.as_mut();
            if let (Some(window), Some(state)) = (window, state) {
                if let Some(debug_ui) = self.world.resource::<DebugUi>() {
                    let ctx = debug_ui.ctx().clone();
                    let raw_input = state.take_egui_input(window);
                    ctx.begin_pass(raw_input);
                    Some(ctx)
                } else {
                    None
                }
            } else {
                None
            }
        };

        // 스케줄 재계산 (라벨/순서 변경 시)
        if self.schedule_dirty {
            // 안전: 씬 직접 push 흡수
            if self.system_meta.len() != self.systems.len() {
                self.system_meta.resize(
                    self.systems.len(),
                    crate::ecs::schedule::SystemMeta::default(),
                );
            }
            match crate::ecs::schedule::compute_order(&self.system_meta) {
                Ok(order) => self.exec_order = order,
                Err(crate::ecs::schedule::ScheduleError::Cycle(remaining)) => {
                    log::error!(
                        "시스템 순서 순환 의존성 감지 — 삽입 순서로 폴백 (영향 인덱스: {:?})",
                        remaining
                    );
                    self.exec_order = (0..self.systems.len()).collect();
                }
            }
            self.schedule_dirty = false;
        }

        // 시스템 실행 + 프로파일러 계측 (exec_order 순회, 비활성 set 스킵)
        {
            let system_count = self.systems.len();
            let mut timings: Vec<(usize, &'static str, u64)> = Vec::with_capacity(system_count);
            let order = self.exec_order.clone();
            for i in order {
                if i >= self.systems.len() {
                    continue;
                }
                if let Some(set) = self.system_meta.get(i).and_then(|m| m.set) {
                    if self.disabled_sets.contains(set) {
                        continue;
                    }
                }
                let name = self.systems[i].name();
                let t0 = Instant::now();
                self.systems[i].run(&mut self.world, dt);
                timings.push((i, name, t0.elapsed().as_micros() as u64));
            }
            if let Some(prof) = self.world.resource_mut::<crate::resources::ProfilerData>() {
                if prof.systems.len() != system_count {
                    prof.systems.clear();
                    prof.systems
                        .resize(system_count, crate::resources::SystemProfile::default());
                }
                for (i, name, us) in timings {
                    prof.record_system(i, name, us);
                }
                prof.frame_ms = dt * 1000.0;
            }
        }
        // 계층 변환 전파 — 유저 시스템(물리 포함) 이후, 렌더 직전에 실행
        HierarchySystem.run(&mut self.world, dt);

        // 카메라 이펙트 업데이트 (shake decay, zoom tween, smooth follow)
        {
            let follow_pos = self
                .world
                .resource::<Camera>()
                .and_then(|cam| cam.follow_entity)
                .and_then(|e| self.world.get::<crate::components::Transform>(e))
                .map(|t| t.position);
            if let Some(cam) = self.world.resource_mut::<Camera>() {
                cam.update(dt, follow_pos);
            }
        }

        // Inspector: 선택된 엔티티 유효성 확인 + 필드 스테이징
        if let Some(sel) = self.inspector_selected {
            if !self.world.is_alive(sel) {
                self.inspector_selected = None;
            }
        }
        let entity_list: Vec<Entity> = self.world.entities().to_vec();
        let tag_map: HashMap<Entity, String> = self
            .world
            .query::<Tag>()
            .map(|(e, t)| (e, t.0.clone()))
            .collect();
        let mut comp_fields: Vec<(&'static str, Vec<(&'static str, ReflectValue)>)> = Vec::new();
        if let Some(sel) = self.inspector_selected {
            for tid in self.world.reflected_components(sel) {
                if let Some(refl) = self.world.get_reflect(sel, tid) {
                    comp_fields.push((refl.type_name(), refl.fields()));
                }
            }
        }
        // 선택 엔티티가 가진 Reflect 등록 컴포넌트 이름 목록 (네이티브 전용, 컴포넌트 관리 UI용).
        // comp_fields에서 이름을 추출하면 borrow 충돌 없이 안전하다.
        #[cfg(not(target_arch = "wasm32"))]
        let selected_comp_names: Vec<&'static str> =
            comp_fields.iter().map(|(name, _)| *name).collect();

        // ── 씬 그래프 데이터 사전 수집 (네이티브 전용) ──────────────────────────
        // borrow checker 우회: egui 클로저 진입 전에 계층 구조를 모두 복사해 둔다.
        #[cfg(not(target_arch = "wasm32"))]
        let scene_graph_data: Vec<(Entity, Option<Entity>)> = {
            // (entity, parent_entity_or_none)
            entity_list
                .iter()
                .map(|&e| {
                    let parent = self.world.get::<crate::hierarchy::Parent>(e).map(|p| p.0);
                    (e, parent)
                })
                .collect()
        };
        // children_map: 부모 → 자식 목록
        #[cfg(not(target_arch = "wasm32"))]
        let children_map: HashMap<Entity, Vec<Entity>> = {
            let mut map: HashMap<Entity, Vec<Entity>> = HashMap::new();
            for &(child, parent_opt) in &scene_graph_data {
                if let Some(parent) = parent_opt {
                    map.entry(parent).or_default().push(child);
                }
            }
            map
        };
        // 루트 엔티티 = Parent 컴포넌트 없는 것
        #[cfg(not(target_arch = "wasm32"))]
        let root_entities: Vec<Entity> = scene_graph_data
            .iter()
            .filter_map(|&(e, p)| if p.is_none() { Some(e) } else { None })
            .collect();

        // 내장 EngineStats 패널 + Inspector
        if let Some(ctx) = &egui_ctx {
            // ── Undo (Ctrl+Z) / Redo (Ctrl+Shift+Z) ─────────────────────────
            #[cfg(not(target_arch = "wasm32"))]
            {
                let (want_undo, want_redo) = ctx.input(|i| {
                    let ctrl = i.modifiers.ctrl;
                    let z = i.key_pressed(egui::Key::Z);
                    let shift = i.modifiers.shift;
                    (ctrl && z && !shift, ctrl && z && shift)
                });
                if want_undo {
                    let mut sel = self.inspector_selected;
                    self.cmd_history.undo(&mut self.world, &mut sel);
                    self.inspector_selected = sel;
                }
                if want_redo {
                    let mut sel = self.inspector_selected;
                    self.cmd_history.redo(&mut self.world, &mut sel);
                    self.inspector_selected = sel;
                }
            }
            if self
                .world
                .resource::<DebugUi>()
                .map(|d| d.is_enabled())
                .unwrap_or(false)
            {
                let entity_count = self.world.entity_count();
                let asset_count = self
                    .world
                    .resource::<AssetServer>()
                    .map(|a| a.image_count())
                    .unwrap_or(0);
                egui::Window::new("Engine Stats")
                    .default_pos([10.0, 10.0])
                    .resizable(true)
                    .show(ctx, |ui| {
                        ui.label(format!("FPS   {:>6.1}", 1.0_f32 / dt.max(0.001)));
                        ui.label(format!("ms    {:>6.2}", dt * 1000.0));
                        ui.label(format!("Ent   {entity_count}"));
                        ui.label(format!("Asset {asset_count}"));
                        ui.separator();
                        if let Some(prof) = self.world.resource::<crate::resources::ProfilerData>()
                        {
                            ui.collapsing("Systems", |ui| {
                                egui::Grid::new("sys_prof")
                                    .num_columns(2)
                                    .striped(true)
                                    .show(ui, |ui| {
                                        for sys in &prof.systems {
                                            let label = if sys.name.is_empty() {
                                                "anonymous"
                                            } else {
                                                &sys.name
                                            };
                                            ui.label(label);
                                            ui.label(format!("{:.0} µs", sys.avg_us));
                                            ui.end_row();
                                        }
                                    });
                            });
                            let r = prof.render;
                            ui.collapsing("Render", |ui| {
                                ui.label(format!("draw calls  {}", r.draw_calls));
                                ui.label(format!("rendered    {}", r.sprites_rendered));
                                ui.label(format!("culled      {}", r.sprites_culled));
                            });
                        }
                    });

                // Inspector 패널: 엔티티 목록 + 컴포넌트 필드 편집기 + 에셋 브라우저
                egui::Window::new("Inspector")
                    .default_pos([10.0, 130.0])
                    .default_size([440.0, 380.0])
                    .show(ctx, |ui| {
                        // ── 탭 선택 ──────────────────────────────────────────────
                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(self.inspector_tab == 0, "Entities")
                                .clicked()
                            {
                                self.inspector_tab = 0;
                            }
                            if ui
                                .selectable_label(self.inspector_tab == 1, "Assets")
                                .clicked()
                            {
                                self.inspector_tab = 1;
                            }
                            #[cfg(not(target_arch = "wasm32"))]
                            if ui
                                .selectable_label(self.inspector_tab == 2, "Scene")
                                .clicked()
                            {
                                self.inspector_tab = 2;
                            }
                        });
                        ui.separator();

                        // ── Grid Snap 컨트롤 (Entities 탭, 네이티브 전용) ─────────
                        #[cfg(not(target_arch = "wasm32"))]
                        if self.inspector_tab == 0 {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut self.snap_enabled, "Snap");
                                if self.snap_enabled {
                                    ui.add(
                                        egui::DragValue::new(&mut self.snap_size)
                                            .range(1.0..=128.0)
                                            .speed(1.0)
                                            .suffix(" px"),
                                    );
                                }
                            });
                        }

                        // ── 씬 그래프 탭 (네이티브 전용) ─────────────────────────
                        #[cfg(not(target_arch = "wasm32"))]
                        if self.inspector_tab == 2 {
                            // 씬 그래프: 루트 → 자식 들여쓰기 트리
                            let mut clicked_entity: Option<Entity> = None;

                            egui::ScrollArea::vertical()
                                .id_salt("scene_graph")
                                .max_height(300.0)
                                .show(ui, |ui| {
                                    // 재귀 대신 스택 기반 DFS (클로저 안에서 fn 호출 불가 제약 우회)
                                    let mut stack: Vec<(Entity, usize)> = root_entities
                                        .iter()
                                        .rev()
                                        .map(|&e| (e, 0))
                                        .collect();
                                    while let Some((entity, depth)) = stack.pop() {
                                        let name = tag_map
                                            .get(&entity)
                                            .cloned()
                                            .unwrap_or_else(|| format!("Entity {}", entity.0));
                                        let is_selected =
                                            self.inspector_selected == Some(entity);
                                        let has_children = children_map
                                            .get(&entity)
                                            .map(|c| !c.is_empty())
                                            .unwrap_or(false);
                                        let prefix = if has_children { "▶ " } else { "  " };
                                        let label_text =
                                            format!("{}{}{}", "  ".repeat(depth), prefix, name);

                                        let response = ui.selectable_label(
                                            is_selected,
                                            &label_text,
                                        );
                                        if response.clicked() {
                                            clicked_entity = Some(entity);
                                        }

                                        // 자식을 역순으로 스택에 push (DFS 순서 유지)
                                        if let Some(ch) = children_map.get(&entity) {
                                            for &child in ch.iter().rev() {
                                                stack.push((child, depth + 1));
                                            }
                                        }
                                    }
                                });

                            if let Some(e) = clicked_entity {
                                self.inspector_selected = Some(e);
                            }

                            // 선택된 엔티티의 Tag 이름 편집
                            ui.separator();
                            if let Some(sel) = self.inspector_selected {
                                let current_name = tag_map
                                    .get(&sel)
                                    .cloned()
                                    .unwrap_or_default();
                                let has_tag = self.world.get::<Tag>(sel).is_some();
                                ui.horizontal(|ui| {
                                    ui.label("Name:");
                                    if has_tag {
                                        let mut name_buf = current_name.clone();
                                        if ui.text_edit_singleline(&mut name_buf).changed() {
                                            self.world.add_component(sel, Tag(name_buf));
                                        }
                                    } else {
                                        ui.label(format!("Entity {}", sel.0));
                                        if ui.button("Add Name").clicked() {
                                            self.world.add_component(
                                                sel,
                                                Tag(format!("Entity {}", sel.0)),
                                            );
                                        }
                                    }
                                });
                            } else {
                                ui.label("(no entity selected)");
                            }
                        }

                        if self.inspector_tab == 1 {
                            // ── 에셋 브라우저 ─────────────────────────────────────
                            let entries = self
                                .world
                                .resource::<AssetServer>()
                                .map(|a| a.image_list())
                                .unwrap_or_default();
                            if entries.is_empty() {
                                ui.label("(No images loaded)");
                            } else {
                                egui::ScrollArea::vertical()
                                    .id_salt("asset_browser")
                                    .max_height(300.0)
                                    .show(ui, |ui| {
                                        egui::Grid::new("asset_grid")
                                            .num_columns(2)
                                            .spacing([8.0, 4.0])
                                            .show(ui, |ui| {
                                                for entry in &entries {
                                                    let filename =
                                                        std::path::Path::new(&entry.path)
                                                            .file_name()
                                                            .map(|f| {
                                                                f.to_string_lossy().into_owned()
                                                            })
                                                            .unwrap_or_else(|| {
                                                                entry.path.clone()
                                                            });
                                                    ui.label("[ ]");
                                                    ui.vertical(|ui| {
                                                        ui.label(&filename);
                                                        ui.small(format!(
                                                            "{}×{}",
                                                            entry.width, entry.height
                                                        ));
                                                    });
                                                    ui.end_row();
                                                }
                                            });
                                    });
                            }
                        } else {
                        // ── 에디터 액션 버튼 ─────────────────────────────────────
                        ui.horizontal(|ui| {
                            if ui.button("＋ New Entity").clicked() {
                                let e = self.world.spawn();
                                self.world.add_component(
                                    e,
                                    crate::components::Transform::default(),
                                );
                                self.world.add_component(
                                    e,
                                    crate::prefab::Tag("New Entity".into()),
                                );
                                self.inspector_selected = Some(e);
                                #[cfg(not(target_arch = "wasm32"))]
                                self.cmd_history.push(EditorCmd::CreateEntity { entity: e });
                            }
                            if let Some(sel) = self.inspector_selected {
                                if ui
                                    .add_enabled(true, egui::Button::new("🗑 Delete"))
                                    .clicked()
                                {
                                    #[cfg(not(target_arch = "wasm32"))]
                                    {
                                        let tag = self.world.get::<crate::prefab::Tag>(sel).map(|t| t.0.clone());
                                        let transform = self.world.get::<crate::components::Transform>(sel).cloned();
                                        let sprite = self.world.get::<crate::components::Sprite>(sel).cloned();
                                        self.cmd_history.push(EditorCmd::DeleteEntity { tag, transform, sprite });
                                    }
                                    self.world.despawn(sel);
                                    self.inspector_selected = None;
                                }
                                if ui
                                    .add_enabled(true, egui::Button::new("⎘ Duplicate"))
                                    .clicked()
                                {
                                    let new_entity = self.world.clone_entity(sel);
                                    if let Some(t) = self.world.get_mut::<crate::components::Transform>(new_entity) {
                                        t.position += glam::Vec2::new(16.0, 16.0);
                                    }
                                    self.inspector_selected = Some(new_entity);
                                }
                            }
                        });
                        ui.separator();
                        ui.horizontal_top(|ui| {
                            // 왼쪽: 엔티티 목록
                            ui.vertical(|ui| {
                                ui.set_min_width(130.0);
                                ui.strong("Entities");
                                egui::ScrollArea::vertical()
                                    .id_salt("inspector_ent")
                                    .max_height(250.0)
                                    .show(ui, |ui| {
                                        for &e in &entity_list {
                                            let label = tag_map
                                                .get(&e)
                                                .cloned()
                                                .unwrap_or_else(|| format!("E{}", e.0));
                                            if ui
                                                .selectable_label(
                                                    self.inspector_selected == Some(e),
                                                    &label,
                                                )
                                                .clicked()
                                            {
                                                self.inspector_selected = Some(e);
                                            }
                                        }
                                    });
                            });
                            ui.separator();
                            // 오른쪽: 컴포넌트 필드 편집기
                            ui.vertical(|ui| {
                                ui.strong("Components");
                                egui::ScrollArea::vertical()
                                    .id_salt("inspector_comp")
                                    .max_height(250.0)
                                    .show(ui, |ui| {
                                        for (comp_name, fields) in comp_fields.iter_mut() {
                                            ui.collapsing(*comp_name, |ui| {
                                                egui::Grid::new(*comp_name)
                                                    .num_columns(2)
                                                    .spacing([4.0, 2.0])
                                                    .show(ui, |ui| {
                                                        for (fname, fval) in fields.iter_mut() {
                                                            ui.label(*fname);
                                                            match fval {
                                                                ReflectValue::F32(v) => {
                                                                    ui.add(
                                                                        egui::DragValue::new(v)
                                                                            .speed(0.5),
                                                                    );
                                                                }
                                                                ReflectValue::Vec2(v) => {
                                                                    ui.horizontal(|ui| {
                                                                        ui.add(
                                                                            egui::DragValue::new(
                                                                                &mut v.x,
                                                                            )
                                                                            .speed(0.5)
                                                                            .prefix("x:"),
                                                                        );
                                                                        ui.add(
                                                                            egui::DragValue::new(
                                                                                &mut v.y,
                                                                            )
                                                                            .speed(0.5)
                                                                            .prefix("y:"),
                                                                        );
                                                                    });
                                                                }
                                                                ReflectValue::Bool(v) => {
                                                                    ui.checkbox(v, "");
                                                                }
                                                                ReflectValue::String(s) => {
                                                                    ui.text_edit_singleline(s);
                                                                }
                                                                ReflectValue::Color(c) => {
                                                                    ui.horizontal(|ui| {
                                                                        ui.add(
                                                                            egui::DragValue::new(
                                                                                &mut c[0],
                                                                            )
                                                                            .speed(0.01)
                                                                            .prefix("r:"),
                                                                        );
                                                                        ui.add(
                                                                            egui::DragValue::new(
                                                                                &mut c[1],
                                                                            )
                                                                            .speed(0.01)
                                                                            .prefix("g:"),
                                                                        );
                                                                        ui.add(
                                                                            egui::DragValue::new(
                                                                                &mut c[2],
                                                                            )
                                                                            .speed(0.01)
                                                                            .prefix("b:"),
                                                                        );
                                                                        ui.add(
                                                                            egui::DragValue::new(
                                                                                &mut c[3],
                                                                            )
                                                                            .speed(0.01)
                                                                            .prefix("a:"),
                                                                        );
                                                                    });
                                                                }
                                                            }
                                                            ui.end_row();
                                                        }
                                                    });
                                            });
                                        }
                                    });
                            });
                        });

                        // ── 컴포넌트 추가/제거 (네이티브 전용, Phase 39b) ────────────
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Some(sel) = self.inspector_selected {
                            ui.separator();
                            ui.strong("Component List");

                            // 제거할 컴포넌트 이름 (클로저 밖에서 결정)
                            let mut to_remove: Option<&'static str> = None;
                            for &comp_name in &selected_comp_names {
                                ui.horizontal(|ui| {
                                    ui.label(comp_name);
                                    if comp_name != "Transform" {
                                        if ui.small_button("✕").clicked() {
                                            to_remove = Some(comp_name);
                                        }
                                    }
                                });
                            }

                            // 클로저 종료 후 실제 제거
                            if let Some(name) = to_remove {
                                match name {
                                    "Sprite" => {
                                        self.world.remove_component::<crate::components::Sprite>(sel);
                                    }
                                    "Tag" => {
                                        self.world.remove_component::<crate::prefab::Tag>(sel);
                                    }
                                    "RenderLayer" => {
                                        self.world.remove_component::<crate::components::RenderLayer>(sel);
                                    }
                                    "ParticleEmitter" => {
                                        self.world.remove_component::<crate::particle::ParticleEmitter>(sel);
                                    }
                                    _ => {}
                                }
                            }

                            ui.separator();
                            // Add Component 드롭다운
                            let factory_names: Vec<String> = {
                                let mut names: Vec<String> =
                                    self.component_factories.keys().cloned().collect();
                                names.sort();
                                names
                            };
                            if !factory_names.is_empty() {
                                if self.add_component_selected.is_empty() {
                                    self.add_component_selected =
                                        factory_names[0].clone();
                                }
                                let cur = self.add_component_selected.clone();
                                egui::ComboBox::from_id_salt("add_comp_combo")
                                    .selected_text(&cur)
                                    .show_ui(ui, |ui| {
                                        for name in &factory_names {
                                            ui.selectable_value(
                                                &mut self.add_component_selected,
                                                name.clone(),
                                                name,
                                            );
                                        }
                                    });
                                if ui.button("+ Add").clicked() {
                                    let chosen = self.add_component_selected.clone();
                                    if let Some(factory) =
                                        self.component_factories.get(&chosen)
                                    {
                                        factory(&mut self.world, sel);
                                    }
                                }
                            }
                        }

                        // ── 선택 엔티티 이름(Tag) 편집 (네이티브 전용) ──────────────
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Some(sel) = self.inspector_selected {
                            ui.separator();
                            let current_name =
                                tag_map.get(&sel).cloned().unwrap_or_default();
                            let has_tag = self.world.get::<Tag>(sel).is_some();
                            ui.horizontal(|ui| {
                                ui.label("Name:");
                                if has_tag {
                                    let mut name_buf = current_name;
                                    if ui.text_edit_singleline(&mut name_buf).changed() {
                                        self.world.add_component(sel, Tag(name_buf));
                                    }
                                } else {
                                    ui.label(format!("Entity {}", sel.0));
                                    if ui.button("Add Name").clicked() {
                                        self.world.add_component(
                                            sel,
                                            Tag(format!("Entity {}", sel.0)),
                                        );
                                    }
                                }
                            });
                        }
                        // ── 씬 저장 (Phase 28) ───────────────────────────────────
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label("Path:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.editor_save_path)
                                        .desired_width(180.0),
                                );
                                if ui.button("📂 Load Scene").clicked() {
                                    let path = std::path::Path::new(&self.editor_save_path);
                                    match crate::prefab::SceneDef::load(path) {
                                        Ok(scene_def) => {
                                            // 기존 에디터 엔티티(Transform 또는 Tag 보유) 제거
                                            let to_remove: Vec<Entity> = self
                                                .world
                                                .query::<crate::components::Transform>()
                                                .map(|(e, _)| e)
                                                .collect();
                                            for e in to_remove {
                                                self.world.despawn(e);
                                            }
                                            self.inspector_selected = None;
                                            let count = scene_def.entities.len();
                                            crate::prefab::spawn_scene_def(
                                                &mut self.world,
                                                &scene_def,
                                            );
                                            self.editor_load_status =
                                                Some(format!("✓ {count} entities ← {}", self.editor_save_path));
                                            self.editor_save_status = None;
                                        }
                                        Err(e) => {
                                            self.editor_load_status = Some(format!("✗ {e}"));
                                        }
                                    }
                                }
                                if ui.button("💾 Save Scene").clicked() {
                                    let mut scene_def = crate::prefab::SceneDef::default();
                                    // 부모가 자식보다 먼저 나오도록 위상 정렬
                                    let sorted = crate::prefab::topological_sort_entities(
                                        &entity_list,
                                        &self.world,
                                    );
                                    for &e in &sorted {
                                        let tag = self
                                            .world
                                            .get::<crate::prefab::Tag>(e)
                                            .map(|t| t.0.clone());
                                        let transform = self
                                            .world
                                            .get::<crate::components::Transform>(e)
                                            .cloned();
                                        let sprite = self
                                            .world
                                            .get::<crate::components::Sprite>(e)
                                            .cloned();
                                        // Parent 컴포넌트 → 부모의 tag 문자열
                                        let parent = self
                                            .world
                                            .get::<crate::hierarchy::Parent>(e)
                                            .and_then(|p| tag_map.get(&p.0))
                                            .cloned();
                                        if tag.is_some()
                                            || transform.is_some()
                                            || sprite.is_some()
                                        {
                                            scene_def.entities.push(
                                                crate::prefab::EntityDef {
                                                    tag,
                                                    transform,
                                                    sprite,
                                                    parent,
                                                },
                                            );
                                        }
                                    }
                                    let count = scene_def.entities.len();
                                    let path = self.editor_save_path.clone();
                                    self.editor_save_status = match scene_def
                                        .save(std::path::Path::new(&path))
                                    {
                                        Ok(()) => {
                                            Some(format!("✓ {count} entities → {path}"))
                                        }
                                        Err(e) => Some(format!("✗ {e}")),
                                    };
                                }
                            });
                            if let Some(msg) = &self.editor_save_status {
                                ui.small(msg.as_str());
                            }
                            if let Some(msg) = &self.editor_load_status {
                                ui.small(msg.as_str());
                            }
                        }
                        } // end Entities tab
                    });
            }
        }

        // Inspector: 스테이징 값을 World에 적용 (egui 프레임 종료 전)
        if let Some(sel) = self.inspector_selected {
            let type_ids = self.world.reflected_components(sel);
            for (i, tid) in type_ids.iter().enumerate() {
                if i < comp_fields.len() {
                    if let Some(refl) = self.world.get_reflect_mut(sel, *tid) {
                        for (fname, fval) in &comp_fields[i].1 {
                            refl.set_field(fname, fval.clone());
                        }
                    }
                }
            }
        }

        // ── SelectedEntity 리소스 동기화 ─────────────────────────────────────────
        if let Some(res) = self
            .world
            .resource_mut::<crate::resources::SelectedEntity>()
        {
            res.0 = self.inspector_selected;
        }

        // ── Gizmo: 선택 엔티티 강조 + 드래그 이동 ────────────────────────────────
        let egui_wants_mouse = egui_ctx
            .as_ref()
            .map(|c| c.wants_pointer_input())
            .unwrap_or(false);

        if let Some(sel) = self.inspector_selected {
            // 선택된 엔티티의 Transform을 복사 (borrow 해방)
            let tr_copy = self.world.get::<crate::components::Transform>(sel).cloned();

            if let Some(tr) = tr_copy {
                // 선택 강조: DebugDrawQueue에 테두리 사각형 추가
                if let Some(dq) = self.world.resource_mut::<DebugDrawQueue>() {
                    let half = tr.scale * 0.5;
                    // 외곽 강조 (3px 두께 효과: 약간 확장)
                    let margin = glam::Vec2::splat(3.0 / tr.scale.x.max(1.0) * tr.scale.x);
                    dq.items.push(DebugRect {
                        min: tr.position - half - margin,
                        max: tr.position + half + margin,
                        color: [0.2, 0.85, 1.0, 0.65],
                        z: tr.z + 999.0,
                    });
                }

                // Gizmo 드래그 — egui가 마우스를 소비하지 않을 때만 동작
                if !egui_wants_mouse {
                    // 마우스 입력 + 카메라 좌표 변환 (짧은 borrow 블록)
                    let cam_default = crate::camera::Camera::default();
                    let gizmo_input = {
                        let cam = self
                            .world
                            .resource::<crate::camera::Camera>()
                            .unwrap_or(&cam_default);
                        self.world
                            .resource::<crate::input::InputState>()
                            .map(|inp| {
                                let world_pos = cam.screen_to_world(inp.cursor());
                                let pressed =
                                    inp.mouse_just_pressed(winit::event::MouseButton::Left);
                                let held = inp.is_mouse_pressed(winit::event::MouseButton::Left);
                                let released =
                                    inp.mouse_just_released(winit::event::MouseButton::Left);
                                (world_pos, pressed, held, released)
                            })
                    };

                    if let Some((world_pos, just_pressed, held, just_released)) = gizmo_input {
                        if just_pressed && !self.gizmo_dragging {
                            let half = tr.scale * 0.5;
                            let hit = world_pos.x >= tr.position.x - half.x
                                && world_pos.x <= tr.position.x + half.x
                                && world_pos.y >= tr.position.y - half.y
                                && world_pos.y <= tr.position.y + half.y;
                            if hit {
                                self.gizmo_dragging = true;
                                self.gizmo_drag_offset = tr.position - world_pos;
                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    self.gizmo_drag_start_pos = Some(tr.position);
                                }
                            }
                        }

                        if self.gizmo_dragging && held {
                            if let Some(t) = self.world.get_mut::<crate::components::Transform>(sel)
                            {
                                let new_pos = world_pos + self.gizmo_drag_offset;
                                #[cfg(not(target_arch = "wasm32"))]
                                let final_pos = if self.snap_enabled {
                                    snap_to_grid(new_pos, self.snap_size)
                                } else {
                                    new_pos
                                };
                                #[cfg(target_arch = "wasm32")]
                                let final_pos = new_pos;
                                t.position = final_pos;
                            }
                        }

                        if just_released {
                            #[cfg(not(target_arch = "wasm32"))]
                            if let Some(start_pos) = self.gizmo_drag_start_pos.take() {
                                let new_pos = self
                                    .world
                                    .get::<crate::components::Transform>(sel)
                                    .map(|t| t.position)
                                    .unwrap_or(start_pos);
                                if (new_pos - start_pos).length_squared() > 0.01 {
                                    self.cmd_history.push(EditorCmd::MoveEntity {
                                        entity: sel,
                                        old_pos: start_pos,
                                        new_pos,
                                    });
                                }
                            }
                            self.gizmo_dragging = false;
                        }
                    }
                } else {
                    self.gizmo_dragging = false;
                }
            }
        } else {
            self.gizmo_dragging = false;
        }

        // egui 프레임 종료 + tessellate → render() 로 전달
        if let Some(ctx) = egui_ctx {
            let ppp = self
                .window
                .as_ref()
                .map(|w| w.scale_factor() as f32)
                .unwrap_or(1.0);
            let full_output = ctx.end_pass();
            let paint_jobs = ctx.tessellate(full_output.shapes, ppp);
            self.egui_output = Some((paint_jobs, full_output.textures_delta, ppp));
        }
        // 모든 시스템 실행 후 이벤트 큐를 비운다.
        // std::mem::take 으로 꺼내야 &mut self.world 와 충돌하지 않는다.
        let flushers = std::mem::take(&mut self.event_flushers);
        for flush in &flushers {
            flush(&mut self.world);
        }
        self.event_flushers = flushers;
        if let Some(input) = self.world.resource_mut::<InputState>() {
            input.flush();
        }
        if let Some(gamepad) = self.world.resource_mut::<GamepadState>() {
            gamepad.flush();
        }
        // 씬 전환 명령 처리 (이벤트/입력 flush 이후)
        let cmd = self
            .world
            .resource_mut::<SceneChange>()
            .and_then(|sc| sc.0.take());
        if let Some(cmd) = cmd {
            self.apply_scene_cmd(cmd);
        }

        // FadeTransition 알파 진행
        if let Some(fade) = self
            .world
            .resource_mut::<crate::resources::FadeTransition>()
        {
            fade.update(dt);
        }

        // 핫 리로딩: 변경된 파일 목록을 받아 GPU 텍스처를 재업로드한다.
        let reloaded: Vec<String> = self
            .world
            .resource_mut::<AssetServer>()
            .map(|as_| as_.poll_reloads())
            .unwrap_or_default();
        if !reloaded.is_empty() {
            if let (Some(sr), Some(gpu)) = (&mut self.sprite_renderer, &self.gpu) {
                for path in &reloaded {
                    sr.reload_texture(&gpu.device, &gpu.queue, path);
                }
            }
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let gpu = match self.gpu.as_mut() {
            Some(g) => g,
            None => return Ok(()),
        };

        // PostProcessConfig 리소스 확인 (enabled=true일 때만 중간 텍스처 사용)
        let pp_config: Option<PostProcessConfig> =
            self.world.resource::<PostProcessConfig>().copied();
        let use_post = pp_config.map(|c| c.enabled).unwrap_or(false);

        // 포스트프로세스 렌더러 초기화 / 리사이즈
        if use_post {
            let (w, h, fmt) = (gpu.config.width, gpu.config.height, gpu.config.format);
            match &mut self.post_renderer {
                None => {
                    self.post_renderer = Some(PostProcessRenderer::new(&gpu.device, w, h, fmt));
                }
                Some(pr) if pr.width != w || pr.height != h => {
                    pr.resize(&gpu.device, w, h);
                }
                _ => {}
            }
        }

        // 라이팅 렌더러 초기화 / 리사이즈 / 비활성화
        #[cfg(not(target_arch = "wasm32"))]
        let use_lighting = {
            let has_lighting = self
                .world
                .resource::<crate::resources::AmbientLight>()
                .is_some();
            let (w, h, fmt) = (gpu.config.width, gpu.config.height, gpu.config.format);
            if has_lighting {
                match &mut self.lighting_renderer {
                    None => {
                        self.lighting_renderer =
                            Some(crate::renderer::lighting::LightingRenderer::new(
                                &gpu.device,
                                w,
                                h,
                                fmt,
                            ));
                    }
                    Some(lr) if lr.width != w || lr.height != h => {
                        lr.resize(&gpu.device, w, h);
                    }
                    _ => {}
                }
                // 씬 중간 텍스처 생성 / 리사이즈 (post_renderer가 없을 때만 필요)
                if !use_post {
                    let needs_new = match &self.scene_texture_for_lighting {
                        None => true,
                        Some(_) => {
                            // 크기 재확인을 위해 뷰 크기를 직접 비교할 수 없으므로
                            // lighting_renderer 크기가 바뀌면 텍스처도 재생성한다.
                            // lighting_renderer resize가 이미 처리했으므로, 텍스처와 lr 크기 비교는 불필요.
                            // 여기선 None인 경우만 처리.
                            false
                        }
                    };
                    if needs_new {
                        let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
                            label: Some("scene_for_lighting"),
                            size: wgpu::Extent3d {
                                width: w,
                                height: h,
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: fmt,
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                                | wgpu::TextureUsages::TEXTURE_BINDING,
                            view_formats: &[],
                        });
                        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
                        self.scene_texture_for_lighting = Some((tex, view));
                    }
                }
            } else {
                self.lighting_renderer = None;
                self.scene_texture_for_lighting = None;
            }
            has_lighting
        };
        #[cfg(target_arch = "wasm32")]
        let use_lighting = false;

        let frame = gpu.surface.get_current_texture()?;
        let final_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame encoder"),
            });

        // 렌더 타겟 선택:
        //   라이팅 있고 포스트 없음 → 중간 씬 텍스처
        //   포스트 있음 (라이팅 여부 무관) → post_renderer.target_view
        //   둘 다 없음 → 스왑체인 직접
        let render_view: &wgpu::TextureView = if use_lighting && !use_post {
            #[cfg(not(target_arch = "wasm32"))]
            {
                &self.scene_texture_for_lighting.as_ref().unwrap().1
            }
            #[cfg(target_arch = "wasm32")]
            {
                &final_view
            }
        } else if use_post {
            &self.post_renderer.as_ref().unwrap().target_view
        } else {
            &final_view
        };

        // 1단계: 배경 Clear
        let [cr, cg, cb, ca] = self
            .world
            .resource::<WindowConfig>()
            .map(|c| c.clear_color)
            .unwrap_or([0.08, 0.08, 0.12, 1.0]);
        {
            let _pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: render_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: cr,
                            g: cg,
                            b: cb,
                            a: ca,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        let viewport = self
            .world
            .resource::<ViewportSize>()
            .copied()
            .unwrap_or_else(|| ViewportSize::new(gpu.config.width, gpu.config.height));
        let logical_w = viewport.width.round().max(1.0) as u32;
        let logical_h = viewport.height.round().max(1.0) as u32;

        // 2단계: 스프라이트 그리기
        if let Some(sr) = &mut self.sprite_renderer {
            let render_stats = sr.render(
                &gpu.device,
                &gpu.queue,
                render_view,
                &mut enc,
                &self.world,
                logical_w,
                logical_h,
            );
            if let Some(prof) = self.world.resource_mut::<crate::resources::ProfilerData>() {
                prof.render = render_stats;
            }
        }

        // 2.5단계: UI 사각형 그리기 (DebugDrawQueue → UiQueue 변환)
        let debug_rects: Vec<DrawRect> = self
            .world
            .resource_mut::<DebugDrawQueue>()
            .map(|q| {
                std::mem::take(&mut q.items)
                    .into_iter()
                    .map(|r| {
                        DrawRect::new(
                            r.min.x,
                            r.min.y,
                            r.max.x - r.min.x,
                            r.max.y - r.min.y,
                            r.color,
                        )
                        .with_z(r.z)
                    })
                    .collect()
            })
            .unwrap_or_default();
        if let Some(q) = self.world.resource_mut::<UiQueue>() {
            q.items.extend(debug_rects);
        }

        // 2.6단계: DebugDraw 도형 → UiQueue 변환 (Rect/Line/Circle/Cross)
        let debug_shapes: Vec<crate::resources::DebugShape> = self
            .world
            .resource_mut::<DebugDraw>()
            .map(|d| std::mem::take(&mut d.shapes))
            .unwrap_or_default();
        if !debug_shapes.is_empty() {
            if let Some(q) = self.world.resource_mut::<UiQueue>() {
                for shape in debug_shapes {
                    Self::debug_shape_to_draw_rects(shape, q);
                }
            }
        }

        let ui_rects: Vec<DrawRect> = self
            .world
            .resource_mut::<UiQueue>()
            .map(|q| std::mem::take(&mut q.items))
            .unwrap_or_default();
        if !ui_rects.is_empty() {
            if let Some(sr) = &mut self.sprite_renderer {
                sr.render_ui_rects_from_slice(
                    &gpu.device,
                    &gpu.queue,
                    render_view,
                    &mut enc,
                    &ui_rects,
                    logical_w,
                    logical_h,
                );
            }
        }

        // 3단계: 텍스트 그리기
        let (w, h) = (gpu.config.width, gpu.config.height);
        if let Some(tr) = &mut self.text_renderer {
            tr.render(
                &gpu.device,
                &gpu.queue,
                &mut enc,
                render_view,
                &mut self.world,
                w,
                h,
            );
        }

        // 4단계: 포스트프로세스 패스 (중간 텍스처 → 스왑체인 또는 라이팅 중간 텍스처)
        if use_post {
            #[cfg(not(target_arch = "wasm32"))]
            let post_output: &wgpu::TextureView = if use_lighting {
                // 라이팅도 활성이면 post 출력을 씬 중간 텍스처로 보낸다.
                // 씬 중간 텍스처는 post_renderer 없을 때만 생성되므로 여기선 별도 버퍼 필요.
                // 단순화: post → final_view, 그 다음 lighting 패스는 final_view를 읽을 수 없다.
                // 따라서 post+lighting 조합에서는 final_view를 post 출력으로 사용하고
                // lighting을 그 위에 다시 적용하는 대신, post 출력에 lighting을 적용한다.
                // 구현: post → final_view 후 lighting 패스를 final_view→output(=final_view)로 수행.
                // (이 케이스에서는 lighting input = post target_view, output = final_view)
                &final_view
            } else {
                &final_view
            };
            #[cfg(target_arch = "wasm32")]
            let post_output: &wgpu::TextureView = &final_view;

            if let (Some(pr), Some(cfg)) = (&self.post_renderer, pp_config.as_ref()) {
                pr.update_uniforms(&gpu.queue, cfg);
                pr.run_pass(&mut enc, post_output);
            }
        }

        // 4.5단계: 라이팅 패스
        #[cfg(not(target_arch = "wasm32"))]
        if use_lighting {
            if let Some(lr) = &self.lighting_renderer {
                lr.update(&gpu.queue, &self.world, gpu.config.width, gpu.config.height);

                // 노멀 버퍼를 평면 노멀(0.5, 0.5, 1.0)으로 초기화한다.
                // (스프라이트별 노멀 맵 렌더링은 향후 여기서 수행)
                lr.clear_normal_buffer(&mut enc);

                // scene input: post가 있으면 post.target_view, 없으면 씬 중간 텍스처
                let scene_input: &wgpu::TextureView = if use_post {
                    &self.post_renderer.as_ref().unwrap().target_view
                } else {
                    &self.scene_texture_for_lighting.as_ref().unwrap().1
                };
                lr.run_pass(&gpu.device, &mut enc, scene_input, &final_view);
            }
        }

        // 5단계 (pre): 페이드 오버레이 패스 (다른 모든 패스 이후 최상위)
        #[cfg(not(target_arch = "wasm32"))]
        {
            // 필요 시 lazy init
            if self.fade_renderer.is_none() {
                self.fade_renderer = Some(crate::renderer::fade::FadeRenderer::new(
                    &gpu.device,
                    gpu.config.format,
                ));
            }
            if let (Some(fr), Some(fade)) = (
                &self.fade_renderer,
                self.world.resource::<crate::resources::FadeTransition>(),
            ) {
                if fade.alpha > 0.001 {
                    fr.update(&gpu.queue, fade.color, fade.alpha);
                    fr.run_pass(&mut enc, &final_view);
                }
            }
        }

        // 씬+포스트프로세스+라이팅+페이드 완료 후 제출
        gpu.queue.submit(std::iter::once(enc.finish()));

        // 5단계: egui 오버레이 패스
        if let (Some(mut er), Some((paint_jobs, textures_delta, ppp))) =
            (self.egui_renderer.take(), self.egui_output.take())
        {
            let screen_desc = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [gpu.config.width, gpu.config.height],
                pixels_per_point: ppp,
            };
            for (id, delta) in &textures_delta.set {
                er.update_texture(&gpu.device, &gpu.queue, *id, delta);
            }
            let mut egui_enc = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("egui encoder"),
                });
            er.update_buffers(
                &gpu.device,
                &gpu.queue,
                &mut egui_enc,
                &paint_jobs,
                &screen_desc,
            );
            // Renderer::render<'rp>(&'rp self, &mut RenderPass<'rp>) 의 lifetime 제약 때문에
            // 독립 함수에서 &er 와 &mut egui_enc 를 동일 lifetime 'a 로 묶는다.
            egui_render_pass(&er, &mut egui_enc, &paint_jobs, &screen_desc, &final_view);
            gpu.queue.submit(std::iter::once(egui_enc.finish()));
            for id in &textures_delta.free {
                er.free_texture(id);
            }
            self.egui_renderer = Some(er);
        }

        frame.present();
        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

// ─── winit ApplicationHandler 구현 ───────────────────────────────────────────

impl ApplicationHandler for App {
    /// 앱이 활성화될 때 호출 (macOS: Resumed, 기타: 시작 시 1회)
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (init_w, init_h, title) = self
            .world
            .resource::<WindowConfig>()
            .map(|c| (c.width, c.height, c.title.clone()))
            .unwrap_or((1280, 720, "Game".to_string()));
        let attrs = Window::default_attributes()
            .with_title(&title)
            .with_inner_size(winit::dpi::LogicalSize::new(init_w, init_h));

        // WASM: HTML 내 <canvas id="game-canvas"> 를 winit 창에 연결한다.
        #[cfg(target_arch = "wasm32")]
        let attrs = {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;
            if let Some(canvas) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id("game-canvas"))
                .and_then(|el| el.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            {
                attrs.with_canvas(Some(canvas))
            } else {
                attrs
            }
        };

        let window = Arc::new(event_loop.create_window(attrs).expect("창 생성 실패"));

        #[cfg(not(target_arch = "wasm32"))]
        {
            let gpu = pollster::block_on(GpuContext::new(window.clone()));
            self.finish_init(gpu, window);
        }

        // WASM: WebGPU/WebGL2 adapter 요청이 Promise 기반이므로 spawn_local로 비동기 처리한다.
        // GPU 준비 완료 시 PENDING_GPU thread_local에 저장 → about_to_wait()에서 finish_init 호출.
        #[cfg(target_arch = "wasm32")]
        {
            self.window = Some(window.clone());
            wasm_bindgen_futures::spawn_local(async move {
                let gpu = GpuContext::new(window.clone()).await;
                PENDING_GPU.with(|p| {
                    *p.borrow_mut() = Some((gpu, window));
                });
            });
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // egui에 이벤트 선전달
        if let (Some(state), Some(window)) = (&mut self.egui_state, &self.window) {
            let _ = state.on_window_event(window, &event);
        }

        match event {
            // ── 창 닫기 ──────────────────────────────────────────────────────
            WindowEvent::CloseRequested => event_loop.exit(),

            // ── 창 크기 변경 ─────────────────────────────────────────────────
            WindowEvent::Resized(size) => {
                if let Some(gpu) = &mut self.gpu {
                    // WASM: Retina DPR 때문에 winit이 CSS 픽셀 × DPR(= 2560×1440)을 보고한다.
                    // WebGL2의 최대 텍스처 크기(2048)를 초과하므로, DOM에서 canvas 크기를 직접 읽는다.
                    #[cfg(target_arch = "wasm32")]
                    let size = {
                        use wasm_bindgen::JsCast;
                        web_sys::window()
                            .and_then(|w| w.document())
                            .and_then(|d| d.get_element_by_id("game-canvas"))
                            .and_then(|el| el.dyn_into::<web_sys::HtmlCanvasElement>().ok())
                            .map(|c| {
                                winit::dpi::PhysicalSize::new(c.width().max(1), c.height().max(1))
                            })
                            .unwrap_or(size)
                    };
                    gpu.resize(size);
                }
            }

            // ── 키보드 입력 ──────────────────────────────────────────────────
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        logical_key,
                        state,
                        ..
                    },
                ..
            } => {
                // F1 → DebugUi 토글
                if key == winit::keyboard::KeyCode::F1 && state == ElementState::Pressed {
                    if let Some(debug_ui) = self.world.resource_mut::<DebugUi>() {
                        debug_ui.toggle();
                    }
                }
                if let Some(input) = self.world.resource_mut::<InputState>() {
                    match state {
                        ElementState::Pressed => {
                            input.press(key);
                            use winit::keyboard::{Key, NamedKey};
                            match &logical_key {
                                Key::Character(s) => {
                                    for c in s.chars() {
                                        input.push_char(c);
                                    }
                                }
                                Key::Named(NamedKey::Backspace) => input.push_backspace(),
                                Key::Named(NamedKey::Enter) => input.push_enter(),
                                _ => {}
                            }
                        }
                        ElementState::Released => input.release(key),
                    }
                }
            }

            WindowEvent::Ime(winit::event::Ime::Preedit(text, _cursor)) => {
                if let Some(input) = self.world.resource_mut::<InputState>() {
                    input.set_ime_preedit(text);
                }
            }

            WindowEvent::Ime(winit::event::Ime::Commit(text)) => {
                if let Some(input) = self.world.resource_mut::<InputState>() {
                    input.push_text(&text);
                    input.clear_ime_preedit();
                }
            }

            // ── 마우스 커서 이동 ─────────────────────────────────────────────
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(input) = self.world.resource_mut::<InputState>() {
                    input.set_cursor(Vec2::new(position.x as f32, position.y as f32));
                }
            }

            // ── 마우스 버튼 ──────────────────────────────────────────────────
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(input) = self.world.resource_mut::<InputState>() {
                    match state {
                        ElementState::Pressed => input.press_mouse(button),
                        ElementState::Released => input.release_mouse(button),
                    }
                }
            }

            // ── 마우스 휠 ────────────────────────────────────────────────────
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(input) = self.world.resource_mut::<InputState>() {
                    match delta {
                        MouseScrollDelta::LineDelta(_, y) => input.add_scroll(y),
                        // 픽셀 단위 휠(트랙패드 등)을 line 단위로 환산: 20px ≈ 1 line (경험적 근사값)
                        MouseScrollDelta::PixelDelta(p) => input.add_scroll(p.y as f32 / 20.0),
                    }
                }
            }

            // ── 프레임 렌더 ──────────────────────────────────────────────────
            WindowEvent::RedrawRequested => {
                // WASM: about_to_wait 타이밍에 GPU가 준비되지 않은 경우를 대비해 여기서도 체크
                #[cfg(target_arch = "wasm32")]
                if self.gpu.is_none() {
                    if let Some((gpu, window)) = PENDING_GPU.with(|p| p.borrow_mut().take()) {
                        self.finish_init(gpu, window);
                    }
                }

                let now = Instant::now();
                let dt = self
                    .last_frame
                    .map(|t| (now - t).as_secs_f32().min(0.1))
                    .unwrap_or(1.0 / 60.0);
                self.last_frame = Some(now);

                self.update(dt);

                // 시스템이 ShouldQuit(true) 를 설정했으면 종료
                if self
                    .world
                    .resource::<ShouldQuit>()
                    .map(|q| q.0)
                    .unwrap_or(false)
                {
                    event_loop.exit();
                    return;
                }

                // PendingResize: 게임이 요청한 해상도로 창 크기 변경
                let pending = self.world.resource::<PendingResize>().and_then(|r| r.0);
                if let Some((w, h)) = pending {
                    if let Some(window) = &self.window {
                        let _ = window.request_inner_size(winit::dpi::LogicalSize::new(w, h));
                    }
                    if let Some(r) = self.world.resource_mut::<PendingResize>() {
                        *r = PendingResize(None);
                    }
                }

                match self.render() {
                    Ok(()) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        if let Some(gpu) = &self.gpu {
                            gpu.reconfigure();
                        }
                    }
                    Err(e) => log::error!("렌더링 오류: {e:?}"),
                }
            }

            _ => {}
        }
    }

    /// 이벤트 큐가 비었을 때 → 게임패드 폴링 후 매 프레임 redraw 요청
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        #[cfg(not(target_arch = "wasm32"))]
        self.poll_gilrs();

        // WASM: spawn_local로 시작한 GPU 비동기 초기화 완료를 여기서 감지한다.
        #[cfg(target_arch = "wasm32")]
        if self.gpu.is_none() {
            if let Some((gpu, window)) = PENDING_GPU.with(|p| p.borrow_mut().take()) {
                self.finish_init(gpu, window);
            }
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl App {
    /// GPU 컨텍스트와 창이 준비된 후 렌더러·egui를 초기화한다.
    /// 네이티브: resumed()에서 직접 호출. WASM: about_to_wait()에서 PENDING_GPU 확인 후 호출.
    fn finish_init(&mut self, gpu: GpuContext, window: Arc<Window>) {
        let mut sprite_renderer = SpriteRenderer::new(&gpu.device, &gpu.queue, gpu.config.format);
        for path in self.pending_textures.drain(..) {
            sprite_renderer.load_texture(&gpu.device, &gpu.queue, &path);
        }
        let font_bytes = self
            .world
            .resource::<FontData>()
            .map(|f| f.0.clone())
            .unwrap_or_default();
        // WASM: 시스템 폰트가 없으므로 font_bytes가 비어있으면 텍스트 렌더러를 생성하지 않는다.
        // cosmic-text는 폰트 없이 shape를 시도할 때 패닉한다.
        #[cfg(not(target_arch = "wasm32"))]
        let text_renderer = Some(TextRenderer::new(
            &gpu.device,
            &gpu.queue,
            gpu.config.format,
            &font_bytes,
        ));
        #[cfg(target_arch = "wasm32")]
        let text_renderer = if !font_bytes.is_empty() {
            Some(TextRenderer::new(
                &gpu.device,
                &gpu.queue,
                gpu.config.format,
                &font_bytes,
            ))
        } else {
            None
        };
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &*window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let egui_renderer =
            egui_wgpu::Renderer::new(&gpu.device, gpu.config.format, None, 1, false);
        self.world.insert_resource(DebugUi::new_with_ctx(egui_ctx));
        self.egui_renderer = Some(egui_renderer);
        self.egui_state = Some(egui_state);
        self.sprite_renderer = Some(sprite_renderer);
        self.text_renderer = text_renderer;
        self.gpu = Some(gpu);
        self.window = Some(window);
        self.last_frame = Some(Instant::now());
        log::info!("엔진 초기화 완료");
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn poll_gilrs(&mut self) {
        let mut events = Vec::new();
        if let Some(gilrs) = &mut self.gilrs {
            while let Some(event) = gilrs.next_event() {
                events.push(event);
            }
        }
        if events.is_empty() {
            return;
        }
        if let Some(state) = self.world.resource_mut::<GamepadState>() {
            for event in events {
                state.process_event(event);
            }
        }
    }
}

// ── egui 렌더 헬퍼 ───────────────────────────────────────────────────────────
// egui-wgpu 0.29 의 PaintCallbackFn 이 &mut RenderPass<'static> 을 요구하기 때문에
// render<'rp>(&'rp self, &mut RenderPass<'rp>) 가 'rp: 'static 을 강제한다.
//
// SAFETY: paint callback 을 등록하지 않으므로 'static transmute 는 실제로 안전하다.
// rpass 를 transmute 로 소비(move)하면 NLL borrow checker 가 enc 의 borrow 를 해제하고,
// 새 RenderPass<'static> 은 enc 와 독립적 lifetime 으로 추론되어 enc.finish() 가 가능해진다.
fn egui_render_pass(
    er: &egui_wgpu::Renderer,
    enc: &mut wgpu::CommandEncoder,
    paint_jobs: &[egui::ClippedPrimitive],
    screen_desc: &egui_wgpu::ScreenDescriptor,
    view: &wgpu::TextureView,
) {
    let rpass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("egui"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
    });
    // SAFETY: rpass 는 이 함수 밖으로 탈출하지 않는다. transmute 로 'encoder borrow 를
    // 'static 으로 변환해 NLL 이 enc borrow 를 해제하도록 한다. er 도 동일하게 처리.
    unsafe {
        let er_s: &'static egui_wgpu::Renderer = &*(er as *const _);
        let mut rpass_s: wgpu::RenderPass<'static> = std::mem::transmute(rpass);
        er_s.render(&mut rpass_s, paint_jobs, screen_desc);
    }
}
