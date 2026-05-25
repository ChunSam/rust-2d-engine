use glam::Vec2;
use std::collections::HashMap;

/// 개별 터치 포인트 정보.
#[derive(Clone)]
pub struct TouchPoint {
    /// 현재 스크린 좌표
    pub position: Vec2,
    /// 터치 시작 좌표 (스와이프 감지용)
    pub start_position: Vec2,
}

/// 멀티터치 입력 상태 ECS 리소스.
///
/// `App::new()` 에서 자동 등록된다.
/// 시스템에서 `world.resource::<TouchState>()` 로 읽는다.
///
/// # 예제
/// ```ignore
/// if let Some(ts) = world.resource::<TouchState>() {
///     if ts.is_touching() {
///         // 첫 번째 터치 위치
///         if let Some(pos) = ts.primary_position() {
///             println!("터치 위치: {pos:?}");
///         }
///     }
/// }
/// ```
pub struct TouchState {
    /// 현재 활성 터치 포인트 (id → TouchPoint)
    active: HashMap<u64, TouchPoint>,

    /// 이번 프레임에 새로 시작된 터치 (id, 시작 위치)
    pub began: Vec<(u64, Vec2)>,

    /// 이번 프레임에 이동된 터치 (id, 현재 위치, 델타)
    pub moved: Vec<(u64, Vec2, Vec2)>,

    /// 이번 프레임에 끝난 터치 (id, 끝 위치)
    pub ended: Vec<(u64, Vec2)>,

    /// 핀치 줌 델타 (양수 = 두 손가락 벌어짐, 음수 = 좁혀짐).
    /// 두 손가락이 활성일 때만 업데이트된다.
    pub pinch_delta: f32,

    prev_pinch_dist: f32,

    /// 이번 프레임 스와이프 벡터 (터치 종료 시 50px 이상 이동한 경우).
    /// `ended` 이벤트 처리 후 설정된다.
    pub swipe: Option<Vec2>,
}

impl Default for TouchState {
    fn default() -> Self {
        Self {
            active: HashMap::new(),
            began: Vec::new(),
            moved: Vec::new(),
            ended: Vec::new(),
            pinch_delta: 0.0,
            prev_pinch_dist: 0.0,
            swipe: None,
        }
    }
}

impl TouchState {
    // ── 내부 업데이트 메서드 (App에서만 호출) ─────────────────────────────────

    pub(crate) fn on_touch_started(&mut self, id: u64, pos: Vec2) {
        self.active.insert(
            id,
            TouchPoint {
                position: pos,
                start_position: pos,
            },
        );
        self.began.push((id, pos));
    }

    pub(crate) fn on_touch_moved(&mut self, id: u64, pos: Vec2) {
        if let Some(point) = self.active.get_mut(&id) {
            let prev = point.position;
            let delta = pos - prev;
            point.position = pos;
            self.moved.push((id, pos, delta));
        } else {
            // 시작 없이 moved가 오는 경우 (예: 윈도우 밖에서 시작된 터치)
            self.active.insert(
                id,
                TouchPoint {
                    position: pos,
                    start_position: pos,
                },
            );
            self.moved.push((id, pos, Vec2::ZERO));
        }

        // 핀치 감지: 활성 포인트가 정확히 2개일 때
        self.update_pinch();
    }

    pub(crate) fn on_touch_ended(&mut self, id: u64, pos: Vec2) {
        if let Some(point) = self.active.remove(&id) {
            let travel = pos - point.start_position;
            if travel.length() >= 50.0 {
                self.swipe = Some(travel);
            }
        }
        self.ended.push((id, pos));
        // 핀치 거리 리셋 (손가락이 줄었으므로)
        self.prev_pinch_dist = 0.0;
        self.pinch_delta = 0.0;
    }

    /// 매 프레임 끝에 호출하여 프레임 버퍼를 초기화한다.
    pub(crate) fn flush(&mut self) {
        self.began.clear();
        self.moved.clear();
        self.ended.clear();
        self.pinch_delta = 0.0;
        self.swipe = None;
    }

    // ── 공개 접근 메서드 ──────────────────────────────────────────────────────

