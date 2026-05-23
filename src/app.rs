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
    camera::Camera,
    ecs::{Events, System, World},
    input::InputState,
    renderer::{DrawRect, GpuContext, SpriteRenderer, TextQueue, TextRenderer, UiQueue},
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
    last_frame: Option<Instant>,
    /// GPU 초기화 전에 등록된 텍스처 경로를 보관한다. resumed()에서 실제로 로드한다.
    pending_textures: Vec<String>,
    /// 매 프레임 종료 시 이벤트 큐를 비우는 클로저 목록.
    event_flushers: Vec<Box<dyn Fn(&mut World)>>,
    /// reload_scene 시 이벤트 리소스를 재삽입하는 클로저 목록.
    event_initializers: Vec<Box<dyn Fn(&mut World)>>,
}

impl App {
    pub fn new() -> Self {
        let mut world = World::new();
        world.insert_resource(InputState::default());
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
        Self {
            world,
            systems: Vec::new(),
            scene_stack: Vec::new(),
            window: None,
            gpu: None,
            sprite_renderer: None,
            text_renderer: None,
            last_frame: None,
            pending_textures: Vec::new(),
            event_flushers: Vec::new(),
            event_initializers: Vec::new(),
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

    /// ECS 월드를 초기화하고 기본 리소스를 재삽입한다.
    ///
    /// 씬 전환 시 엔티티·컴포넌트를 전부 지우고 싶을 때 사용한다.
    /// 시스템은 유지되므로 필요하면 `add_system`으로 새로 등록한다.
    pub fn reload_scene(&mut self) {
        self.world = World::new();
        self.world.insert_resource(InputState::default());
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
        // 등록된 이벤트 리소스 재삽입
        let inits = std::mem::take(&mut self.event_initializers);
        for init in &inits {
            init(&mut self.world);
        }
        self.event_initializers = inits;
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
        for system in &mut self.systems {
            system.run(&mut self.world, dt);
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
        // 씬 전환 명령 처리 (이벤트/입력 flush 이후)
        let cmd = self
            .world
            .resource_mut::<SceneChange>()
            .and_then(|sc| sc.0.take());
        if let Some(cmd) = cmd {
            self.apply_scene_cmd(cmd);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let gpu = match self.gpu.as_mut() {
            Some(g) => g,
            None => return Ok(()),
        };

        let frame = gpu.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame encoder"),
            });

        // 1단계: 배경 Clear (색상은 WindowConfig 리소스에서 읽음)
        let [cr, cg, cb, ca] = self
            .world
            .resource::<WindowConfig>()
            .map(|c| c.clear_color)
            .unwrap_or([0.08, 0.08, 0.12, 1.0]);
        {
            let _pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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

        // 2단계: 스프라이트 그리기
        if let Some(sr) = &mut self.sprite_renderer {
            sr.render(
                &gpu.device,
                &gpu.queue,
                &view,
                &mut enc,
                &self.world,
                gpu.config.width,
                gpu.config.height,
            );
        }

        // 2.5단계: UI 사각형 그리기 (스프라이트 위, 텍스트 아래)
        // DebugDrawQueue → UiQueue 로 변환 (레이어 경계: 순수 데이터 → 렌더러 타입)
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

        // UiQueue items를 먼저 drain해 로컬 Vec에 보관 → sprite_renderer 가변 빌림과 충돌 방지
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
                    &view,
                    &mut enc,
                    &ui_rects,
                    gpu.config.width,
                    gpu.config.height,
                );
            }
        }

        // 3단계: 텍스트 그리기 (스프라이트 위에 LoadOp::Load 로 합성)
        // gpu(self.gpu 필드 빌림)와 self.text_renderer는 서로 다른 필드이므로
        // NLL(Non-Lexical Lifetimes) 하에서 동시 빌림이 허용된다.
        let (w, h) = (gpu.config.width, gpu.config.height);
        if let Some(tr) = &mut self.text_renderer {
            tr.render(&gpu.device, &gpu.queue, &mut enc, &view, &mut self.world, w, h);
        }

        gpu.queue.submit(std::iter::once(enc.finish()));
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

        self.sprite_renderer = Some(sprite_renderer);
        self.text_renderer = Some(text_renderer);
        self.gpu = Some(gpu);
        self.window = Some(window);
        self.last_frame = Some(Instant::now());

        log::info!("엔진 초기화 완료");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
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

    /// 이벤트 큐가 비었을 때 → 매 프레임 redraw 요청
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
