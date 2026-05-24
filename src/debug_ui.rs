/// egui 인게임 디버그 오버레이 리소스.
///
/// ECS World에 Resource로 삽입되어 `System` 안에서 `debug_ui.ctx()`로 egui 윈도우를 그릴 수 있다.
/// F1 키로 토글한다. 비활성 상태에서는 draw call을 스킵한다.
///
/// # 사용법
/// ```rust,no_run
/// # use engine::{DebugUi, System, World};
/// struct MyDebugPanel;
/// impl System for MyDebugPanel {
///     fn run(&mut self, world: &mut World, _dt: f32) {
///         let debug = world.resource::<DebugUi>().unwrap();
///         if !debug.is_enabled() { return; }
///         egui::Window::new("Stats").show(debug.ctx(), |ui| {
///             ui.label("Hello from debug!");
///         });
///     }
/// }
/// ```
pub struct DebugUi {
    ctx: egui::Context,
    enabled: bool,
}

impl DebugUi {
    pub(crate) fn new_with_ctx(ctx: egui::Context) -> Self {
        Self { ctx, enabled: false }
    }

    /// egui 드로 컨텍스트를 반환한다. `begin_frame`/`end_frame` 사이에만 사용해야 한다.
    pub fn ctx(&self) -> &egui::Context {
        &self.ctx
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.enabled = v;
    }
}
