use glam::Vec2;
use rand::Rng;

use crate::components::{Sprite, Transform};
use crate::ecs::{Entity, System, World};

// ─── 컴포넌트 ─────────────────────────────────────────────────────────────────

/// 파티클을 방출하는 이미터 컴포넌트.
///
/// 엔티티에 `Transform`과 함께 붙이면 `ParticleSystem`이 파티클을 생성한다.
pub struct ParticleEmitter {
    /// 초당 파티클 생성 수
    pub spawn_rate: f32,
    /// 파티클 생존 시간 (초)
    pub lifetime: f32,
    /// 기본 속도 (픽셀/초)
    pub velocity: Vec2,
    /// 속도에 추가되는 랜덤 범위 (±각 축)
    pub velocity_spread: Vec2,
    /// 생성 시 색상 (RGBA)
    pub color_start: [f32; 4],
    /// 소멸 시 색상 (RGBA) — 생존 시간에 따라 보간
    pub color_end: [f32; 4],
    /// 파티클 크기 (픽셀)
    pub size: Vec2,
    /// 텍스처 경로. None이면 단색 사각형.
    pub texture: Option<String>,
    /// false이면 방출 중단
    pub emit: bool,
    /// 내부 타이머 (직접 수정 불필요)
    pub(crate) timer: f32,
}

impl Default for ParticleEmitter {
    fn default() -> Self {
        Self {
            spawn_rate: 20.0,
            lifetime: 1.0,
            velocity: Vec2::new(0.0, -50.0),
            velocity_spread: Vec2::new(20.0, 10.0),
            color_start: [1.0, 1.0, 1.0, 1.0],
            color_end: [1.0, 1.0, 1.0, 0.0],
            size: Vec2::splat(8.0),
            texture: None,
            emit: true,
            timer: 0.0,
        }
    }
}

/// 활성 파티클 컴포넌트.
pub struct Particle {
    pub lifetime: f32,
    pub age: f32,
    pub velocity: Vec2,
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
}

// ─── 시스템 ──────────────────────────────────────────────────────────────────

pub struct ParticleSystem;

impl System for ParticleSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // 1. 기존 파티클 이동·색상 업데이트, 만료된 것은 수집
        let updates: Vec<(Entity, f32, f32, Vec2, [f32; 4], [f32; 4])> = world
            .query::<Particle>()
            .map(|(e, p)| (e, p.age, p.lifetime, p.velocity, p.color_start, p.color_end))
            .collect();

        let mut to_despawn = Vec::new();
        for (entity, age, lifetime, velocity, color_start, color_end) in updates {
            let new_age = age + dt;
            if new_age >= lifetime {
                to_despawn.push(entity);
                continue;
            }
            if let Some(tr) = world.get_mut::<Transform>(entity) {
                tr.position += velocity * dt;
            }
            let t = new_age / lifetime;
            let lerped = [
                color_start[0] + (color_end[0] - color_start[0]) * t,
                color_start[1] + (color_end[1] - color_start[1]) * t,
                color_start[2] + (color_end[2] - color_start[2]) * t,
                color_start[3] + (color_end[3] - color_start[3]) * t,
            ];
            if let Some(sp) = world.get_mut::<Sprite>(entity) {
                sp.color = lerped;
            }
            if let Some(p) = world.get_mut::<Particle>(entity) {
                p.age = new_age;
            }
        }
        for e in to_despawn {
            world.despawn(e);
        }

        // 2. 이미터에서 새 파티클 방출
        let emitter_data: Vec<(
            Entity,
            Vec2,
            bool,
            f32,
            f32,
            Vec2,
            Vec2,
            [f32; 4],
            [f32; 4],
            Vec2,
            Option<String>,
        )> = world
            .query2::<Transform, ParticleEmitter>()
            .map(|(e, tr, em)| {
                (
                    e,
                    tr.position,
                    em.emit,
                    em.spawn_rate,
                    em.lifetime,
                    em.velocity,
                    em.velocity_spread,
                    em.color_start,
                    em.color_end,
                    em.size,
                    em.texture.clone(),
                )
            })
            .collect();

        let mut rng = rand::thread_rng();
        for (
            emitter_entity,
            pos,
            emit,
            spawn_rate,
            lifetime,
            velocity,
            spread,
            color_start,
            color_end,
            size,
            texture,
        ) in emitter_data
        {
            if !emit || spawn_rate <= 0.0 {
                continue;
            }
            let should_spawn = {
                let em = world.get_mut::<ParticleEmitter>(emitter_entity).unwrap();
                em.timer += dt;
                let interval = 1.0 / spawn_rate;
                if em.timer >= interval {
                    em.timer -= interval;
                    true
                } else {
                    false
                }
            };
            if !should_spawn {
                continue;
            }

            let actual_velocity = Vec2::new(
                velocity.x + rng.gen_range(-spread.x..=spread.x),
                velocity.y + rng.gen_range(-spread.y..=spread.y),
            );

            let pe = world.spawn();
            world.add_component(
                pe,
                Transform {
                    position: pos,
                    scale: size,
                    rotation: 0.0,
                    z: 0.0,
                },
            );
            let sprite = match texture {
                Some(ref path) => Sprite::textured(path.as_str()),
                None => Sprite {
                    texture: None,
                    color: color_start,
                    image_handle: None,
                },
            };
            world.add_component(pe, sprite);
            world.add_component(
                pe,
                Particle {
                    lifetime,
                    age: 0.0,
                    velocity: actual_velocity,
                    color_start,
                    color_end,
                },
            );
        }
    }
}
