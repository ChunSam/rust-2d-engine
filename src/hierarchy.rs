use glam::{EulerRot, Mat4, Quat, Vec2, Vec3};

use crate::components::Transform;
use crate::ecs::{Entity, System, World};

/// 부모 엔티티를 가리키는 컴포넌트.
///
/// 이 컴포넌트를 가진 엔티티는 `HierarchySystem`에 의해
/// 부모의 월드 변환을 합성한 `GlobalTransform`을 매 프레임 부여받는다.
#[derive(Debug, Clone, Copy)]
pub struct Parent(pub Entity);

/// 자식 엔티티 목록을 보관하는 컴포넌트.
///
/// `attach()` 헬퍼를 통해 `Parent`와 함께 자동으로 관리된다.
#[derive(Debug, Clone)]
pub struct Children(pub Vec<Entity>);

/// 계층 전파 후의 월드 공간 변환.
///
/// `HierarchySystem`이 매 프레임 계산해 덮어쓴다.
/// 렌더러는 이 컴포넌트를 `Transform`보다 우선 사용한다.
/// `Parent`가 없는 루트 엔티티도 `Transform`의 복사본으로 채워진다.
#[derive(Debug, Clone, Copy)]
pub struct GlobalTransform {
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
    pub z: f32,
}

impl GlobalTransform {
    pub fn from_transform(t: &Transform) -> Self {
        Self {
            position: t.position,
            scale: t.scale,
            rotation: t.rotation,
            z: t.z,
        }
    }

    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            Vec3::new(self.scale.x, self.scale.y, 1.0),
            Quat::from_rotation_z(self.rotation),
            Vec3::new(self.position.x, self.position.y, 0.0),
        )
    }
}

/// `child`를 `parent`에 붙인다.
///
/// `Parent` 컴포넌트와 `Children` 목록을 동시에 관리한다.
pub fn attach(world: &mut World, child: Entity, parent: Entity) {
    world.add_component(child, Parent(parent));
    let mut children = world
        .get::<Children>(parent)
        .map(|c| c.0.clone())
        .unwrap_or_default();
    if !children.contains(&child) {
        children.push(child);
    }
    world.add_component(parent, Children(children));
}

/// `child`에서 부모 연결을 끊는다.
///
/// `Parent` 컴포넌트를 제거하고 부모의 `Children` 목록에서도 삭제한다.
pub fn detach(world: &mut World, child: Entity) {
    let parent = match world.get::<Parent>(child).copied() {
        Some(p) => p.0,
        None => return,
    };
    world.remove_component::<Parent>(child);

    let children: Vec<Entity> = world
        .get::<Children>(parent)
        .map(|c| c.0.iter().copied().filter(|&e| e != child).collect())
        .unwrap_or_default();
    world.add_component(parent, Children(children));
}

/// `Transform` 계층을 따라 `GlobalTransform`을 전파하는 시스템.
///
/// `App`이 자동으로 유저 시스템 직후에 실행하므로 별도로 등록할 필요 없다.
///
/// # 깊이 제한
/// 내부적으로 2회 패스를 실행해 최대 깊이 3(루트→자식→손자)을 지원한다.
pub struct HierarchySystem;

impl System for HierarchySystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        // 1단계: 루트 엔티티(Parent 없음) → GlobalTransform = local Transform
        let roots: Vec<(Entity, GlobalTransform)> = world
            .query_opt2::<Transform, Parent>()
            .filter_map(|(e, t, p_opt)| {
                if p_opt.is_none() {
                    Some((e, GlobalTransform::from_transform(t)))
                } else {
                    None
                }
            })
            .collect();
        for (e, gt) in roots {
            world.add_component(e, gt);
        }

        // 2단계: 자식 엔티티 → 부모 GlobalTransform과 합성 (2회 반복으로 깊이 3 지원)
        for _ in 0..2 {
            let children: Vec<(Entity, Entity)> =
                world.query::<Parent>().map(|(e, p)| (e, p.0)).collect();

            let updates: Vec<(Entity, GlobalTransform)> = children
                .into_iter()
                .filter_map(|(child, parent)| {
                    let pgt = world.get::<GlobalTransform>(parent).copied()?;
                    let lt = world.get::<Transform>(child)?;
                    Some((child, compose(&pgt, lt)))
                })
                .collect();

            for (e, gt) in updates {
                world.add_component(e, gt);
            }
        }
    }
}

/// 부모 월드 변환과 자식 로컬 변환을 합성한다.
fn compose(parent: &GlobalTransform, local: &Transform) -> GlobalTransform {
    let world_mat = parent.to_matrix() * local.to_matrix();
    let (scale, rot_quat, translation) = world_mat.to_scale_rotation_translation();
    GlobalTransform {
        position: Vec2::new(translation.x, translation.y),
        scale: Vec2::new(scale.x, scale.y),
        rotation: rot_quat.to_euler(EulerRot::ZYX).0,
        z: parent.z + local.z,
    }
}
