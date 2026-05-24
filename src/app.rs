use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

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
    renderer::{DrawRect, GpuContext, PostProcessConfig, PostProcessRenderer, SpriteRenderer, TextQueue, TextRenderer, UiQueue},
    resources::{DebugDrawQueue, FontData, GameState, PendingResize, ShouldQuit, ViewportSize, WindowConfig},
    scene::{Scene, SceneChange, SceneCmd},
};

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
    /// (씬, 해당 씬이 등록한 시스템 수). Push/Pop 시 시스템 복원에 사용.
    scene_stack: Vec<(Box<dyn Scene>, usize)>,
    window: Option<Arc<Window>>,
    gpu: Option<GpuContext>,
    sprite_renderer: Option<SpriteRenderer>,
    /// 스프라이트 pass 직후 텍스트를 덮어쓴다. GPU 초기화 이후 Some으로 채워진다.
    text_renderer: Option<TextRenderer>,
    /// PostProcessConfig 리소스가 enabled=true일 때 활성화된다.
    post_renderer: Option<PostProcessRenderer>,
    last_frame: Option<Instant>,
    /// GPU 초기화 전에 등록된 텍스처 경로를 보관한다. resumed()에서 실제로 로드한다.
    pending_textures: Vec<String>,
    /// 매 프레임 종료 시 이벤트 큐를 비우는 클로저 목록.
    event_flushers: Vec<Box<dyn Fn(&mut World)>>,
    /// reload_scene 시 이벤트 리소스를 재삽입하는 클로저 목록.
    event_initializers: Vec<Box<dyn Fn(&mut World)>>,
    /// gilrs 게임패드 컨텍스트. 초기화 실패 시 None (게임패드 없이 동작).
    gilrs: Option<gilrs::Gilrs>,
    /// egui 렌더러 (wgpu 백엔드).
    egui_renderer: Option<egui_wgpu::Renderer>,
    /// winit ↔ egui 이벤트 변환기.
    egui_state: Option<egui_winit::State>,
    /// update() 에서 tessellate 한 결과를 render() 까지 전달하는 임시 버퍼.
    egui_output: Option<(Vec<egui::ClippedPrimitive>, egui::TexturesDelta, f32)>,
    /// Inspector 패널에서 현재 선택된 엔티티.
    inspector_selected: Option<Entity>,
}

