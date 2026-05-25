/// Phase 47 예제: 터치 입력 + 가상 조이스틱 데모
///
/// - 화면 좌하단 가상 조이스틱 (반경 60px) 으로 플레이어 이동
/// - 활성 터치 포인트마다 원형 시각화 (DebugDraw)
/// - 핀치 줌: 두 손가락 거리로 카메라 줌 조절
/// - 스와이프: 방향에 따라 콘솔 출력
///
/// 데스크톱에서는 마우스 클릭이 터치로 에뮬레이션되어 조이스틱을 조작할 수 있다.
use engine::{
    App, Camera, DebugDraw, Sprite, TouchState, Transform, VirtualJoystick, WindowConfig,
    ecs::{Entity, System, World},
};
use glam::Vec2;

// ─── 컴포넌트 ─────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct Player;

#[derive(Clone)]
struct JoystickTag;

// ─── 시스템: 터치 피드백 시각화 ───────────────────────────────────────────────

struct TouchVisualSystem;

impl System for TouchVisualSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        // 활성 터치 포인트 목록 수집 (owned)
        let touches: Vec<Vec2> = world
            .resource::<TouchState>()
            .map(|ts| ts.active_touches().map(|(_, pos)| pos).collect())
            .unwrap_or_default();

        // 스와이프 감지 로그
        let swipe = world
            .resource::<TouchState>()
            .and_then(|ts| ts.swipe);
        if let Some(dir) = swipe {
            let label = if dir.x.abs() > dir.y.abs() {
                if dir.x > 0.0 { "오른쪽" } else { "왼쪽" }
            } else if dir.y > 0.0 {
                "아래"
            } else {
                "위"
            };
            log::info!("스와이프: {} ({:.0}, {:.0})", label, dir.x, dir.y);
        }

        if let Some(dbg) = world.resource_mut::<DebugDraw>() {
            for pos in touches {
                dbg.circle(pos, 24.0, [1.0, 0.6, 0.0, 0.7]);
            }
        }
    }

    fn name(&self) -> &'static str {
        "TouchVisualSystem"
    }
}

// ─── 시스템: 핀치 줌 ─────────────────────────────────────────────────────────

struct PinchZoomSystem;

impl System for PinchZoomSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let pinch_delta = world
            .resource::<TouchState>()
            .map(|ts| ts.pinch_delta)
            .unwrap_or(0.0);

        if pinch_delta.abs() > 0.5 {
            if let Some(cam) = world.resource_mut::<Camera>() {
                // 핀치 델타 1px ≈ 0.002 줌 변화 (경험적 값)
                let zoom_change = pinch_delta * 0.002;
                cam.zoom = (cam.zoom + zoom_change).clamp(0.1, 5.0);
            }
        }
    }

    fn name(&self) -> &'static str {
        "PinchZoomSystem"
    }
}

// ─── 시스템: 조이스틱 업데이트 + 플레이어 이동 ───────────────────────────────

struct JoystickMoveSystem;

