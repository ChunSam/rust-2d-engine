use glam::Vec2;
use std::collections::HashMap;

use crate::components::Transform;
use crate::ecs::{Entity, System, World};

// ─── Collider ────────────────────────────────────────────────────────────────

/// 엔티티에 붙이는 충돌 형태 컴포넌트
#[derive(Debug, Clone, Copy)]
pub enum Collider {
    Circle { radius: f32 },
    Aabb { half_extents: Vec2 },
}

impl Collider {
    /// 콜라이더가 차지하는 AABB(min, max) — 그리드 셀 인덱싱용
    pub fn aabb(&self, center: Vec2) -> (Vec2, Vec2) {
        match self {
            Collider::Circle { radius } => (
                Vec2::new(center.x - radius, center.y - radius),
                Vec2::new(center.x + radius, center.y + radius),
            ),
            Collider::Aabb { half_extents } => (center - *half_extents, center + *half_extents),
        }
    }
}

// ─── CollisionLayer ──────────────────────────────────────────────────────────

/// 비트마스크 형태의 충돌 레이어. AND 결과가 0 이면 충돌 무시.
///
/// 예시:
/// ```rust
/// const LAYER_PLAYER: u32 = 1 << 0;
/// const LAYER_ENEMY:  u32 = 1 << 1;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionLayer(pub u32);

impl CollisionLayer {
    pub const ALL: Self = Self(u32::MAX);
    pub const NONE: Self = Self(0);

    /// 이 레이어와 mask 사이에 공통 비트가 있으면 true
    pub fn matches(&self, mask: CollisionLayer) -> bool {
        (self.0 & mask.0) != 0
    }
}

// ─── SpatialGrid ─────────────────────────────────────────────────────────────

/// 엔티티별 그리드 항목 — 쿼리 시 사용
#[derive(Debug, Clone, Copy)]
pub struct GridEntry {
    pub center: Vec2,
    pub collider: Collider,
    pub layer: CollisionLayer,
}

/// 공간 해시 그리드.
///
/// 매 프레임 `rebuild` 로 갱신하고 `query_radius` / `query_aabb` 로 조회.
/// 기본 셀 크기: 128 픽셀.
#[derive(Debug)]
pub struct SpatialGrid {
    /// 셀 한 변 길이 (픽셀)
    pub cell: f32,
    /// (col, row) → 그 셀에 겹치는 엔티티 목록
    pub buckets: HashMap<(i32, i32), Vec<Entity>>,
    /// 쿼리 시 center/collider/layer 를 다시 읽지 않도록 캐시
    pub entries: HashMap<Entity, GridEntry>,
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell: cell_size,
            buckets: HashMap::new(),
            entries: HashMap::new(),
        }
    }

    /// 내부 상태를 비운다.
    pub fn clear(&mut self) {
        self.buckets.clear();
        self.entries.clear();
    }

    /// World 에서 `(Transform, Collider)` 를 가진 모든 엔티티를 읽어 그리드를 재구성한다.
    ///
    /// `CollisionLayer` 컴포넌트가 없는 엔티티는 `CollisionLayer::ALL` 로 간주한다.
    pub fn rebuild(&mut self, world: &World) {
        self.clear();

        // query2 로 Transform + Collider 를 동시에 가진 엔티티만 순회
        let pairs: Vec<(Entity, Vec2, Collider, CollisionLayer)> = world
            .query2::<Transform, Collider>()
            .map(|(entity, transform, collider)| {
                let center = transform.position;
                let layer = world
                    .get::<CollisionLayer>(entity)
                    .copied()
                    .unwrap_or(CollisionLayer::ALL);
                (entity, center, *collider, layer)
            })
            .collect();

        for (entity, center, collider, layer) in pairs {
            // 셀 인덱스 범위 계산
            let (aabb_min, aabb_max) = collider.aabb(center);
            let col_min = (aabb_min.x / self.cell).floor() as i32;
            let col_max = (aabb_max.x / self.cell).floor() as i32;
            let row_min = (aabb_min.y / self.cell).floor() as i32;
            let row_max = (aabb_max.y / self.cell).floor() as i32;

            for col in col_min..=col_max {
                for row in row_min..=row_max {
                    self.buckets.entry((col, row)).or_default().push(entity);
                }
            }

            self.entries.insert(
                entity,
                GridEntry {
                    center,
                    collider,
                    layer,
                },
            );
        }
    }

    /// 셀 좌표에서 해당 bucket 을 꺼낸다.
    fn cell_key(&self, x: f32, y: f32) -> (i32, i32) {
        (
            (x / self.cell).floor() as i32,
            (y / self.cell).floor() as i32,
        )
    }

    /// AABB 범위에 해당하는 모든 셀에서 중복 없는 후보 엔티티를 수집한다.
    pub(crate) fn candidates_in_aabb(&self, min: Vec2, max: Vec2) -> Vec<Entity> {
        let (col_min, row_min) = self.cell_key(min.x, min.y);
        let (col_max, row_max) = self.cell_key(max.x, max.y);

        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();

        for col in col_min..=col_max {
            for row in row_min..=row_max {
                if let Some(bucket) = self.buckets.get(&(col, row)) {
                    for &entity in bucket {
                        if seen.insert(entity) {
                            result.push(entity);
                        }
                    }
                }
            }
        }
        result
    }
}

