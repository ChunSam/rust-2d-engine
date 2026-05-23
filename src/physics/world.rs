use glam::Vec2;
use rapier2d::prelude::*;

use crate::physics::body::PhysicsBody;

/// rapier2d 2D 물리 시뮬레이션 세계.
///
/// `PhysicsSystem`이 소유하거나 직접 시스템 구조체에 넣어 사용한다.
pub struct PhysicsWorld {
    pub(crate) rigid_body_set: RigidBodySet,
    pub(crate) collider_set: ColliderSet,
    pub(crate) narrow_phase: NarrowPhase,
    gravity: Vector<f32>,
    integration_params: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    pub(crate) island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    pub(crate) impulse_joint_set: ImpulseJointSet,
    pub(crate) multibody_joint_set: MultibodyJointSet,
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

    /// 중력에 반응하는 동적 원형 바디를 추가한다.
    pub fn add_dynamic_circle(
        &mut self,
        position: Vec2,
        radius: f32,
        lock_rotation: bool,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let mut builder =
            RigidBodyBuilder::dynamic().translation(vector![position.x, position.y]);
        if lock_rotation {
            builder = builder.lock_rotations();
        }
        let handle = self.rigid_body_set.insert(builder.build());
        let collider = ColliderBuilder::ball(radius)
            .friction(0.3)
            .restitution(0.0)
            .build();
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

    // ── 타입 안전 접근자 ──────────────────────────────────────────────────────

    /// 핸들로 강체(rigid body)를 불변 참조로 가져온다.
    pub fn rigid_body(&self, handle: RigidBodyHandle) -> Option<&RigidBody> {
        self.rigid_body_set.get(handle)
    }

    /// 핸들로 강체를 가변 참조로 가져온다.
    pub fn rigid_body_mut(&mut self, handle: RigidBodyHandle) -> Option<&mut RigidBody> {
        self.rigid_body_set.get_mut(handle)
    }

    /// 핸들로 rapier 콜라이더를 불변 참조로 가져온다.
    pub fn get_collider(&self, handle: ColliderHandle) -> Option<&Collider> {
        self.collider_set.get(handle)
    }

    /// 핸들로 rapier 콜라이더를 가변 참조로 가져온다.
    pub fn get_collider_mut(&mut self, handle: ColliderHandle) -> Option<&mut Collider> {
        self.collider_set.get_mut(handle)
    }

    /// 바디와 연결된 모든 콜라이더를 제거한 뒤 강체를 삭제한다.
    pub fn remove_body(&mut self, body: &PhysicsBody) {
        self.rigid_body_set.remove(
            body.rigid_body_handle,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            true,
        );
    }
}
