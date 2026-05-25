use glam::Vec2;
use rapier2d::prelude::*;

use crate::physics::body::PhysicsBody;
use crate::physics::character::CharacterController;

// ── 충돌 그룹 ────────────────────────────────────────────────────────────────

/// Rapier `InteractionGroups`를 감싼 엔진용 충돌 레이어/마스크.
///
/// `memberships`는 이 콜라이더가 속한 레이어 비트, `filter`는 상호작용을 허용할
/// 상대 레이어 비트다. 두 콜라이더가 모두 서로를 허용해야 충돌/센서 교차가 발생한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionGroups {
    pub memberships: u32,
    pub filter: u32,
}

impl CollisionGroups {
    pub const ALL_BITS: u32 = u32::MAX;
    pub const NONE_BITS: u32 = 0;

    pub const fn new(memberships: u32, filter: u32) -> Self {
        Self {
            memberships,
            filter,
        }
    }

    pub const fn all() -> Self {
        Self::new(Self::ALL_BITS, Self::ALL_BITS)
    }

    pub const fn none() -> Self {
        Self::new(Self::NONE_BITS, Self::NONE_BITS)
    }

    pub const fn layer(bit_index: u8) -> Self {
        let bit = 1u32 << bit_index;
        Self::new(bit, Self::ALL_BITS)
    }

    pub const fn with_filter(mut self, filter: u32) -> Self {
        self.filter = filter;
        self
    }

    fn to_rapier(self) -> InteractionGroups {
        InteractionGroups::new(Group::from(self.memberships), Group::from(self.filter))
    }

    fn from_rapier(groups: InteractionGroups) -> Self {
        Self::new(groups.memberships.bits(), groups.filter.bits())
    }
}

impl Default for CollisionGroups {
    fn default() -> Self {
        Self::all()
    }
}

// ── 레이캐스트 결과 ──────────────────────────────────────────────────────────

