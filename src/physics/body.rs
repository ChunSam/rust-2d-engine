use rapier2d::prelude::*;

/// 물리 바디를 가진 엔티티에 붙이는 컴포넌트.
/// `PhysicsWorld::add_dynamic_box` / `add_static_box` 가 반환하는 핸들을 보관한다.
pub struct PhysicsBody {
    pub rigid_body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
}
