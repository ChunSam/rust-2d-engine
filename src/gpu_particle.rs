use glam::Vec2;
use rand::Rng;

use crate::components::Transform;
use crate::ecs::{Entity, World};
use crate::renderer::gpu_particle::GpuParticle;

/// GPU 컴퓨트 셰이더로 업데이트되는 파티클 이미터 컴포넌트.
///
/// 네이티브 전용 (WASM에서는 CPU `ParticleEmitter` 사용).
///
/// # 예시
/// ```rust,no_run
/// # use engine::{App, GpuParticleEmitter, Transform};
/// # use glam::Vec2;
/// # let mut app = App::new();
/// # let world = &mut app.world;
/// # let entity = world.spawn();
/// world.add_component(entity, Transform { position: Vec2::ZERO, ..Default::default() });
/// let mut emitter = GpuParticleEmitter::default();
/// emitter.spawn_rate = 100.0;
/// emitter.lifetime = 2.0;
/// emitter.velocity = Vec2::new(0.0, 80.0);
/// emitter.velocity_spread = Vec2::new(30.0, 20.0);
/// emitter.color_start = [1.0, 0.5, 0.0, 1.0];
/// emitter.color_end = [1.0, 0.0, 0.0, 0.0];
/// emitter.size = 6.0;
/// emitter.emit = true;
/// world.add_component(entity, emitter);
/// ```
pub struct GpuParticleEmitter {
    /// 초당 방출 파티클 수
    pub spawn_rate: f32,
    /// 파티클 수명 (초)
    pub lifetime: f32,
    /// 기본 속도 (픽셀/초)
    pub velocity: Vec2,
    /// 속도 랜덤 범위 (±각 축)
    pub velocity_spread: Vec2,
    /// 시작 색상 (RGBA)
    pub color_start: [f32; 4],
    /// 종료 색상 (RGBA)
    pub color_end: [f32; 4],
    /// 파티클 크기 (픽셀)
    pub size: f32,
    /// false이면 방출 중단
    pub emit: bool,
    /// 내부 방출 타이머
    pub(crate) timer: f32,
    /// 다음 방출할 링 버퍼 슬롯
    pub(crate) next_slot: u32,
}

impl Default for GpuParticleEmitter {
    fn default() -> Self {
        Self {
            spawn_rate: 50.0,
            lifetime: 1.5,
            velocity: Vec2::new(0.0, 60.0),
            velocity_spread: Vec2::new(20.0, 10.0),
            color_start: [1.0, 0.8, 0.2, 1.0],
            color_end: [1.0, 0.2, 0.0, 0.0],
            size: 5.0,
            emit: true,
            timer: 0.0,
            next_slot: 0,
        }
    }
}

/// GPU 파티클 이미터를 처리해 새 파티클 데이터를 수집한다.
///
/// `App`의 렌더 루프에서 `GpuParticleRenderer::upload_particles`와 함께 사용한다.
pub(crate) fn collect_new_particles(
    world: &mut World,
    capacity: u32,
    dt: f32,
) -> Vec<(u32, GpuParticle)> {
    let mut rng = rand::thread_rng();
    let mut result: Vec<(u32, GpuParticle)> = Vec::new();

    let emitter_entities: Vec<Entity> = world
        .query::<GpuParticleEmitter>()
        .map(|(e, _)| e)
        .collect();

    for entity in emitter_entities {
        let pos = world
            .get::<Transform>(entity)
            .map(|t| t.position)
            .unwrap_or(Vec2::ZERO);

        let emitter = match world.get_mut::<GpuParticleEmitter>(entity) {
            Some(e) => e,
            None => continue,
        };

        if !emitter.emit {
            continue;
        }

        emitter.timer += dt;
        let interval = 1.0 / emitter.spawn_rate.max(0.001);

        while emitter.timer >= interval {
            emitter.timer -= interval;

            let vx = emitter.velocity.x
                + rng.gen_range(-emitter.velocity_spread.x..=emitter.velocity_spread.x);
            let vy = emitter.velocity.y
                + rng.gen_range(-emitter.velocity_spread.y..=emitter.velocity_spread.y);

            let particle = GpuParticle {
                pos: [pos.x, pos.y],
                vel: [vx, vy],
                life: emitter.lifetime,
                max_life: emitter.lifetime,
                size: emitter.size,
                _pad: 0.0,
                color_start: emitter.color_start,
                color_end: emitter.color_end,
            };

            let slot = emitter.next_slot % capacity;
            emitter.next_slot = emitter.next_slot.wrapping_add(1);
            result.push((slot, particle));
        }
    }

    result
}