// ─── CollisionGridSystem ──────────────────────────────────────────────────────

/// 매 프레임 `SpatialGrid` 를 rebuild 하는 시스템.
///
/// `PhysicsSystem` 이 `PhysicsWorld` 를 직접 소유하듯, 이 시스템이 `SpatialGrid`
/// 를 소유한다. borrow checker 충돌을 피하기 위해 ECS 리소스로 넣지 않는다.
///
/// ```ignore
/// app.add_system(CollisionGridSystem::new(128.0));
/// // 이후 시스템에서 SpatialGrid 를 직접 참조하고 싶으면 CollisionGridSystem 에서 꺼낸다.
/// ```
pub struct CollisionGridSystem {
    pub grid: SpatialGrid,
}

impl CollisionGridSystem {
    pub fn new(cell_size: f32) -> Self {
        Self {
            grid: SpatialGrid::new(cell_size),
        }
    }
}

impl System for CollisionGridSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        self.grid.rebuild(world);
    }
}

// ─── 단위 테스트 ──────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::World;

    fn make_world_with_circle(pos: Vec2, radius: f32) -> (World, Entity) {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(
            e,
            Transform {
                position: pos,
                scale: Vec2::ONE,
                rotation: 0.0,
                z: 0.0,
            },
        );
        world.add_component(e, Collider::Circle { radius });
        (world, e)
    }

    /// 빈 grid 의 query_radius 는 빈 Vec 을 반환해야 한다.
    #[test]
    fn empty_grid_returns_empty_query() {
        let grid = SpatialGrid::new(128.0);
        let result = grid.query_radius(Vec2::ZERO, 100.0, CollisionLayer::ALL);
        assert!(result.is_empty());
    }

    /// 반경 내에 있는 단일 Circle 콜라이더를 검출해야 한다.
    #[test]
    fn single_circle_in_radius() {
        let (world, e) = make_world_with_circle(Vec2::new(50.0, 50.0), 16.0);
        let mut grid = SpatialGrid::new(128.0);
        grid.rebuild(&world);

        // 쿼리 중심에서 거리 = 0, 반경 200 → 반드시 검출
        let result = grid.query_radius(Vec2::new(50.0, 50.0), 200.0, CollisionLayer::ALL);
        assert!(result.contains(&e), "엔티티가 반경 내에 있어야 함");
    }

    /// 마스크와 레이어가 AND = 0 이면 엔티티가 제외되어야 한다.
    #[test]
    fn layer_mask_filters_results() {
        let mut world = World::new();

        // LAYER_A 엔티티
        let e_a = world.spawn();
        world.add_component(
            e_a,
            Transform {
                position: Vec2::new(10.0, 10.0),
                scale: Vec2::ONE,
                rotation: 0.0,
                z: 0.0,
            },
        );
        world.add_component(e_a, Collider::Circle { radius: 8.0 });
        world.add_component(e_a, CollisionLayer(1 << 0)); // bit 0

        // LAYER_B 엔티티
        let e_b = world.spawn();
        world.add_component(
            e_b,
            Transform {
                position: Vec2::new(20.0, 10.0),
                scale: Vec2::ONE,
                rotation: 0.0,
                z: 0.0,
            },
        );
        world.add_component(e_b, Collider::Circle { radius: 8.0 });
        world.add_component(e_b, CollisionLayer(1 << 1)); // bit 1

        let mut grid = SpatialGrid::new(128.0);
        grid.rebuild(&world);

        // bit 0 마스크 → e_a 만 검출, e_b 제외
        let result = grid.query_radius(Vec2::ZERO, 500.0, CollisionLayer(1 << 0));
        assert!(result.contains(&e_a), "e_a 는 마스크와 일치해야 함");
        assert!(!result.contains(&e_b), "e_b 는 마스크와 불일치해야 함");
    }

    /// despawn 후 rebuild 하면 결과에서 사라져야 한다.
    #[test]
    fn rebuild_after_despawn() {
        let (mut world, e) = make_world_with_circle(Vec2::new(30.0, 30.0), 10.0);
        let mut grid = SpatialGrid::new(128.0);

        grid.rebuild(&world);
        assert!(!grid
            .query_radius(Vec2::ZERO, 500.0, CollisionLayer::ALL)
            .is_empty());

        // despawn 후 rebuild
        world.despawn(e);
        grid.rebuild(&world);

        let result = grid.query_radius(Vec2::ZERO, 500.0, CollisionLayer::ALL);
        assert!(result.is_empty(), "despawn 된 엔티티는 결과에 없어야 함");
    }
}
