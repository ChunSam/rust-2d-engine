//! 스티어링 행동 (Steering Behaviors) 시스템 (Phase 37a)
//!
//! 엔티티의 `Transform.position`을 읽어 목표 방향으로 속도 벡터를 계산하고
//! `SteeringVelocity`에 저장한다. `SteeringSystem`이 매 프레임 Transform을 실제로 이동시킨다.
//!
//! # 포함 행동
//! - [`Seek`]   — 목표 위치를 향해 최대 속도로 이동
//! - [`Flee`]   — 목표 위치로부터 도망 (flee_radius 이내일 때만)
//! - [`Arrive`] — 목표에 가까워지면 감속, stop_radius 이내에서 정지
//! - [`Wander`] — 무작위 방향으로 배회 (change_interval마다 방향 변경)
//!
//! # 등록 예시
//! ```rust,no_run
//! use engine::steering::{Seek, SteeringSystem, SteeringVelocity};
//! use engine::{App, Transform};
//! use glam::Vec2;
//!
//! let mut app = App::new();
//! let e = app.world.spawn();
//! app.world.add_component(e, Transform::default());
//! app.world.add_component(e, SteeringVelocity { velocity: Vec2::ZERO, max_speed: 200.0 });
//! app.world.add_component(e, Seek { target: Vec2::new(400.0, 300.0), max_speed: 200.0 });
//! app.add_system(SteeringSystem);
//! ```

use glam::Vec2;

use crate::components::Transform;
use crate::ecs::{Entity, World};
use crate::System;

// ─── SteeringVelocity ─────────────────────────────────────────────────────────

/// 스티어링 계산 결과를 저장하는 컴포넌트.
///
/// `SteeringSystem`이 각 스티어링 행동 컴포넌트(Seek/Flee/Arrive/Wander)를 평가해
/// 이 필드를 갱신하고, 최종적으로 `Transform.position`에 적용한다.
#[derive(Debug, Clone, Default)]
pub struct SteeringVelocity {
    pub velocity: Vec2,
    pub max_speed: f32,
}

// ─── Seek ─────────────────────────────────────────────────────────────────────

/// 목표 위치를 향해 최대 속도로 직선 이동.
#[derive(Debug, Clone)]
pub struct Seek {
    pub target: Vec2,
    pub max_speed: f32,
}

// ─── Flee ─────────────────────────────────────────────────────────────────────

/// 목표 위치로부터 도망. `flee_radius` 이내일 때만 활성화된다.
#[derive(Debug, Clone)]
pub struct Flee {
    pub target: Vec2,
    pub max_speed: f32,
    /// 이 거리 이내일 때만 도망 속도가 생성된다.
    pub flee_radius: f32,
}

// ─── Arrive ───────────────────────────────────────────────────────────────────

/// 목표에 가까워지면 감속. `stop_radius` 이내에서 정지.
#[derive(Debug, Clone)]
pub struct Arrive {
    pub target: Vec2,
    pub max_speed: f32,
    /// 이 거리 이내에서 감속을 시작한다.
    pub slow_radius: f32,
    /// 이 거리 이내에서 속도를 0으로 만든다.
    pub stop_radius: f32,
}

// ─── Wander ───────────────────────────────────────────────────────────────────

/// 무작위 방향으로 배회. `change_interval`마다 방향을 바꾼다.
#[derive(Debug, Clone)]
pub struct Wander {
    pub max_speed: f32,
    /// 방향 변경 주기 (초).
    pub change_interval: f32,
    pub(crate) timer: f32,
    pub(crate) current_dir: Vec2,
}

impl Wander {
    pub fn new(max_speed: f32, change_interval: f32) -> Self {
        Self {
            max_speed,
            change_interval,
            timer: 0.0,
            current_dir: Vec2::X,
        }
    }
}

// ─── SteeringSystem ───────────────────────────────────────────────────────────

/// 스티어링 행동 컴포넌트를 매 프레임 평가해 `Transform`을 이동시키는 시스템.
///
/// 한 엔티티에 여러 스티어링 컴포넌트가 있을 경우 마지막에 계산된 행동이 적용된다.
/// 등록 순서: Seek → Flee → Arrive → Wander.
pub struct SteeringSystem;

