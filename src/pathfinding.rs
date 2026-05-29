use glam::IVec2;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// 통행 가능 여부를 저장하는 격자 (row-major 배열)
pub struct PathGrid {
    pub width: i32,
    pub height: i32,
    cells: Vec<bool>,
}

const MAX_PATH_GRID_CELLS: usize = 10_000_000;

impl PathGrid {
    /// 전부 통행 가능 상태로 초기화
    pub fn new(width: i32, height: i32) -> Self {
        let size = grid_cell_count(width, height);
        Self {
            width,
            height,
            cells: vec![true; size],
        }
    }

    /// 전부 막힌 상태로 초기화
    pub fn new_blocked(width: i32, height: i32) -> Self {
        let size = grid_cell_count(width, height);
        Self {
            width,
            height,
            cells: vec![false; size],
        }
    }

    pub fn set_walkable(&mut self, x: i32, y: i32, walkable: bool) {
        if let Some(idx) = self.index(x, y) {
            self.cells[idx] = walkable;
        }
    }

    /// 범위 밖 좌표는 `false` 반환 (패닉 없음)
    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        self.index(x, y).map(|i| self.cells[i]).unwrap_or(false)
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        let idx = y.checked_mul(self.width)?.checked_add(x)? as usize;
        (idx < self.cells.len()).then_some(idx)
    }
}

fn grid_cell_count(width: i32, height: i32) -> usize {
    let Some(size) = width.checked_mul(height) else {
        return 0;
    };
    if size <= 0 {
        return 0;
    }
    let size = size as usize;
    if size > MAX_PATH_GRID_CELLS {
        0
    } else {
        size
    }
}

// ── A* 내부 구조 ──────────────────────────────────────────────────────────────

#[derive(Eq, PartialEq)]
struct Node {
    f: i32, // f = g + h
    pos: IVec2,
}

// BinaryHeap은 최대 힙 → f가 작을수록 우선순위 높게 역순 비교
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.cmp(&self.f).then_with(|| {
            other
                .pos
                .x
                .cmp(&self.pos.x)
                .then(other.pos.y.cmp(&self.pos.y))
        })
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn manhattan(a: IVec2, b: IVec2) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

const NEIGHBORS: [IVec2; 4] = [
    IVec2::new(0, -1),
    IVec2::new(0, 1),
    IVec2::new(-1, 0),
    IVec2::new(1, 0),
];

/// A* 경로 탐색.
/// 반환: 시작점 제외, 목표점 포함한 경로. 경로 없으면 `None`.
pub fn find_path(grid: &PathGrid, start: IVec2, goal: IVec2) -> Option<Vec<IVec2>> {
    // 시작 == 목표
    if start == goal {
        return Some(vec![goal]);
    }

    // 목표가 막혀 있으면 즉시 None
    if !grid.is_walkable(goal.x, goal.y) {
        return None;
    }

    let mut open: BinaryHeap<Node> = BinaryHeap::new();
    // g_score: 시작점에서 각 노드까지의 실제 비용
    let mut g_score: HashMap<IVec2, i32> = HashMap::new();
    // came_from: 경로 역추적용
    let mut came_from: HashMap<IVec2, IVec2> = HashMap::new();

    g_score.insert(start, 0);
    open.push(Node {
        f: manhattan(start, goal),
        pos: start,
    });

    while let Some(Node { pos: current, .. }) = open.pop() {
        if current == goal {
            // 경로 역추적
            let mut path = Vec::new();
            let mut cur = current;
            while let Some(&prev) = came_from.get(&cur) {
                path.push(cur);
                cur = prev;
            }
            path.reverse();
            return Some(path);
        }

        let g_current = *g_score.get(&current).unwrap_or(&i32::MAX);

        for &dir in &NEIGHBORS {
            let next = current + dir;
            if !grid.is_walkable(next.x, next.y) {
                continue;
            }
            let g_next = g_current + 1;
            if g_next < *g_score.get(&next).unwrap_or(&i32::MAX) {
                g_score.insert(next, g_next);
                came_from.insert(next, current);
                open.push(Node {
                    f: g_next + manhattan(next, goal),
                    pos: next,
                });
            }
        }
    }

    None
}

// ── 테스트 ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_straight_path() {
        // 5x1 격자, 장애물 없음 → start(0,0)에서 goal(4,0)까지 직선
        let grid = PathGrid::new(5, 1);
        let path = find_path(&grid, IVec2::new(0, 0), IVec2::new(4, 0)).unwrap();
        assert_eq!(path.last(), Some(&IVec2::new(4, 0)));
        // start 미포함, goal 포함
        assert!(!path.contains(&IVec2::new(0, 0)));
        assert_eq!(path.len(), 4);
    }

    #[test]
    fn test_obstacle_detour() {
        // 3x3 격자에서 (0,1) 장애물로 인해 우회
        // S . .
        // X X .
        // . . G
        let mut grid = PathGrid::new(3, 3);
        grid.set_walkable(0, 1, false);
        grid.set_walkable(1, 1, false);

        let path = find_path(&grid, IVec2::new(0, 0), IVec2::new(2, 2)).unwrap();
        // 경로가 존재하고 모든 셀이 통행 가능해야 함
        for pos in &path {
            assert!(grid.is_walkable(pos.x, pos.y), "막힌 셀 포함: {pos:?}");
        }
        assert_eq!(path.last(), Some(&IVec2::new(2, 2)));
        assert!(!path.contains(&IVec2::new(0, 0)));
    }

    #[test]
    fn test_no_path() {
        // 3x3 격자, 오른쪽 열 전체 막음 → 목표(2,0) 도달 불가
        let mut grid = PathGrid::new(3, 3);
        grid.set_walkable(1, 0, false);
        grid.set_walkable(1, 1, false);
        grid.set_walkable(1, 2, false);
        // 목표도 막힌 열에 있으므로 즉시 None
        let result = find_path(&grid, IVec2::new(0, 0), IVec2::new(2, 0));
        assert!(result.is_none());
    }

    #[test]
    fn test_start_equals_goal() {
        let grid = PathGrid::new(5, 5);
        let pos = IVec2::new(2, 3);
        let result = find_path(&grid, pos, pos);
        assert_eq!(result, Some(vec![pos]));
    }

    #[test]
    fn path_grid_invalid_or_overflow_size_is_empty() {
        let overflow = PathGrid::new(i32::MAX, 2);
        assert!(!overflow.is_walkable(0, 0));

        let negative = PathGrid::new(-3, 4);
        assert!(!negative.is_walkable(0, 0));

        let blocked = PathGrid::new_blocked(i32::MAX, 2);
        assert!(!blocked.is_walkable(0, 0));
    }
}