/// 레이캐스트 충돌 결과.
#[derive(Debug, Clone, Copy)]
pub struct RaycastHit {
    /// 충돌한 콜라이더 핸들.
    pub collider_handle: ColliderHandle,
    /// 월드 공간 충돌 지점 (물리 단위).
    pub point: Vec2,
    /// 충돌 면의 법선 벡터 (정규화됨).
    pub normal: Vec2,
    /// 레이 시작점으로부터의 거리 배율 (`origin + direction * toi` = 충돌 지점).
    pub toi: f32,
}

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
        self.add_dynamic_box_with_groups(
            position,
            half_w,
            half_h,
            lock_rotation,
            CollisionGroups::all(),
        )
    }

    /// 충돌 그룹을 지정해 중력에 반응하는 동적 박스 바디를 추가한다.
    pub fn add_dynamic_box_with_groups(
        &mut self,
        position: Vec2,
        half_w: f32,
        half_h: f32,
        lock_rotation: bool,
        groups: CollisionGroups,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let mut builder = RigidBodyBuilder::dynamic().translation(vector![position.x, position.y]);
        if lock_rotation {
            builder = builder.lock_rotations();
        }
        let handle = self.rigid_body_set.insert(builder.build());
        let collider = ColliderBuilder::cuboid(half_w, half_h)
            .friction(0.3)
            .restitution(0.0)
            .collision_groups(groups.to_rapier())
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
        self.add_static_box_with_groups(position, half_w, half_h, CollisionGroups::all())
    }

    /// 충돌 그룹을 지정해 움직이지 않는 정적 박스 바디를 추가한다.
    pub fn add_static_box_with_groups(
        &mut self,
        position: Vec2,
        half_w: f32,
        half_h: f32,
        groups: CollisionGroups,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let body = RigidBodyBuilder::fixed()
            .translation(vector![position.x, position.y])
            .build();
        let handle = self.rigid_body_set.insert(body);
        let collider = ColliderBuilder::cuboid(half_w, half_h)
            .collision_groups(groups.to_rapier())
            .build();
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
        self.add_dynamic_circle_with_groups(position, radius, lock_rotation, CollisionGroups::all())
    }

    /// 충돌 그룹을 지정해 중력에 반응하는 동적 원형 바디를 추가한다.
    pub fn add_dynamic_circle_with_groups(
        &mut self,
        position: Vec2,
        radius: f32,
        lock_rotation: bool,
        groups: CollisionGroups,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let mut builder = RigidBodyBuilder::dynamic().translation(vector![position.x, position.y]);
        if lock_rotation {
            builder = builder.lock_rotations();
        }
        let handle = self.rigid_body_set.insert(builder.build());
        let collider = ColliderBuilder::ball(radius)
            .friction(0.3)
            .restitution(0.0)
            .collision_groups(groups.to_rapier())
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

    /// 콜라이더의 충돌 그룹을 변경한다. 핸들이 없으면 `false`를 반환한다.
    pub fn set_collision_groups(
        &mut self,
        handle: ColliderHandle,
        groups: CollisionGroups,
    ) -> bool {
        let Some(collider) = self.collider_set.get_mut(handle) else {
            return false;
        };
        collider.set_collision_groups(groups.to_rapier());
        true
    }

    /// 콜라이더의 현재 충돌 그룹을 반환한다.
    pub fn collision_groups(&self, handle: ColliderHandle) -> Option<CollisionGroups> {
        self.collider_set
            .get(handle)
            .map(|collider| CollisionGroups::from_rapier(collider.collision_groups()))
    }

    /// 키네마틱 박스 바디를 추가한다 (중력 비반응, 수동 위치 제어).
    pub fn add_kinematic_box(
        &mut self,
        position: Vec2,
        half_w: f32,
        half_h: f32,
    ) -> (RigidBodyHandle, ColliderHandle) {
        self.add_kinematic_box_with_groups(position, half_w, half_h, CollisionGroups::all())
    }

    /// 충돌 그룹을 지정해 키네마틱 박스 바디를 추가한다.
    pub fn add_kinematic_box_with_groups(
        &mut self,
        position: Vec2,
        half_w: f32,
        half_h: f32,
        groups: CollisionGroups,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let body = RigidBodyBuilder::kinematic_position_based()
            .translation(vector![position.x, position.y])
            .build();
        let handle = self.rigid_body_set.insert(body);
        let collider = ColliderBuilder::cuboid(half_w, half_h)
            .collision_groups(groups.to_rapier())
            .build();
        let col_handle =
            self.collider_set
                .insert_with_parent(collider, handle, &mut self.rigid_body_set);
        (handle, col_handle)
    }

    /// 키네마틱 원형 바디를 추가한다 (중력 비반응, 수동 위치 제어).
    pub fn add_kinematic_circle(
        &mut self,
        position: Vec2,
        radius: f32,
    ) -> (RigidBodyHandle, ColliderHandle) {
        self.add_kinematic_circle_with_groups(position, radius, CollisionGroups::all())
    }

    /// 충돌 그룹을 지정해 키네마틱 원형 바디를 추가한다.
    pub fn add_kinematic_circle_with_groups(
        &mut self,
        position: Vec2,
        radius: f32,
        groups: CollisionGroups,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let body = RigidBodyBuilder::kinematic_position_based()
            .translation(vector![position.x, position.y])
            .build();
        let handle = self.rigid_body_set.insert(body);
        let collider = ColliderBuilder::ball(radius)
            .collision_groups(groups.to_rapier())
            .build();
        let col_handle =
            self.collider_set
                .insert_with_parent(collider, handle, &mut self.rigid_body_set);
        (handle, col_handle)
    }

    /// 물리 반응 없이 교차만 감지하는 정적 박스 센서를 추가한다.
    pub fn add_sensor_box(
        &mut self,
        position: Vec2,
        half_w: f32,
        half_h: f32,
    ) -> (RigidBodyHandle, ColliderHandle) {
        self.add_sensor_box_with_groups(position, half_w, half_h, CollisionGroups::all())
    }

    /// 충돌 그룹을 지정해 정적 박스 센서를 추가한다.
    pub fn add_sensor_box_with_groups(
        &mut self,
        position: Vec2,
        half_w: f32,
        half_h: f32,
        groups: CollisionGroups,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let body = RigidBodyBuilder::fixed()
            .translation(vector![position.x, position.y])
            .build();
        let handle = self.rigid_body_set.insert(body);
        let collider = ColliderBuilder::cuboid(half_w, half_h)
            .sensor(true)
            .collision_groups(groups.to_rapier())
            .build();
        let col_handle =
            self.collider_set
                .insert_with_parent(collider, handle, &mut self.rigid_body_set);
        (handle, col_handle)
    }

    /// 물리 반응 없이 교차만 감지하는 정적 원형 센서를 추가한다.
    pub fn add_sensor_circle(
        &mut self,
        position: Vec2,
        radius: f32,
    ) -> (RigidBodyHandle, ColliderHandle) {
        self.add_sensor_circle_with_groups(position, radius, CollisionGroups::all())
    }

    /// 충돌 그룹을 지정해 정적 원형 센서를 추가한다.
    pub fn add_sensor_circle_with_groups(
        &mut self,
        position: Vec2,
        radius: f32,
        groups: CollisionGroups,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let body = RigidBodyBuilder::fixed()
            .translation(vector![position.x, position.y])
            .build();
        let handle = self.rigid_body_set.insert(body);
        let collider = ColliderBuilder::ball(radius)
            .sensor(true)
            .collision_groups(groups.to_rapier())
            .build();
        let col_handle =
            self.collider_set
                .insert_with_parent(collider, handle, &mut self.rigid_body_set);
        (handle, col_handle)
    }

    // ── 레이캐스트 ─────────────────────────────────────────────────────────────

    /// 단순 레이캐스트. 최초 충돌 콜라이더 핸들과 toi(레이 이동 거리 배율)를 반환한다.
    ///
    /// - `origin` / `direction` — 물리 단위 (픽셀 ÷ pixels_per_unit).
    /// - `max_toi` — 최대 레이 길이 배율 (보통 최대 거리 / direction.length()).
    /// - `solid` — `true`이면 레이 시작점이 콜라이더 내부일 때도 교차로 처리.
    pub fn cast_ray(
        &self,
        origin: Vec2,
        direction: Vec2,
        max_toi: f32,
        solid: bool,
    ) -> Option<(ColliderHandle, f32)> {
        let ray = Ray::new(
            point![origin.x, origin.y],
            vector![direction.x, direction.y],
        );
        self.query_pipeline.cast_ray(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            max_toi,
            solid,
            QueryFilter::default(),
        )
    }

    /// 레이캐스트 — 충돌 지점과 법선 벡터를 포함한 `RaycastHit`를 반환한다.
    ///
    /// 물리 단위 기준. 픽셀 단위를 쓰려면 `origin`과 `direction`을 `pixels_per_unit`으로 나눠 전달하고,
    /// 반환된 `RaycastHit::point`에 `pixels_per_unit`을 곱해 변환한다.
    pub fn cast_ray_with_normal(
        &self,
        origin: Vec2,
        direction: Vec2,
        max_toi: f32,
        solid: bool,
    ) -> Option<RaycastHit> {
        let ray = Ray::new(
            point![origin.x, origin.y],
            vector![direction.x, direction.y],
        );
        self.query_pipeline
            .cast_ray_and_get_normal(
                &self.rigid_body_set,
                &self.collider_set,
                &ray,
                max_toi,
                solid,
                QueryFilter::default(),
            )
            .map(|(handle, intersection)| {
                let hit_point = ray.point_at(intersection.time_of_impact);
                RaycastHit {
                    collider_handle: handle,
                    point: Vec2::new(hit_point.x, hit_point.y),
                    normal: Vec2::new(intersection.normal.x, intersection.normal.y),
                    toi: intersection.time_of_impact,
                }
            })
    }

    // ── 캐릭터 컨트롤러 ────────────────────────────────────────────────────────

    /// `CharacterController`를 이용해 충돌 해결 후 키네마틱 바디를 이동한다.
    ///
    /// `desired_translation` — **픽셀 단위** 이동 벡터.
    /// 내부에서 `pixels_per_unit`으로 물리 단위로 변환하고,
    /// 충돌 해결 후 `set_next_kinematic_translation()`으로 바디에 적용한다.
    /// 다음 `step()` 호출 시 해당 위치로 이동한다.
    ///
    /// `controller.grounded`가 갱신되므로 이 메서드를 `PhysicsSystem::run()` 이전에 호출해야 한다.
    pub fn move_character(
        &mut self,
        controller: &mut CharacterController,
        body_handle: RigidBodyHandle,
        col_handle: ColliderHandle,
        desired_translation: Vec2,
        dt: f32,
        pixels_per_unit: f32,
    ) {
        let ppu = pixels_per_unit;
        let desired = vector![desired_translation.x / ppu, desired_translation.y / ppu];

        // 콜라이더 위치와 shape를 먼저 복사해 borrow 분리
        let (col_pos, shape_type) = match self.collider_set.get(col_handle) {
            Some(c) => (*c.position(), c.shape().shape_type()),
            None => return,
        };

        // shape를 collider_set에서 재획득 (두 번째 불변 참조 — Rust 허용)
        let shape = match self.collider_set.get(col_handle) {
            Some(c) => c.shape(),
            None => return,
        };
        let _ = shape_type; // 타입 힌트용으로 저장, 실제 사용은 shape 참조

        let output = controller.inner.move_shape(
            dt,
            &self.rigid_body_set,
            &self.collider_set,
            &self.query_pipeline,
            shape,
            &col_pos,
            desired,
            QueryFilter::default().exclude_collider(col_handle),
            |_| {},
        );

        controller.grounded = output.grounded;

        // 바디 현재 위치 + 이동 벡터로 next_kinematic_translation 설정
        let body_t = self
            .rigid_body_set
            .get(body_handle)
            .map(|b| *b.translation())
            .unwrap_or_default();
        let new_t = body_t + output.translation;
        if let Some(body) = self.rigid_body_set.get_mut(body_handle) {
            body.set_next_kinematic_translation(new_t);
        }
    }

    // ── 조인트 ────────────────────────────────────────────────────────────────

    /// 두 바디 사이에 DistanceJoint를 생성한다.
    /// `anchor1/2` — 각 바디 로컬 공간의 연결점 (월드 단위).
    /// 내부적으로 `SpringJointBuilder`(stiffness=1000, damping=10)를 사용해 고정 거리를 유지한다.
    pub fn add_distance_joint(
        &mut self,
        body1: RigidBodyHandle,
        body2: RigidBodyHandle,
        anchor1: Vec2,
        anchor2: Vec2,
        rest_length: f32,
    ) -> ImpulseJointHandle {
        let data = SpringJointBuilder::new(rest_length, 1000.0, 10.0)
            .local_anchor1(point![anchor1.x, anchor1.y])
            .local_anchor2(point![anchor2.x, anchor2.y])
            .build();
        self.impulse_joint_set.insert(body1, body2, data, true)
    }

    /// RevoluteJoint (힌지) — 두 바디가 공통 피벗점을 기준으로 자유 회전.
    pub fn add_revolute_joint(
        &mut self,
        body1: RigidBodyHandle,
        body2: RigidBodyHandle,
        anchor1: Vec2,
        anchor2: Vec2,
    ) -> ImpulseJointHandle {
        let data = RevoluteJointBuilder::new()
            .local_anchor1(point![anchor1.x, anchor1.y])
            .local_anchor2(point![anchor2.x, anchor2.y])
            .build();
        self.impulse_joint_set.insert(body1, body2, data, true)
    }

    /// PrismaticJoint (슬라이더) — 특정 축 방향으로만 상대 이동 허용.
    pub fn add_prismatic_joint(
        &mut self,
        body1: RigidBodyHandle,
        body2: RigidBodyHandle,
        anchor1: Vec2,
        anchor2: Vec2,
        axis: Vec2,
    ) -> ImpulseJointHandle {
        let unit_axis = UnitVector::new_normalize(vector![axis.x, axis.y]);
        let data = PrismaticJointBuilder::new(unit_axis)
            .local_anchor1(point![anchor1.x, anchor1.y])
            .local_anchor2(point![anchor2.x, anchor2.y])
            .build();
        self.impulse_joint_set.insert(body1, body2, data, true)
    }

    /// 조인트를 제거한다.
    pub fn remove_joint(&mut self, handle: ImpulseJointHandle) {
        self.impulse_joint_set.remove(handle, true);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::character::CharacterController;

    fn make_world() -> PhysicsWorld {
        PhysicsWorld::new(Vec2::new(0.0, 9.8))
    }

    #[test]
    fn collision_groups_filter_contacts() {
        let mut pw = PhysicsWorld::new(Vec2::ZERO);
        let player = 1 << 0;
        let enemy = 1 << 1;
        let pickup = 1 << 2;

        let (_, player_col) = pw.add_dynamic_box_with_groups(
            Vec2::ZERO,
            0.5,
            0.5,
            false,
            CollisionGroups::new(player, enemy),
        );
        let (_, enemy_col) = pw.add_static_box_with_groups(
            Vec2::ZERO,
            0.5,
            0.5,
            CollisionGroups::new(enemy, player),
        );
        let (_, pickup_col) = pw.add_static_box_with_groups(
            Vec2::ZERO,
            0.5,
            0.5,
            CollisionGroups::new(pickup, pickup),
        );

        pw.step(1.0 / 60.0);

        assert!(pw.has_contact(player_col));
        assert!(pw.has_contact(enemy_col));
        assert!(
            !pw.has_contact(pickup_col),
            "pickup layer should be ignored by player/enemy filters"
        );
    }

    #[test]
    fn set_collision_groups_updates_existing_collider() {
        let mut pw = PhysicsWorld::new(Vec2::ZERO);
        let (_, col) = pw.add_dynamic_circle(Vec2::ZERO, 0.5, false);
        let groups = CollisionGroups::new(1 << 3, 1 << 4);

        assert!(pw.set_collision_groups(col, groups));
        assert_eq!(pw.collision_groups(col), Some(groups));
    }

    #[test]
    fn cast_ray_hits_static_box() {
        let mut pw = make_world();
        // Y=0 에 두께 1 바닥
        pw.add_static_box(Vec2::new(0.0, 0.0), 5.0, 0.5);
        pw.step(1.0 / 60.0); // query_pipeline 갱신

        // Y=-5 에서 아래(+Y)로 레이캐스트
        let result = pw.cast_ray(Vec2::new(0.0, -5.0), Vec2::new(0.0, 1.0), 10.0, true);
        assert!(result.is_some(), "바닥에 레이가 맞아야 함");
        let (_, toi) = result.unwrap();
        assert!(toi > 0.0 && toi < 10.0, "toi 범위 확인: {toi}");
    }

    #[test]
    fn cast_ray_misses_when_no_obstacle() {
        let mut pw = make_world();
        pw.add_static_box(Vec2::new(100.0, 0.0), 5.0, 0.5); // 멀리 있음
        pw.step(1.0 / 60.0);

        // X 방향으로 레이캐스트 — 바닥이 Y 방향에 있으므로 맞지 않음
        let result = pw.cast_ray(Vec2::new(0.0, -5.0), Vec2::new(0.0, -1.0), 5.0, true);
        assert!(result.is_none(), "반대 방향은 맞지 않아야 함");
    }

    #[test]
    fn cast_ray_with_normal_returns_correct_normal() {
        let mut pw = make_world();
        pw.add_static_box(Vec2::new(0.0, 0.0), 5.0, 0.5);
        pw.step(1.0 / 60.0);

        let hit = pw.cast_ray_with_normal(Vec2::new(0.0, -5.0), Vec2::new(0.0, 1.0), 20.0, true);
        assert!(hit.is_some());
        let h = hit.unwrap();
        // 위에서 아래로 쐈으므로 법선은 위쪽 (Y < 0 in physics coords)
        assert!(
            h.normal.y < 0.0,
            "법선은 레이 반대 방향이어야 함: {:?}",
            h.normal
        );
    }

    #[test]
    fn add_kinematic_box_creates_body() {
        let mut pw = make_world();
        let (rb, col) = pw.add_kinematic_box(Vec2::new(1.0, 2.0), 0.5, 1.0);
        assert!(pw.rigid_body(rb).is_some());
        assert!(pw.get_collider(col).is_some());
        let body = pw.rigid_body(rb).unwrap();
        assert!(body.is_kinematic(), "키네마틱 바디여야 함");
    }

    #[test]
    fn add_kinematic_circle_creates_body() {
        let mut pw = make_world();
        let (rb, _col) = pw.add_kinematic_circle(Vec2::new(0.0, 0.0), 0.5);
        let body = pw.rigid_body(rb).unwrap();
        assert!(body.is_kinematic());
    }

    #[test]
    fn move_character_grounded_on_floor() {
        let mut pw = make_world();
        // 바닥: Y=2.0, half_h=0.5 → 상단이 Y=1.5
        pw.add_static_box(Vec2::new(0.0, 2.0), 5.0, 0.5);
        // 캐릭터: Y=0.0, half_h=0.5 → 하단이 Y=0.5 (바닥과 1.0 떨어짐)
        let (rb, col) = pw.add_kinematic_box(Vec2::new(0.0, 0.0), 0.4, 0.5);
        pw.step(1.0 / 60.0);

        let mut ctrl = CharacterController::new();
        // 아래로 이동 시도 (픽셀 단위, ppu=1)
        pw.move_character(
            &mut ctrl,
            rb,
            col,
            Vec2::new(0.0, 5.0), // 아래로 이동
            1.0 / 60.0,
            1.0,
        );
        pw.step(1.0 / 60.0);

        assert!(ctrl.grounded, "바닥에 닿으면 grounded=true여야 함");
    }

    #[test]
    fn character_controller_builder_methods() {
        let ctrl = CharacterController::new()
            .with_max_slope_deg(30.0)
            .with_autostep(0.5, 0.2)
            .with_snap_to_ground(0.2);
        assert!((ctrl.max_slope_angle - 30_f32.to_radians()).abs() < 1e-5);
    }

    #[test]
    fn add_distance_joint_creates_and_removes() {
        let mut pw = make_world();
        let (b1, _) = pw.add_dynamic_box(Vec2::new(-1.0, 0.0), 0.4, 0.4, false);
        let (b2, _) = pw.add_dynamic_box(Vec2::new(1.0, 0.0), 0.4, 0.4, false);
        let h = pw.add_distance_joint(b1, b2, Vec2::ZERO, Vec2::ZERO, 2.0);
        assert!(pw.impulse_joint_set.get(h).is_some());
        pw.remove_joint(h);
        assert!(pw.impulse_joint_set.get(h).is_none());
    }

    #[test]
    fn add_revolute_joint_creates() {
        let mut pw = make_world();
        let (b1, _) = pw.add_dynamic_box(Vec2::new(0.0, 0.0), 0.4, 0.4, false);
        let (b2, _) = pw.add_dynamic_box(Vec2::new(1.0, 0.0), 0.4, 0.4, false);
        let h = pw.add_revolute_joint(b1, b2, Vec2::new(0.5, 0.0), Vec2::new(-0.5, 0.0));
        assert!(pw.impulse_joint_set.get(h).is_some());
    }

    #[test]
    fn add_prismatic_joint_creates() {
        let mut pw = make_world();
        let (b1, _) = pw.add_dynamic_box(Vec2::new(0.0, 0.0), 0.4, 0.4, false);
        let (b2, _) = pw.add_dynamic_box(Vec2::new(0.0, 1.0), 0.4, 0.4, false);
        let h = pw.add_prismatic_joint(b1, b2, Vec2::ZERO, Vec2::ZERO, Vec2::new(0.0, 1.0));
        assert!(pw.impulse_joint_set.get(h).is_some());
    }
}
