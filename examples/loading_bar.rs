/// Phase 51 예제: 비동기 에셋 로딩 + 진행률 바
///
/// - LoadingScene: 여러 이미지를 load_image_async로 비동기 로드하면서
///   DrawRect로 진행률 바를 표시한다.
/// - 로딩 완료 후 GameScene으로 자동 전환한다.
use engine::{
    ecs::{System, World},
    renderer::{DrawRect, TextQueue, UiQueue},
    resources::WindowConfig,
    App, AssetServer, LoadProgress, Scene, SceneChange, SceneCmd,
};

// ─── 로딩 씬 ─────────────────────────────────────────────────────────────────

struct LoadingScene;

impl Scene for LoadingScene {
    fn on_enter(&mut self, world: &mut World, systems: &mut Vec<Box<dyn System>>) {
        // 여러 이미지를 비동기로 로드 요청 (파일 없어도 마젠타 폴백으로 대체됨)
        let paths = [
            "assets/bg.png",
            "assets/player.png",
            "assets/enemy.png",
            "assets/tileset.png",
        ];
        let mut count = 0usize;
        if let Some(assets) = world.resource_mut::<AssetServer>() {
            for path in &paths {
                assets.load_image_async(*path);
                count += 1;
            }
        }
        // LoadProgress 초기화
        if let Some(prog) = world.resource_mut::<LoadProgress>() {
            prog.total = count;
            prog.loaded = 0;
        }
        systems.push(Box::new(LoadingUpdateSystem { done: false }));
    }

    fn on_exit(&mut self, _world: &mut World) {}
}

// ─── 로딩 업데이트 시스템 ──────────────────────────────────────────────────────

struct LoadingUpdateSystem {
    done: bool,
}

impl System for LoadingUpdateSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let (total, loaded) = world
            .resource::<LoadProgress>()
            .map(|p| (p.total, p.loaded))
            .unwrap_or((0, 0));

        let progress = if total == 0 {
            1.0f32
        } else {
            loaded as f32 / total as f32
        };

        // 진행률 바 렌더링
        let bar_w = 400.0f32;
        let bar_h = 36.0f32;
        let bar_x = -bar_w / 2.0;
        let bar_y = -bar_h / 2.0;

        if let Some(ui) = world.resource_mut::<UiQueue>() {
            // 배경 바
            ui.items.push(DrawRect {
                x: bar_x - 2.0,
                y: bar_y - 2.0,
                w: bar_w + 4.0,
                h: bar_h + 4.0,
                color: [0.15, 0.15, 0.15, 1.0],
                z: 0.5,
            });
            // 진행 바
            ui.items.push(DrawRect {
                x: bar_x,
                y: bar_y,
                w: (bar_w * progress).max(0.0),
                h: bar_h,
                color: [0.3, 0.8, 0.3, 1.0],
                z: 0.6,
            });
        }

        // 퍼센트 텍스트
        if let Some(tq) = world.resource_mut::<TextQueue>() {
            tq.push(engine::renderer::DrawText::new(
                format!("Loading... {:.0}%", progress * 100.0),
                glam::Vec2::new(-70.0, -60.0),
                22.0,
                [255, 255, 255, 255],
            ));
        }

        // 완료 시 씬 전환
        if !self.done && loaded >= total && total > 0 {
            self.done = true;
            if let Some(sc) = world.resource_mut::<SceneChange>() {
                sc.request(SceneCmd::Replace(Box::new(GameScene)));
            }
        }
    }
}

// ─── 게임 씬 ─────────────────────────────────────────────────────────────────

struct GameScene;

impl Scene for GameScene {
    fn on_enter(&mut self, _world: &mut World, systems: &mut Vec<Box<dyn System>>) {
        systems.push(Box::new(GameUpdateSystem));
    }
    fn on_exit(&mut self, _world: &mut World) {}
}

struct GameUpdateSystem;

impl System for GameUpdateSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        if let Some(tq) = world.resource_mut::<TextQueue>() {
            tq.push(engine::renderer::DrawText::new(
                "Loading complete! Game ready.",
                glam::Vec2::new(-130.0, 0.0),
                22.0,
                [100, 255, 100, 255],
            ));
        }
    }
}

// ─── 진입점 ──────────────────────────────────────────────────────────────────

fn main() {
    let mut app = App::new();
    app.world.insert_resource(WindowConfig {
        title: "Phase 51 — Async Loading Bar".to_string(),
        width: 800,
        height: 600,
        clear_color: [0.05, 0.05, 0.10, 1.0],
    });

    app.set_scene(Box::new(LoadingScene));
    app.run();
}
