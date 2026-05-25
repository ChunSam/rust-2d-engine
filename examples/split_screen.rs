/// Phase 46 예제: 분할화면 (OffscreenCamera × 2)
///
/// P1(녹색)과 P2(파란색) 플레이어가 서로 다른 위치를 보는 좌우 분할화면을 구현한다.
/// - 화면 왼쪽 절반: "left_view" RenderTarget (P1 시점)
/// - 화면 오른쪽 절반: "right_view" RenderTarget (P2 시점)
use engine::{
    App, Camera, OffscreenCamera, RenderLayer, Sprite, Transform,
    ecs::{System, World},
    WindowConfig,
};
use glam::Vec2;
use winit::keyboard::KeyCode;

// ─── 태그 컴포넌트 ───────────────────────────────────────────────────────────
#[derive(Clone)]
struct Player1;

#[derive(Clone)]
struct Player2;

// ─── 시스템: P1 이동 (WASD) + P2 이동 (방향키) ────────────────────────────
struct MoveSystem;

impl System for MoveSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        let speed = 200.0;

        // P1: WASD, P2: 화살표
        let mut d1 = Vec2::ZERO;
        let mut d2 = Vec2::ZERO;

        if let Some(input) = world.resource::<engine::InputState>() {
            if input.is_pressed(KeyCode::KeyA) {
                d1.x -= 1.0;
            }
            if input.is_pressed(KeyCode::KeyD) {
                d1.x += 1.0;
            }
            if input.is_pressed(KeyCode::KeyW) {
                d1.y += 1.0;
            }
            if input.is_pressed(KeyCode::KeyS) {
                d1.y -= 1.0;
            }
            if input.is_pressed(KeyCode::ArrowLeft) {
                d2.x -= 1.0;
            }
            if input.is_pressed(KeyCode::ArrowRight) {
                d2.x += 1.0;
            }
            if input.is_pressed(KeyCode::ArrowUp) {
                d2.y += 1.0;
            }
            if input.is_pressed(KeyCode::ArrowDown) {
                d2.y -= 1.0;
            }
        }

        // P1 이동
        let p1_entities: Vec<_> = world.query::<Player1>().map(|(e, _)| e).collect();
        for e in &p1_entities {
            if let Some(t) = world.get_mut::<Transform>(*e) {
                t.position += d1 * speed * dt;
            }
        }

        // P2 이동
        let p2_entities: Vec<_> = world.query::<Player2>().map(|(e, _)| e).collect();
        for e in &p2_entities {
            if let Some(t) = world.get_mut::<Transform>(*e) {
                t.position += d2 * speed * dt;
            }
        }

        // OffscreenCamera를 플레이어 위치에 맞게 업데이트
        let p1_pos = p1_entities
            .first()
            .and_then(|&e| world.get::<Transform>(e))
            .map(|t| t.position);
        let p2_pos = p2_entities
            .first()
            .and_then(|&e| world.get::<Transform>(e))
            .map(|t| t.position);

        let oc_entities: Vec<_> = world
            .query::<OffscreenCamera>()
            .map(|(e, _)| e)
            .collect();
        for e in oc_entities {
            if let Some(oc) = world.get_mut::<OffscreenCamera>(e) {
                if oc.target == "left_view" {
                    if let Some(pos) = p1_pos {
                        oc.camera.position = pos;
                    }
                } else if oc.target == "right_view" {
                    if let Some(pos) = p2_pos {
                        oc.camera.position = pos;
                    }
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "MoveSystem"
    }
}

fn main() {
    let mut app = App::new();

    // 윈도우 설정 (800×600)
    app.world.insert_resource(WindowConfig {
        title: "Phase 46 — Split Screen".into(),
        width: 800,
        height: 600,
        clear_color: [0.05, 0.05, 0.08, 1.0],
    });

    // ─── 오프스크린 렌더 타겟 등록 ──────────────────────────────────────────
    // 각 뷰는 화면 절반 너비×높이
    app.create_render_target("left_view", 400, 600);
    app.create_render_target("right_view", 400, 600);

    // ─── 플레이어 1 (녹색, 좌측) ─────────────────────────────────────────────
    let p1 = app.world.spawn();
    app.world.add_component(
        p1,
        Transform {
            position: Vec2::new(-200.0, 0.0),
            scale: Vec2::new(40.0, 40.0),
            z: 1.0,
            ..Default::default()
        },
    );
    app.world.add_component(
        p1,
        Sprite {
            color: [0.2, 0.9, 0.3, 1.0],
            ..Default::default()
        },
    );
    app.world.add_component(p1, Player1);

    // ─── 플레이어 2 (파란색, 우측) ───────────────────────────────────────────
    let p2 = app.world.spawn();
    app.world.add_component(
        p2,
        Transform {
            position: Vec2::new(200.0, 0.0),
            scale: Vec2::new(40.0, 40.0),
            z: 1.0,
            ..Default::default()
        },
    );
    app.world.add_component(
        p2,
        Sprite {
            color: [0.2, 0.4, 1.0, 1.0],
            ..Default::default()
        },
    );
    app.world.add_component(p2, Player2);

    // ─── 배경 오브젝트들 ─────────────────────────────────────────────────────
    let bg_objects = [
        (Vec2::new(0.0, 150.0), [0.8, 0.7, 0.2, 1.0f32]),
        (Vec2::new(-100.0, -100.0), [0.7, 0.3, 0.8, 1.0]),
        (Vec2::new(100.0, -150.0), [0.3, 0.7, 0.8, 1.0]),
        (Vec2::new(-300.0, 50.0), [0.9, 0.5, 0.2, 1.0]),
        (Vec2::new(300.0, 100.0), [0.5, 0.9, 0.5, 1.0]),
    ];
    for (pos, color) in bg_objects {
        let e = app.world.spawn();
        app.world.add_component(
            e,
            Transform {
                position: pos,
                scale: Vec2::new(50.0, 50.0),
                z: 0.0,
                ..Default::default()
            },
        );
        app.world.add_component(e, Sprite { color, ..Default::default() });
    }

    // ─── 배경 타일 ───────────────────────────────────────────────────────────
    for i in -6..=6 {
        for j in -4..=4 {
            let bg = app.world.spawn();
            app.world.add_component(
                bg,
                Transform {
                    position: Vec2::new(i as f32 * 100.0, j as f32 * 100.0),
                    scale: Vec2::new(90.0, 90.0),
                    z: -1.0,
                    ..Default::default()
                },
            );
            let shade = if (i + j) % 2 == 0 { 0.12 } else { 0.18 };
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

    // ─── OffscreenCamera 엔티티 ───────────────────────────────────────────────
    // left_view: P1 초기 위치
    let oc1 = app.world.spawn();
    app.world.add_component(
        oc1,
        OffscreenCamera {
            target: "left_view".to_string(),
            camera: Camera::new(Vec2::new(-200.0, 0.0), 1.0),
        },
    );

    // right_view: P2 초기 위치
    let oc2 = app.world.spawn();
    app.world.add_component(
        oc2,
        OffscreenCamera {
            target: "right_view".to_string(),
            camera: Camera::new(Vec2::new(200.0, 0.0), 1.0),
        },
    );

    // ─── 화면에 표시할 뷰 스프라이트 ─────────────────────────────────────────
    // 화면 중심 (0,0) 기준, 월드 좌표로 배치.
    // left_view: 화면 왼쪽 절반 (-200, 0), 크기 400×600
    // right_view: 화면 오른쪽 절반 (+200, 0), 크기 400×600
    let left_sprite = app.world.spawn();
    app.world.add_component(
        left_sprite,
        Transform {
            position: Vec2::new(-200.0, 0.0),
            scale: Vec2::new(400.0, 600.0),
            z: 200.0,
            ..Default::default()
        },
    );
    app.world.add_component(
        left_sprite,
        Sprite {
            texture: Some("left_view".to_string()),
            color: [1.0, 1.0, 1.0, 1.0],
            ..Default::default()
        },
    );
    app.world.add_component(left_sprite, RenderLayer(20));

    let right_sprite = app.world.spawn();
    app.world.add_component(
        right_sprite,
        Transform {
            position: Vec2::new(200.0, 0.0),
            scale: Vec2::new(400.0, 600.0),
            z: 200.0,
            ..Default::default()
        },
    );
    app.world.add_component(
        right_sprite,
        Sprite {
            texture: Some("right_view".to_string()),
            color: [1.0, 1.0, 1.0, 1.0],
            ..Default::default()
        },
    );
    app.world.add_component(right_sprite, RenderLayer(20));

    app.add_system(MoveSystem);
    app.run();
}