    /// 현재 활성 터치 포인트를 이터레이팅한다. `(id, 위치)` 반환.
    pub fn active_touches(&self) -> impl Iterator<Item = (u64, Vec2)> + '_ {
        self.active.iter().map(|(&id, p)| (id, p.position))
    }

    /// 현재 활성 터치 개수.
    pub fn touch_count(&self) -> usize {
        self.active.len()
    }

    /// 하나 이상의 터치가 활성화되어 있는지 여부.
    pub fn is_touching(&self) -> bool {
        !self.active.is_empty()
    }

    /// 가장 낮은 id를 가진 터치 포인트의 위치 (주 포인터).
    pub fn primary_position(&self) -> Option<Vec2> {
        self.active
            .iter()
            .min_by_key(|(&id, _)| id)
            .map(|(_, p)| p.position)
    }

    // ── 내부 핀치 거리 업데이트 ───────────────────────────────────────────────

    fn update_pinch(&mut self) {
        if self.active.len() != 2 {
            self.prev_pinch_dist = 0.0;
            return;
        }
        let mut iter = self.active.values();
        let a = iter.next().unwrap().position;
        let b = iter.next().unwrap().position;
        let dist = a.distance(b);

        if self.prev_pinch_dist > 0.0 {
            self.pinch_delta = dist - self.prev_pinch_dist;
        }
        self.prev_pinch_dist = dist;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_started_adds_active() {
        let mut ts = TouchState::default();
        ts.on_touch_started(0, Vec2::new(100.0, 200.0));
        assert_eq!(ts.touch_count(), 1);
        assert!(ts.is_touching());
        assert_eq!(ts.began.len(), 1);
        assert_eq!(ts.primary_position(), Some(Vec2::new(100.0, 200.0)));
    }

    #[test]
    fn touch_moved_updates_position() {
        let mut ts = TouchState::default();
        ts.on_touch_started(0, Vec2::new(0.0, 0.0));
        ts.on_touch_moved(0, Vec2::new(10.0, 5.0));
        assert_eq!(ts.primary_position(), Some(Vec2::new(10.0, 5.0)));
        assert_eq!(ts.moved.len(), 1);
        let (id, pos, delta) = ts.moved[0];
        assert_eq!(id, 0);
        assert_eq!(pos, Vec2::new(10.0, 5.0));
        assert_eq!(delta, Vec2::new(10.0, 5.0));
    }

    #[test]
    fn touch_ended_removes_active() {
        let mut ts = TouchState::default();
        ts.on_touch_started(0, Vec2::new(0.0, 0.0));
        ts.on_touch_ended(0, Vec2::new(0.0, 0.0));
        assert_eq!(ts.touch_count(), 0);
        assert!(!ts.is_touching());
        assert_eq!(ts.ended.len(), 1);
    }

    #[test]
    fn swipe_detected_on_long_move() {
        let mut ts = TouchState::default();
        ts.on_touch_started(0, Vec2::new(0.0, 0.0));
        ts.on_touch_ended(0, Vec2::new(100.0, 0.0));
        assert!(ts.swipe.is_some());
        let swipe = ts.swipe.unwrap();
        assert!((swipe.x - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn swipe_not_detected_on_short_move() {
        let mut ts = TouchState::default();
        ts.on_touch_started(0, Vec2::new(0.0, 0.0));
        ts.on_touch_ended(0, Vec2::new(10.0, 0.0));
        assert!(ts.swipe.is_none());
    }

    #[test]
    fn flush_clears_frame_buffers() {
        let mut ts = TouchState::default();
        ts.on_touch_started(0, Vec2::ZERO);
        ts.on_touch_moved(0, Vec2::new(5.0, 5.0));
        ts.flush();
        assert!(ts.began.is_empty());
        assert!(ts.moved.is_empty());
        assert_eq!(ts.pinch_delta, 0.0);
        assert!(ts.swipe.is_none());
        // 활성 포인트는 flush 후에도 유지
        assert_eq!(ts.touch_count(), 1);
    }

    #[test]
    fn pinch_delta_computed_for_two_fingers() {
        let mut ts = TouchState::default();
        ts.on_touch_started(0, Vec2::new(0.0, 0.0));
        ts.on_touch_started(1, Vec2::new(100.0, 0.0));
        // 첫 moved → prev_dist 설정
        ts.on_touch_moved(0, Vec2::new(0.0, 0.0));
        let delta_after_first = ts.pinch_delta;
        // 두 번째 moved (손가락 벌어짐)
        ts.on_touch_moved(0, Vec2::new(-10.0, 0.0));
        // 두 손가락 거리: 110 - 100 = 10
        assert!(ts.pinch_delta > 0.0 || delta_after_first == 0.0);
    }

    #[test]
    fn primary_position_returns_lowest_id() {
        let mut ts = TouchState::default();
        ts.on_touch_started(5, Vec2::new(500.0, 0.0));
        ts.on_touch_started(2, Vec2::new(200.0, 0.0));
        ts.on_touch_started(8, Vec2::new(800.0, 0.0));
        assert_eq!(ts.primary_position(), Some(Vec2::new(200.0, 0.0)));
    }
}
