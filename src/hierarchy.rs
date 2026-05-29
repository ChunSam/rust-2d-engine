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
/// # 깊이
/// 위상 정렬(루트→자식 순)로 단일 패스 전파하므로 **임의 깊이**의 계층을 지원한다.
/// (스켈레탈 애니메이션처럼 hip→torso→upper_arm→forearm→hand 같은 깊은 본 체인도 동작.)
pub struct HierarchySystem;

impl System for HierarchySystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        // Transform을 가진 모든 엔티티를 루트→자식 순으로 위상 정렬한다.
        // 부모가 항상 자식보다 먼저 처리되므로 한 번의 패스로 임의 깊이를 전파할 수 있다.
        let all: Vec<Entity> = world.query::<Transform>().map(|(e, _)| e).collect();
        let ordered = crate::prefab::topological_sort_entities(&all, world);

        for entity in ordered {
            let gt = match world.get::<Parent>(entity).map(|p| p.0) {
                // 부모가 있으면 부모의 GlobalTransform(이미 계산됨)과 합성
                Some(parent) => {
                    match (
                        world.get::<GlobalTransform>(parent).copied(),
                        world.get::<Transform>(entity),
                    ) {
                        (Some(pgt), Some(lt)) => compose(&pgt, lt),
                        // 부모에 Transform/GlobalTransform이 없으면 로컬을 그대로 사용
                        _ => match world.get::<Transform>(entity) {
                            Some(lt) => GlobalTransform::from_transform(lt),
                            None => continue,
                        },
                    }
                }
                // 루트: 로컬 Transform 복사
                None => match world.get::<Transform>(entity) {
                    Some(lt) => GlobalTransform::from_transform(lt),
                    None => continue,
                },
            };
            world.add_component(entity, gt);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::World;

    fn pos(world: &World, e: Entity) -> Vec2 {
        world.get::<GlobalTransform>(e).unwrap().position
    }

    #[test]
    fn propagates_arbitrary_depth_chain() {
        // hip→torso→upper_arm→forearm→hand : 깊이 5 (기존 2패스로는 불가능했던 깊이)
        let mut world = World::new();
        let names = ["hip", "torso", "upper_arm", "forearm", "hand"];
        let mut prev: Option<Entity> = None;
        let mut bones = Vec::new();
        for _ in names {
            let e = world.spawn();
            // 각 본은 부모 기준 +10 만큼 오른쪽으로, 회전/스케일 없음
            world.add_component(
                e,
                Transform {
                    position: Vec2::new(10.0, 0.0),
                    scale: Vec2::ONE,
                    rotation: 0.0,
                    z: 0.0,
                },
            );
            if let Some(p) = prev {
                attach(&mut world, e, p);
            }
            prev = Some(e);
            bones.push(e);
        }

        HierarchySystem.run(&mut world, 0.0);

        // 누적 위치: 본 i는 (i+1)*10
        for (i, &e) in bones.iter().enumerate() {
            assert!(
                (pos(&world, e).x - (i as f32 + 1.0) * 10.0).abs() < 1e-3,
                "bone {i} expected x={}, got {}",
                (i as f32 + 1.0) * 10.0,
                pos(&world, e).x
            );
        }
    }

    #[test]
    fn root_without_parent_copies_local() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(
            e,
            Transform {
                position: Vec2::new(5.0, 7.0),
                scale: Vec2::ONE,
                rotation: 0.0,
                z: 2.0,
            },
        );
        HierarchySystem.run(&mut world, 0.0);
        let gt = world.get::<GlobalTransform>(e).unwrap();
        assert_eq!(gt.position, Vec2::new(5.0, 7.0));
        assert_eq!(gt.z, 2.0);
    }
}
