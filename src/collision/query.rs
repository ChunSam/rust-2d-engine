use glam::Vec2;

use super::grid::{Collider, CollisionLayer, SpatialGrid};
use crate::ecs::Entity;

impl SpatialGrid {
    /// 원 영역 내의 모든 엔티티를 반환한다.
    ///
    /// - `mask` 와 엔티티 레이어의 AND 가 0 이면 제외.
    /// - Circle 콜라이더: 중심 간 거리 ≤ radius + collider.radius
    /// - Aabb 콜라이더: 쿼리 원과 AABB 의 교차 여부
    pub fn query_radius(&self, center: Vec2, radius: f32, mask: CollisionLayer) -> Vec<Entity> {
        // 쿼리 원을 감싸는 AABB 로 후보 셀 범위를 좁힌다
        let search_min = Vec2::new(center.x - radius, center.y - radius);
        let search_max = Vec2::new(center.x + radius, center.y + radius);

        let candidates = self.candidates_in_aabb(search_min, search_max);
        let mut result = Vec::new();

        for entity in candidates {
            let entry = match self.entries.get(&entity) {
                Some(e) => e,
                None => continue,
            };

            // 레이어 마스크 확인
            if !mask.matches(entry.layer) {
                continue;
            }

            // 실제 거리/교차 판정
            if circle_hits_collider(center, radius, entry.center, entry.collider) {
                result.push(entity);
            }
        }

        result
    }

    /// AABB 영역과 겹치는 모든 엔티티를 반환한다.
    ///
    /// - `mask` 와 엔티티 레이어의 AND 가 0 이면 제외.
    pub fn query_aabb(&self, min: Vec2, max: Vec2, mask: CollisionLayer) -> Vec<Entity> {
        let candidates = self.candidates_in_aabb(min, max);
        let mut result = Vec::new();

        for entity in candidates {
            let entry = match self.entries.get(&entity) {
                Some(e) => e,
                None => continue,
            };

            if !mask.matches(entry.layer) {
                continue;
            }

            if aabb_hits_collider(min, max, entry.center, entry.collider) {
                result.push(entity);
            }
        }

        result
    }
}

// ─── 내부 교차 판정 헬퍼 ──────────────────────────────────────────────────────

/// 쿼리 원(center, radius) 이 collider 와 겹치는지 판정한다.
fn circle_hits_collider(
    query_center: Vec2,
    query_radius: f32,
    entity_center: Vec2,
    collider: Collider,
) -> bool {
    match collider {
        Collider::Circle { radius } => {
            let dist = (query_center - entity_center).length();
            dist <= query_radius + radius
        }
        Collider::Aabb { half_extents } => {
            // 원과 AABB 교차: 원 중심에서 AABB 까지의 최소 거리 ≤ 반경
            let aabb_min = entity_center - half_extents;
            let aabb_max = entity_center + half_extents;
            let closest = Vec2::new(
                query_center.x.clamp(aabb_min.x, aabb_max.x),
                query_center.y.clamp(aabb_min.y, aabb_max.y),
            );
            (query_center - closest).length() <= query_radius
        }
    }
}

/// 쿼리 AABB(min, max) 가 collider 와 겹치는지 판정한다.
fn aabb_hits_collider(
    query_min: Vec2,
    query_max: Vec2,
    entity_center: Vec2,
    collider: Collider,
) -> bool {
    // collider 의 AABB 를 구하고 두 AABB 가 겹치는지 확인
    let (col_min, col_max) = collider.aabb(entity_center);

    // AABB vs AABB 교차: 두 축 모두 겹쳐야 한다
    let overlap_x = query_min.x <= col_max.x && query_max.x >= col_min.x;
    let overlap_y = query_min.y <= col_max.y && query_max.y >= col_min.y;
    overlap_x && overlap_y
}
