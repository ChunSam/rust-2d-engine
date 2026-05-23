use crate::ecs::{System, World};

/// 씬 트레잇. 각 게임 화면(메뉴, 플레이, 게임오버 등)이 구현한다.
///
/// # 사용 예
/// ```rust,no_run
/// # use engine::{scene::{Scene, SceneCmd, SceneChange}, ecs::{System, World}};
/// struct GamePlay;
///
/// impl Scene for GamePlay {
///     fn on_enter(&mut self, world: &mut World, systems: &mut Vec<Box<dyn System>>) {
///         // 엔티티 스폰, 리소스 삽입
///     }
///     fn on_exit(&mut self, _world: &mut World) {}
/// }
/// ```
pub trait Scene: 'static {
    /// 씬 진입 시 호출. 엔티티 스폰·리소스 삽입·시스템 등록을 여기서 한다.
    fn on_enter(&mut self, world: &mut World, systems: &mut Vec<Box<dyn System>>);
    /// 씬 종료 시 호출. 정리 작업이 필요할 때만 구현한다.
    fn on_exit(&mut self, _world: &mut World) {}
}

/// 씬 전환 명령.
pub enum SceneCmd {
    /// 현재 씬 스택을 전부 비우고 새 씬으로 교체한다 (월드 리셋 포함).
    Replace(Box<dyn Scene>),
    /// 현재 씬 위에 새 씬을 쌓는다 (월드 유지, 일시정지 메뉴 등에 사용).
    Push(Box<dyn Scene>),
    /// 최상위 씬을 꺼낸다.
    Pop,
}

/// 시스템이 씬 전환을 요청하기 위해 쓰는 리소스.
///
/// # 사용 예
/// ```rust,no_run
/// # use engine::{ecs::World, scene::{SceneChange, SceneCmd}};
/// # struct NextScene;
/// # impl engine::scene::Scene for NextScene {
/// #     fn on_enter(&mut self, _: &mut World, _: &mut Vec<Box<dyn engine::ecs::System>>) {}
/// # }
/// # struct MySystem;
/// # impl engine::ecs::System for MySystem {
/// fn run(&mut self, world: &mut World, _dt: f32) {
///     if let Some(sc) = world.resource_mut::<SceneChange>() {
///         sc.request(SceneCmd::Replace(Box::new(NextScene)));
///     }
/// }
/// # }
/// ```
#[derive(Default)]
pub struct SceneChange(pub(crate) Option<SceneCmd>);

impl SceneChange {
    /// 씬 전환 명령을 등록한다. 같은 프레임에 여러 번 호출하면 마지막 명령만 유효하다.
    pub fn request(&mut self, cmd: SceneCmd) {
        self.0 = Some(cmd);
    }
}