impl System for JoystickMoveSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        let speed = 200.0;

        // 1. 터치 데이터를 owned 값으로 추출 (borrow 해제)
        let (began, ended, active): (Vec<(u64, Vec2)>, Vec<(u64, Vec2)>, Vec<(u64, Vec2)>) =
            world
                .resource::<TouchState>()
                .map(|ts| {
                    (
                        ts.began.clone(),
                        ts.ended.clone(),
                        ts.active_touches().collect(),
                    )
                })
                .unwrap_or_default();

        // 2. 조이스틱 엔티티 목록 수집
        let joy_entities: Vec<Entity> = world
            .query::<VirtualJoystick>()
            .map(|(e, _)| e)
            .collect();

        // 3. 조이스틱 업데이트 (update_raw 사용, borrow 충돌 없음)
        for &e in &joy_entities {
            if let Some(joy) = world.get_mut::<VirtualJoystick>(e) {
                joy.update_raw(&began, &ended, &active);
            }
        }

        // 4. 첫 번째 조이스틱 output 읽기
        let move_dir: Vec2 = joy_entities
            .first()
            .and_then(|&e| world.get::<VirtualJoystick>(e))
            .map(|joy| joy.output)
            .unwrap_or(Vec2::ZERO);

        // 5. 플레이어 이동
        if move_dir.length() > 0.01 {
            let player_entities: Vec<Entity> = world
                .query::<Player>()
                .map(|(e, _)| e)
                .collect();

            for e in player_entities {
                if let Some(t) = world.get_mut::<Transform>(e) {
                    t.position += move_dir * speed * dt;
                    // 화면 경계 클램프 (800x600 기준)
                    t.position.x = t.position.x.clamp(24.0, 776.0);
                    t.position.y = t.position.y.clamp(24.0, 576.0);
                }
            }
        }

        // 6. 조이스틱 DebugDraw 시각화
        let joy_visuals: Vec<(Vec2, f32, Vec2)> = world
            .query::<VirtualJoystick>()
            .filter_map(|(_, joy)| {
                if joy.visible {
                    Some((joy.center, joy.radius, joy.stick_pos))
                } else {
                    None
                }
            })
            .collect();

        if let Some(dbg) = world.resource_mut::<DebugDraw>() {
            for (center, radius, stick_pos) in joy_visuals {
                // 베이스 원 (흰색, 반투명)
                dbg.circle(center, radius, [1.0, 1.0, 1.0, 0.3]);
                // 스틱 핵 (노란색)
                dbg.circle(stick_pos, 18.0, [1.0, 0.9, 0.1, 0.8]);
            }
        }
    }

    fn name(&self) -> &'static str {
        "JoystickMoveSystem"
    }
}

// ─── 메인 ────────────────────────────────────────────────────────────────────

fn main() {
    let mut app = App::new();

    app.world.insert_resource(WindowConfig {
        title: "Phase 47 — Touch Input & Virtual Joystick".into(),
        width: 800,
        height: 600,
        clear_color: [0.07, 0.08, 0.14, 1.0],
    });

    // ─── 플레이어 (파란 사각형) ──────────────────────────────────────────────
    let player = app.world.spawn();
    app.world.add_component(
        player,
        Transform {
            position: Vec2::new(400.0, 300.0),
            scale: Vec2::splat(48.0),
            ..Default::default()
        },
    );
    app.world.add_component(
        player,
        Sprite {
            color: [0.3, 0.6, 1.0, 1.0],
            ..Default::default()
        },
    );
    app.world.add_component(player, Player);

    // ─── 배경 격자 ───────────────────────────────────────────────────────────
    for i in 0..8 {
        for j in 0..6 {
            let bg = app.world.spawn();
            app.world.add_component(
                bg,
                Transform {
                    position: Vec2::new(50.0 + i as f32 * 100.0, 50.0 + j as f32 * 100.0),
                    scale: Vec2::splat(90.0),
                    z: -1.0,
                    ..Default::default()
                },
            );
            let shade = if (i + j) % 2 == 0 { 0.10 } else { 0.14 };
            app.world.add_component(
                bg,
                Sprite {
                    color: [shade, shade, shade + 0.02, 1.0],
                    ..Default::default()
                },
            );
        }
    }

    // ─── 가상 조이스틱 엔티티 (화면 좌하단) ──────────────────────────────────
    // 좌표계: 좌상단 (0,0), 우하단 (800,600)
    let joy_entity = app.world.spawn();
    app.world.add_component(joy_entity, JoystickTag);
    app.world.add_component(
        joy_entity,
        VirtualJoystick::new(Vec2::new(120.0, 480.0), 60.0),
    );

    // ─── 시스템 등록 ─────────────────────────────────────────────────────────
    app.add_system(TouchVisualSystem);
    app.add_system(PinchZoomSystem);
    app.add_system(JoystickMoveSystem);

    app.run();
}