impl System for SteeringSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // ── 1. Seek ────────────────────────────────────────────────────────────
        {
            let entities: Vec<Entity> = world.query::<Seek>().map(|(e, _)| e).collect();

            for entity in entities {
                let (pos, target, max_speed) = {
                    let t = world
                        .query::<Transform>()
                        .find(|(e, _)| *e == entity)
                        .map(|(_, t)| t.position);
                    let seek = world
                        .query::<Seek>()
                        .find(|(e, _)| *e == entity)
                        .map(|(_, s)| (s.target, s.max_speed));
                    match (t, seek) {
                        (Some(p), Some((tgt, ms))) => (p, tgt, ms),
                        _ => continue,
                    }
                };

                let dir = target - pos;
                let velocity = if dir.length_squared() > 1e-6 {
                    dir.normalize() * max_speed
                } else {
                    Vec2::ZERO
                };

                if let Some(sv) = world.get_mut::<SteeringVelocity>(entity) {
                    sv.velocity = velocity;
                    sv.max_speed = max_speed;
                }
            }
        }

        // ── 2. Flee ────────────────────────────────────────────────────────────
        {
            let entities: Vec<Entity> = world.query::<Flee>().map(|(e, _)| e).collect();

            for entity in entities {
                let (pos, target, max_speed, flee_radius) = {
                    let t = world
                        .query::<Transform>()
                        .find(|(e, _)| *e == entity)
                        .map(|(_, t)| t.position);
                    let flee = world
                        .query::<Flee>()
                        .find(|(e, _)| *e == entity)
                        .map(|(_, f)| (f.target, f.max_speed, f.flee_radius));
                    match (t, flee) {
                        (Some(p), Some((tgt, ms, fr))) => (p, tgt, ms, fr),
                        _ => continue,
                    }
                };

                let diff = pos - target;
                let dist = diff.length();
                let velocity = if dist < flee_radius && dist > 1e-6 {
                    diff.normalize() * max_speed
                } else {
                    Vec2::ZERO
                };

                if let Some(sv) = world.get_mut::<SteeringVelocity>(entity) {
                    sv.velocity = velocity;
                    sv.max_speed = max_speed;
                }
            }
        }

        // ── 3. Arrive ──────────────────────────────────────────────────────────
        {
            let entities: Vec<Entity> = world.query::<Arrive>().map(|(e, _)| e).collect();

            for entity in entities {
                let (pos, target, max_speed, slow_radius, stop_radius) = {
                    let t = world
                        .query::<Transform>()
                        .find(|(e, _)| *e == entity)
                        .map(|(_, t)| t.position);
                    let arrive = world
                        .query::<Arrive>()
                        .find(|(e, _)| *e == entity)
                        .map(|(_, a)| (a.target, a.max_speed, a.slow_radius, a.stop_radius));
                    match (t, arrive) {
                        (Some(p), Some((tgt, ms, sr, pr))) => (p, tgt, ms, sr, pr),
                        _ => continue,
                    }
                };

                let dir = target - pos;
                let dist = dir.length();
                let velocity = if dist <= stop_radius {
                    Vec2::ZERO
                } else if dist <= slow_radius {
                    // 선형 감속
                    let ratio = (dist - stop_radius) / (slow_radius - stop_radius);
                    dir.normalize() * max_speed * ratio
                } else if dist > 1e-6 {
                    dir.normalize() * max_speed
                } else {
                    Vec2::ZERO
                };

                if let Some(sv) = world.get_mut::<SteeringVelocity>(entity) {
                    sv.velocity = velocity;
                    sv.max_speed = max_speed;
                }
            }
        }

        // ── 4. Wander ─────────────────────────────────────────────────────────
        {
            let entities: Vec<Entity> = world.query::<Wander>().map(|(e, _)| e).collect();

            for entity in entities {
                // 타이머 갱신 및 방향 결정
                let (max_speed, current_dir) = {
                    let wander = match world.get_mut::<Wander>(entity) {
                        Some(w) => w,
                        None => continue,
                    };
                    wander.timer += dt;
                    if wander.timer >= wander.change_interval {
                        wander.timer = 0.0;
                        // 의사 난수 방향: entity id 기반의 단순한 결정론적 계산
                        // (실제 프로젝트에서는 rand 크레이트 활용 권장)
                        let seed = (entity.0 as f32 * 1.6180339) + wander.current_dir.x * 31.7;
                        let angle = (seed.sin() * 6283.185).abs() % std::f32::consts::TAU;
                        wander.current_dir = Vec2::new(angle.cos(), angle.sin());
                    }
                    (wander.max_speed, wander.current_dir)
                };

                if let Some(sv) = world.get_mut::<SteeringVelocity>(entity) {
                    sv.velocity = current_dir * max_speed;
                    sv.max_speed = max_speed;
                }
            }
        }

        // ── 5. Transform 이동 적용 ─────────────────────────────────────────────
        {
            let entities: Vec<Entity> = world.query::<SteeringVelocity>().map(|(e, _)| e).collect();

            for entity in entities {
                let velocity = match world
                    .query::<SteeringVelocity>()
                    .find(|(e, _)| *e == entity)
                    .map(|(_, sv)| sv.velocity)
                {
                    Some(v) => v,
                    None => continue,
                };

                if let Some(transform) = world.get_mut::<Transform>(entity) {
                    transform.position += velocity * dt;
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "SteeringSystem"
    }
}

// ─── 테스트 ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Transform;
    use crate::ecs::World;
    use glam::Vec2;

    fn make_world_with_transform(pos: Vec2) -> (World, Entity) {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(
            e,
            Transform {
                position: pos,
                scale: Vec2::ONE,
                rotation: 0.0,
                z: 0.0,
            },
        );
        (world, e)
    }

    #[test]
    fn seek_generates_velocity_toward_target() {
        let (mut world, e) = make_world_with_transform(Vec2::ZERO);
        world.add_component(e, SteeringVelocity::default());
        world.add_component(
            e,
            Seek {
                target: Vec2::new(100.0, 0.0),
                max_speed: 200.0,
            },
        );

        let mut sys = SteeringSystem;
        sys.run(&mut world, 0.016);

        let sv = world
            .query::<SteeringVelocity>()
            .find(|(en, _)| *en == e)
            .map(|(_, sv)| sv.velocity)
            .unwrap();

        // 오른쪽(+x)으로 이동해야 함
        assert!(sv.x > 0.0, "velocity.x should be positive, got {}", sv.x);
        assert!(sv.y.abs() < 1e-4, "velocity.y should be ~0, got {}", sv.y);
        let speed = sv.length();
        assert!(
            (speed - 200.0).abs() < 1e-3,
            "speed should equal max_speed=200, got {}",
            speed
        );
    }

    #[test]
    fn arrive_stops_within_stop_radius() {
        // 목표와 동일한 위치 — stop_radius(5.0) 이내
        let (mut world, e) = make_world_with_transform(Vec2::new(1.0, 0.0));
        world.add_component(e, SteeringVelocity::default());
        world.add_component(
            e,
            Arrive {
                target: Vec2::new(2.0, 0.0), // 거리 1.0 < stop_radius=5.0
                max_speed: 200.0,
                slow_radius: 50.0,
                stop_radius: 5.0,
            },
        );

        let mut sys = SteeringSystem;
        sys.run(&mut world, 0.016);

        let sv = world
            .query::<SteeringVelocity>()
            .find(|(en, _)| *en == e)
            .map(|(_, sv)| sv.velocity)
            .unwrap();

        assert!(
            sv.length() < 1e-5,
            "velocity should be ~0 within stop_radius, got {:?}",
            sv
        );
    }

    #[test]
    fn flee_zero_velocity_outside_radius() {
        // flee_radius = 50, 엔티티는 target으로부터 100 떨어져 있음
        let (mut world, e) = make_world_with_transform(Vec2::new(100.0, 0.0));
        world.add_component(e, SteeringVelocity::default());
        world.add_component(
            e,
            Flee {
                target: Vec2::ZERO,
                max_speed: 200.0,
                flee_radius: 50.0,
            },
        );

        let mut sys = SteeringSystem;
        sys.run(&mut world, 0.016);

        let sv = world
            .query::<SteeringVelocity>()
            .find(|(en, _)| *en == e)
            .map(|(_, sv)| sv.velocity)
            .unwrap();

        assert!(
            sv.length() < 1e-5,
            "velocity outside flee_radius should be 0, got {:?}",
            sv
        );
    }

    #[test]
    fn flee_generates_velocity_inside_radius() {
        // 거리 30 < flee_radius 50
        let (mut world, e) = make_world_with_transform(Vec2::new(30.0, 0.0));
        world.add_component(e, SteeringVelocity::default());
        world.add_component(
            e,
            Flee {
                target: Vec2::ZERO,
                max_speed: 200.0,
                flee_radius: 50.0,
            },
        );

        let mut sys = SteeringSystem;
        sys.run(&mut world, 0.016);

        let sv = world
            .query::<SteeringVelocity>()
            .find(|(en, _)| *en == e)
            .map(|(_, sv)| sv.velocity)
            .unwrap();

        // 도망 방향 = +x (target이 원점, 위치가 +x이므로)
        assert!(
            sv.x > 0.0,
            "flee velocity.x should be positive (away from origin)"
        );
        assert!(
            (sv.length() - 200.0).abs() < 1e-3,
            "flee speed should be max_speed when inside radius"
        );
    }
}
