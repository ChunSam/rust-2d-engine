use crate::collision::grid::SpatialGrid;
use crate::ecs::{System, World};
use crate::resources::{DebugDrawQueue, DebugRect};

// ─── 리소스 ──────────────────────────────────────────────────────────────────

/// 디버그 렌더링 설정 리소스.
///
/// `world.insert_resource(DebugConfig { show_colliders: true })` 로 활성화.
#[derive(Debug, Clone, Default)]
pub struct DebugConfig {
    /// true이면 충돌 영역을 반투명 사각형으로 시각화한다.
    pub show_colliders: bool,
}

// ─── 시스템 ──────────────────────────────────────────────────────────────────

/// 충돌 디버그 시각화 시스템.
///
/// 매 프레임 `SpatialGrid`를 재구성하고, `DebugConfig::show_colliders`가
/// true이면 각 콜라이더를 반투명 사각형으로 `UiQueue`에 추가한다.
///
/// `CollisionGridSystem`과 별도로 그리드를 유지한다.
/// `CollisionGridSystem`의 그리드를 직접 접근하려면 커스텀 시스템을 작성한다.
pub struct CollisionDebugSystem {
    grid: SpatialGrid,
}

impl CollisionDebugSystem {
    pub fn new(cell_size: f32) -> Self {
        Self {
            grid: SpatialGrid::new(cell_size),
        }
    }

    pub fn grid(&self) -> &SpatialGrid {
        &self.grid
    }
}

impl System for CollisionDebugSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let enabled = world
            .resource::<DebugConfig>()
            .map(|c| c.show_colliders)
            .unwrap_or(false);
        if !enabled {
            return;
        }
        self.grid.rebuild(world);

        let rects: Vec<DebugRect> = self
            .grid
            .entries
            .values()
            .map(|entry| {
                let (aabb_min, aabb_max) = entry.collider.aabb(entry.center);
                DebugRect {
                    min: aabb_min,
                    max: aabb_max,
                    color: [0.0, 1.0, 0.2, 0.25],
                    z: 999.0,
                }
            })
            .collect();

        if let Some(queue) = world.resource_mut::<DebugDrawQueue>() {
            queue.items.extend(rects);
        }
    }
}
