use glam::Vec2;
use rapier2d::prelude::*;

use crate::components::Transform;
use crate::ecs::{Entity, System, World};
use crate::physics::body::PhysicsBody;
use crate::physics::world::PhysicsWorld;

/// 범용 물리 시스템: 매 프레임 step → Transform 동기화.
///
/// 플레이어 제어 등 커스텀 로직이 필요하면 이 타입을 그대로 쓰지 않고,
/// `PhysicsWorld`를 직접 소유하는 전용 시스템을 만드는 것을 권장한다.
pub struct PhysicsSystem {
    pub physics: PhysicsWorld,
    /// 화면 픽셀당 물리 단위 비율. 예: 50.0 → 1 unit = 50px
    pub pixels_per_unit: f32,
}

impl PhysicsSystem {
    pub fn new(physics: PhysicsWorld, pixels_per_unit: f32) -> Self {
        Self {
            physics,
            pixels_per_unit,
        }
    }
}

impl System for PhysicsSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        self.physics.step(dt);

        // borrow checker: (entity, handle) 를 먼저 수집해야 world를 다시 빌릴 수 있다
        let pairs: Vec<(Entity, RigidBodyHandle)> = world
            .query::<PhysicsBody>()
            .map(|(e, b)| (e, b.rigid_body_handle))
            .collect();

        let scale = self.pixels_per_unit;
        for (entity, handle) in pairs {
            if let Some(body) = self.physics.rigid_body(handle) {
                let t = *body.translation();
                if let Some(tr) = world.get_mut::<Transform>(entity) {
                    tr.position = Vec2::new(t.x * scale, t.y * scale);
                }
            }
        }
    }
}
