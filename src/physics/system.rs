use glam::Vec2;
use rapier2d::prelude::*;

use crate::components::Transform;
use crate::ecs::{Entity, System, World};

// ─── 컴포넌트 ─────────────────────────────────────────────────────────────────

/// 물리 바디를 가진 엔티티에 붙이는 컴포넌트.
/// `add_dynamic_box` / `add_static_box` 가 반환하는 핸들을 보관한다.
pub struct PhysicsBody {
    pub rigid_body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
}

// ─── 물리 세계 ────────────────────────────────────────────────────────────────

/// rapier2d 2D 물리 시뮬레이션 세계.
///
/// `PhysicsSystem`이 소유하거나 직접 시스템 구조체에 넣어 사용한다.
pub struct PhysicsWorld {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub narrow_phase: NarrowPhase,
    gravity: Vector<f32>,
    integration_params: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
}

impl PhysicsWorld {
    /// `gravity` – 중력 벡터. 화면 좌표(Y+ 아래)에서 `Vec2::new(0.0, 9.8)`.
    pub fn new(gravity: Vec2) -> Self {
        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            narrow_phase: NarrowPhase::new(),
            gravity: vector![gravity.x, gravity.y],
            integration_params: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
        }
    }

    /// 중력에 반응하는 동적 박스 바디를 추가한다.
    ///
    /// - `position`: 중심 좌표 (물리 단위)
    /// - `half_w / half_h`: 폭·높이의 절반 (물리 단위)
    /// - `lock_rotation`: true면 Z축 회전 고정 (캐릭터에 권장)
    pub fn add_dynamic_box(
        &mut self,
        position: Vec2,
        half_w: f32,
        half_h: f32,
        lock_rotation: bool,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let mut builder = RigidBodyBuilder::dynamic().translation(vector![position.x, position.y]);
        if lock_rotation {
            builder = builder.lock_rotations();
        }
        let handle = self.rigid_body_set.insert(builder.build());
        let collider = ColliderBuilder::cuboid(half_w, half_h)
            .friction(0.3)
            .restitution(0.0)
            .build();
        let col_handle =
            self.collider_set
                .insert_with_parent(collider, handle, &mut self.rigid_body_set);
        (handle, col_handle)
    }

    /// 움직이지 않는 정적 바닥·벽·플랫폼을 추가한다.
    pub fn add_static_box(
        &mut self,
        position: Vec2,
        half_w: f32,
        half_h: f32,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let body = RigidBodyBuilder::fixed()
            .translation(vector![position.x, position.y])
            .build();
        let handle = self.rigid_body_set.insert(body);
        let collider = ColliderBuilder::cuboid(half_w, half_h).build();
        let col_handle =
            self.collider_set
                .insert_with_parent(collider, handle, &mut self.rigid_body_set);
        (handle, col_handle)
    }

    /// dt초 만큼 물리 시뮬레이션을 진행한다.
    pub fn step(&mut self, dt: f32) {
        self.integration_params.dt = dt;
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_params,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
    }

    /// 콜라이더가 다른 오브젝트와 접촉 중인지 확인 (착지 판정에 사용).
    pub fn has_contact(&self, col_handle: ColliderHandle) -> bool {
        self.narrow_phase
            .contact_pairs_with(col_handle)
            .any(|pair| pair.has_any_active_contact)
    }
}

// ─── 엔진 기본 물리 시스템 ────────────────────────────────────────────────────

/// 범용 물리 시스템: 매 프레임 step → Transform 동기화.
///
/// 플레이어 제어 등 커스텀 로직이 필요하면 이 타입을 그대로 쓰지 않고,
/// `PhysicsWorld`를 직접 소유하는 전용 시스템을 만드는 것을 권장한다.
/// (예시: `game/main.rs`의 `PlatformerSystem`)
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
            if let Some(body) = self.physics.rigid_body_set.get(handle) {
                let t = *body.translation();
                if let Some(tr) = world.get_mut::<Transform>(entity) {
                    tr.position = Vec2::new(t.x * scale, t.y * scale);
                }
            }
        }
    }
}
