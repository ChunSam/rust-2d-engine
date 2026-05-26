/// Phase 46 예제: 미니맵 (오프스크린 RenderTarget)
///
/// 월드 카메라로 정상 렌더, OffscreenCamera로 줌-아웃된 256x256 미니맵을 만들어
/// 화면 우상단에 표시한다.
use engine::{
    App, Camera, OffscreenCamera, RenderLayer, Sprite, Transform,
    ecs::{Entity, System, World},
    WindowConfig,
};
use glam::Vec2;
use winit::keyboard::KeyCode;

// ─── 시스템: 플레이어 이동 ────────────────────────────────────────────────────
struct MoveSystem;

impl System for MoveSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let speed = 150.0;
        let mut dx = 0.0f32;
        let mut dy = 0.0f32;

        {
            if let Some(input) = world.resource::<engine::InputState>() {
                if input.is_pressed(KeyCode::ArrowLeft) || input.is_pressed(KeyCode::KeyA) {
                    dx -= 1.0;
                }
                if input.is_pressed(KeyCode::ArrowRight) || input.is_pressed(KeyCode::KeyD) {
                    dx += 1.0;
                }
                if input.is_pressed(KeyCode::ArrowUp) || input.is_pressed(KeyCode::KeyW) {
                    dy += 1.0;
                }
                if input.is_pressed(KeyCode::ArrowDown) || input.is_pressed(KeyCode::KeyS) {
                    dy -= 1.0;
                }
            }
        }

        let entities: Vec<Entity> = world
            .query::<PlayerTag>()
            .map(|(e, _)| e)
            .collect();
        let dt = _dt;
        for e in &entities {
            if let Some(t) = world.get_mut::<Transform>(*e) {
                t.position.x += dx * speed * dt;
                t.position.y += dy * speed * dt;
            }
        }

        // 메인 카메라를 플레이어 위치에 추적
        let player_pos = entities
            .first()
            .and_then(|&e| world.get::<Transform>(e))
            .map(|t| t.position);
        if let (Some(pos), Some(cam)) = (player_pos, world.resource_mut::<Camera>()) {
            cam.position = pos;
        }
    }

    fn name(&self) -> &'static str {
        "MoveSystem"
    }
}

// ─── 태그 컴포넌트 ───────────────────────────────────────────────────────────
#[derive(Clone)]
struct PlayerTag;

fn main() {
    let mut app = App::new();

    // 윈도우 설정
    app.world.insert_resource(WindowConfig {
        title: "Phase 46 — Minimap".into(),
        width: 800,
        height: 600,
        clear_color: [0.08, 0.10, 0.15, 1.0],
    });

    // ─── 오프스크린 렌더 타겟 등록 (미니맵 256×256) ─────────────────────────
    app.create_render_target("minimap", 256, 256);

    // ─── 플레이어 (녹색 박스) ────────────────────────────────────────────────
    let player = app.world.spawn();
    app.world.add_component(
        player,
        Transform {
            position: Vec2::new(0.0, 0.0),
            scale: Vec2::new(40.0, 40.0),
            ..Default::default()
        },
    );
    app.world.add_component(
        player,
        Sprite {
            color: [0.2, 0.9, 0.3, 1.0],
            ..Default::default()
        },
    );
    app.world.add_component(player, PlayerTag);

    // ─── 적들 (빨간 박스) ────────────────────────────────────────────────────
    let enemy_positions = [
        Vec2::new(200.0, 100.0),
        Vec2::new(-150.0, 200.0),
        Vec2::new(300.0, -100.0),
        Vec2::new(-250.0, -180.0),
        Vec2::new(120.0, 280.0),
    ];
    for pos in enemy_positions {
        let e = app.world.spawn();
        app.world.add_component(
            e,
            Transform {
                position: pos,
                scale: Vec2::new(30.0, 30.0),
                ..Default::default()
            },
        );
        app.world.add_component(
            e,
            Sprite {
                color: [0.9, 0.2, 0.2, 1.0],
                ..Default::default()
            },
        );
    }

    // ─── 배경 타일들 (회색 박스들) ────────────────────────────────────────────
    for i in -5..=5 {
        for j in -5..=5 {
            let bg = app.world.spawn();
            app.world.add_component(
                bg,
                Transform {
                    position: Vec2::new(i as f32 * 80.0, j as f32 * 80.0),
                    scale: Vec2::new(70.0, 70.0),
                    z: -1.0,
                    ..Default::default()
                },
            );
            let shade = if (i + j) % 2 == 0 { 0.15 } else { 0.20 };
            app.world.add_component(
                bg,
                Sprite {
                    color: [shade, shade, shade, 1.0],
                    ..Default::default()
                },
            );
            app.world.add_component(bg, RenderLayer(-1));
        }
    }

    // ─── OffscreenCamera 엔티티 (미니맵용, 줌 아웃) ──────────────────────────
    let oc_entity = app.world.spawn();
    app.world.add_component(
        oc_entity,
        OffscreenCamera {
            target: "minimap".to_string(),
            camera: Camera::new(Vec2::ZERO, 0.15),
            layer_mask: 1 << 0, // 게임 월드(layer 0)만 — 미니맵 표시용 스프라이트(layer 1) 제외
        },
    );

    // ─── 미니맵 표시용 스프라이트 (화면 우상단 고정) ──────────────────────────
    // ViewportSystem이 아직 없으므로 고정 좌표 사용.
    // 화면 우상단: (800 - 256/2 - 10, 600 - 256/2 - 10) = (618, 462)
    // UI 좌표계는 좌하단 원점, 오른쪽/위쪽이 양수.
    let minimap_sprite = app.world.spawn();
    app.world.add_component(
        minimap_sprite,
        Transform {
            position: Vec2::new(272.0, -172.0), // 화면 우상단 근처 (월드 공간)
            scale: Vec2::new(180.0, 180.0),
            z: 100.0, // 최상위 레이어
            ..Default::default()
        },
    );
    app.world.add_component(
        minimap_sprite,
        Sprite {
            texture: Some("minimap".to_string()), // RT 키
            color: [1.0, 1.0, 1.0, 0.9],
            ..Default::default()
        },
    );
    app.world.add_component(minimap_sprite, RenderLayer(10));

    app.add_system(MoveSystem);
    app.run();
}