impl App {
    pub fn new() -> Self {
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
        world.insert_resource(SceneChange::default());
        world.insert_resource(AssetServer::new());
        // 엔진 내장 컴포넌트를 Reflect 레지스트리에 자동 등록
        world.register_reflect::<crate::components::Transform>();
        world.register_reflect::<crate::components::Sprite>();
        world.register_reflect::<crate::prefab::Tag>();

        Self {
            world,
            systems: Vec::new(),
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
            gilrs,
            egui_renderer: None,
            egui_state: None,
            egui_output: None,
            inspector_selected: None,
        }
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
    pub fn load_script(&mut self, path: impl AsRef<std::path::Path>) -> Handle<crate::asset::ScriptAsset> {
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
        self.world.insert_resource(SceneChange::default());
        self.world.insert_resource(AssetServer::new());
        // 등록된 이벤트 리소스 재삽입
        let inits = std::mem::take(&mut self.event_initializers);
        for init in &inits {
            init(&mut self.world);
        }
        self.event_initializers = inits;
        // Reflect 레지스트리 재등록 (World 재생성으로 초기화되었으므로)
        self.world.register_reflect::<crate::components::Transform>();
        self.world.register_reflect::<crate::components::Sprite>();
        self.world.register_reflect::<crate::prefab::Tag>();
        self.inspector_selected = None;
    }

    /// 씬을 즉시 전환한다. `run()` 호출 전·후 모두 사용 가능하다.
    ///
    /// 현재 씬 스택을 전부 종료하고 월드를 리셋한 뒤 `scene`을 진입시킨다.
    pub fn set_scene(&mut self, scene: Box<dyn Scene>) {
        self.apply_scene_cmd(SceneCmd::Replace(scene));
    }

    // ── 씬 전환 처리 ─────────────────────────────────────────────────────────

    fn apply_scene_cmd(&mut self, cmd: SceneCmd) {
        match cmd {
            SceneCmd::Replace(mut new_scene) => {
                for (mut scene, _) in self.scene_stack.drain(..).rev() {
                    scene.on_exit(&mut self.world);
                }
                self.systems.clear();
                self.reload_scene();
                let before = self.systems.len();
                new_scene.on_enter(&mut self.world, &mut self.systems);
                let owned = self.systems.len() - before;
                self.scene_stack.push((new_scene, owned));
            }
            SceneCmd::Push(mut new_scene) => {
                let before = self.systems.len();
                new_scene.on_enter(&mut self.world, &mut self.systems);
                let owned = self.systems.len() - before;
                self.scene_stack.push((new_scene, owned));
            }
            SceneCmd::Pop => {
                if let Some((mut scene, owned)) = self.scene_stack.pop() {
                    scene.on_exit(&mut self.world);
                    let new_len = self.systems.len().saturating_sub(owned);
                    self.systems.truncate(new_len);
                }
            }
        }
    }

    /// 이벤트 루프를 시작한다. 창이 닫힐 때까지 블로킹된다.
    pub fn run(mut self) {
        let event_loop = EventLoop::new().expect("이벤트 루프 생성 실패");
        event_loop.run_app(&mut self).expect("이벤트 루프 오류");
    }

    // ── 내부 메서드 ─────────────────────────────────────────────────────────

    fn update(&mut self, dt: f32) {
        // GPU 실제 크기를 ViewportSize 리소스에 동기화 (매 프레임)
        if let Some(gpu) = &self.gpu {
            self.world
                .insert_resource(ViewportSize::new(gpu.config.width, gpu.config.height));
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

        for system in &mut self.systems {
            system.run(&mut self.world, dt);
        }
        // 계층 변환 전파 — 유저 시스템(물리 포함) 이후, 렌더 직전에 실행
        HierarchySystem.run(&mut self.world, dt);

        // Inspector: 선택된 엔티티 유효성 확인 + 필드 스테이징
        if let Some(sel) = self.inspector_selected {
            if !self.world.is_alive(sel) {
                self.inspector_selected = None;
            }
        }
        let entity_list: Vec<Entity> = self.world.entities().to_vec();
        let tag_map: HashMap<Entity, String> = self.world.query::<Tag>()
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

        // 내장 EngineStats 패널 + Inspector
        if let Some(ctx) = &egui_ctx {
            if self.world.resource::<DebugUi>().map(|d| d.is_enabled()).unwrap_or(false) {
                let entity_count = self.world.entity_count();
                let asset_count = self
                    .world
                    .resource::<AssetServer>()
                    .map(|a| a.image_count())
                    .unwrap_or(0);
                egui::Window::new("Engine Stats")
                    .default_pos([10.0, 10.0])
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(format!("FPS   {:>6.1}", 1.0_f32 / dt.max(0.001)));
                        ui.label(format!("ms    {:>6.2}", dt * 1000.0));
                        ui.label(format!("Ent   {entity_count}"));
                        ui.label(format!("Asset {asset_count}"));
                    });

                // Inspector 패널: 엔티티 목록 + 컴포넌트 필드 편집기
                egui::Window::new("Inspector")
                    .default_pos([10.0, 130.0])
                    .default_size([440.0, 300.0])
                    .show(ctx, |ui| {
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
                                            let label = tag_map.get(&e)
                                                .cloned()
                                                .unwrap_or_else(|| format!("E{}", e.0));
                                            if ui.selectable_label(
                                                self.inspector_selected == Some(e),
                                                &label,
                                            ).clicked() {
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
                                                                    ui.add(egui::DragValue::new(v).speed(0.5));
                                                                }
                                                                ReflectValue::Vec2(v) => {
                                                                    ui.horizontal(|ui| {
                                                                        ui.add(egui::DragValue::new(&mut v.x).speed(0.5).prefix("x:"));
                                                                        ui.add(egui::DragValue::new(&mut v.y).speed(0.5).prefix("y:"));
                                                                    });
                                                                }
                                                                ReflectValue::Bool(v) => { ui.checkbox(v, ""); }
                                                                ReflectValue::String(s) => { ui.text_edit_singleline(s); }
                                                                ReflectValue::Color(c) => {
                                                                    ui.horizontal(|ui| {
                                                                        ui.add(egui::DragValue::new(&mut c[0]).speed(0.01).prefix("r:"));
                                                                        ui.add(egui::DragValue::new(&mut c[1]).speed(0.01).prefix("g:"));
                                                                        ui.add(egui::DragValue::new(&mut c[2]).speed(0.01).prefix("b:"));
                                                                        ui.add(egui::DragValue::new(&mut c[3]).speed(0.01).prefix("a:"));
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

        // egui 프레임 종료 + tessellate → render() 로 전달
        if let Some(ctx) = egui_ctx {
            let ppp = self.window.as_ref().map(|w| w.scale_factor() as f32).unwrap_or(1.0);
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

        let frame = gpu.surface.get_current_texture()?;
        let final_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame encoder"),
            });

        // 렌더 타겟: 포스트프로세싱 사용 시 중간 텍스처, 아니면 스왑체인 직접
        // post_renderer와 gpu/sprite_renderer/text_renderer는 서로 다른 필드이므로 동시 빌림 허용.
        let render_view: &wgpu::TextureView = if use_post {
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
                            r: cr, g: cg, b: cb, a: ca,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        // 2단계: 스프라이트 그리기
        if let Some(sr) = &mut self.sprite_renderer {
            sr.render(
                &gpu.device,
                &gpu.queue,
                render_view,
                &mut enc,
                &self.world,
                gpu.config.width,
                gpu.config.height,
            );
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
                            r.min.x, r.min.y,
                            r.max.x - r.min.x, r.max.y - r.min.y,
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
                    gpu.config.width,
                    gpu.config.height,
                );
            }
        }

        // 3단계: 텍스트 그리기
        let (w, h) = (gpu.config.width, gpu.config.height);
        if let Some(tr) = &mut self.text_renderer {
            tr.render(&gpu.device, &gpu.queue, &mut enc, render_view, &mut self.world, w, h);
        }

        // 4단계: 포스트프로세스 패스 (중간 텍스처 → 스왑체인)
        if use_post {
            if let (Some(pr), Some(cfg)) = (&self.post_renderer, pp_config.as_ref()) {
                pr.update_uniforms(&gpu.queue, cfg);
                pr.run_pass(&mut enc, &final_view);
            }
        }

        // 씬+포스트프로세스 완료 후 제출
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
            let mut egui_enc =
                gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
        let window = Arc::new(event_loop.create_window(attrs).expect("창 생성 실패"));

        let gpu = pollster::block_on(GpuContext::new(window.clone()));
        let mut sprite_renderer = SpriteRenderer::new(&gpu.device, &gpu.queue, gpu.config.format);

        // 대기열에 있던 텍스처를 GPU에 일괄 로드한다.
        for path in self.pending_textures.drain(..) {
            sprite_renderer.load_texture(&gpu.device, &gpu.queue, &path);
        }

        // 폰트 데이터 — FontData 리소스에서 읽음 (없으면 빈 슬라이스 → 시스템 폰트 폴백)
        let font_bytes = self
            .world
            .resource::<FontData>()
            .map(|f| f.0.clone())
            .unwrap_or_default();

        // 텍스트 렌더러: 스프라이트와 동일한 surface format 을 사용한다.
        let text_renderer =
            TextRenderer::new(&gpu.device, &gpu.queue, gpu.config.format, &font_bytes);

        // egui 초기화 (gpu 이동 전에 device/format 접근)
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
        self.text_renderer = Some(text_renderer);
        self.gpu = Some(gpu);
        self.window = Some(window);
        self.last_frame = Some(Instant::now());

        log::info!("엔진 초기화 완료");
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
        self.poll_gilrs();
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl App {
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
