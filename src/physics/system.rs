use std::collections::{HashMap, HashSet};

use glam::Vec2;
use rapier2d::prelude::*;

use crate::components::Transform;
use crate::ecs::{Entity, Events, System, World};
use crate::physics::body::PhysicsBody;
use crate::physics::events::CollisionEvent;
use crate::physics::world::PhysicsWorld;

/// 범용 물리 시스템: 매 프레임 step → Transform 동기화.
///
/// 플레이어 제어 등 커스텀 로직이 필요하면 이 타입을 그대로 쓰지 않고,
/// `PhysicsWorld`를 직접 소유하는 전용 시스템을 만드는 것을 권장한다.
pub struct PhysicsSystem {
    pub physics: PhysicsWorld,
    /// 화면 픽셀당 물리 단위 비율. 예: 50.0 → 1 unit = 50px
    pub pixels_per_unit: f32,
    active_contacts: HashSet<(ColliderHandle, ColliderHandle)>,
}

impl PhysicsSystem {
    pub fn new(physics: PhysicsWorld, pixels_per_unit: f32) -> Self {
        Self {
            physics,
            pixels_per_unit,
            active_contacts: HashSet::new(),
        }
    }
}

impl System for PhysicsSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        self.physics.step(dt);

        // ── 충돌 이벤트 diff ──────────────────────────────────────────────────
        let col_map: HashMap<ColliderHandle, Entity> = world
            .query::<PhysicsBody>()
            .map(|(e, b)| (b.collider_handle, e))
            .collect();

        // Rapier는 동일 쌍의 collider1/collider2 순서를 프레임 간 유지한다
        let current: HashSet<(ColliderHandle, ColliderHandle)> = self
            .physics
            .narrow_phase
            .contact_pairs()
            .filter(|p| p.has_any_active_contact)
            .filter_map(|p| {
                col_map.get(&p.collider1)?;
                col_map.get(&p.collider2)?;
                Some((p.collider1, p.collider2))
            })
            .collect();

        let mut collision_events: Vec<CollisionEvent> = Vec::new();
        for &(c1, c2) in &current {
            if !self.active_contacts.contains(&(c1, c2)) {
                if let (Some(&e1), Some(&e2)) = (col_map.get(&c1), col_map.get(&c2)) {
                    collision_events.push(CollisionEvent::Started(e1, e2));
                }
            }
        }
        for &(c1, c2) in &self.active_contacts {
            if !current.contains(&(c1, c2)) {
                if let (Some(&e1), Some(&e2)) = (col_map.get(&c1), col_map.get(&c2)) {
                    collision_events.push(CollisionEvent::Stopped(e1, e2));
                }
            }
        }
        self.active_contacts = current;

        if !collision_events.is_empty() {
            if let Some(bus) = world.resource_mut::<Events<CollisionEvent>>() {
                for ev in collision_events {
                    bus.send(ev);
                }
            }
        }
        // ── end 충돌 이벤트 diff ─────────────────────────────────────────────

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
