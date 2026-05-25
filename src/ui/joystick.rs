use glam::Vec2;

use crate::input::TouchState;

/// 가상 조이스틱 컴포넌트.
///
/// 엔티티에 붙여 사용한다. 매 프레임 `update()` 를 호출하면
/// 터치(또는 마우스 에뮬레이션) 입력을 받아 `output` 방향 벡터를 갱신한다.
///
/// # 예제
/// ```ignore
/// let joy_e = world.spawn();
/// world.add_component(joy_e, VirtualJoystick::new(Vec2::new(120.0, 480.0), 60.0));
///
/// // 시스템 내부
/// if let Some(joy) = world.get_mut::<VirtualJoystick>(joy_e) {
///     if let Some(ts) = world.resource::<TouchState>() {
///         joy.update(ts);
///     }
///     let dir = joy.output; // Vec2 (-1..1, -1..1)
/// }
/// ```
pub struct VirtualJoystick {
    /// 조이스틱 베이스 중심 좌표 (화면/UI 좌표계)
    pub center: Vec2,

    /// 스틱이 이동 가능한 최대 반경 (픽셀)
    pub radius: f32,

    /// 정규화된 출력 방향. 각 축 범위: -1.0 ~ 1.0.
    /// 입력이 없으면 `Vec2::ZERO`.
    pub output: Vec2,

    /// 현재 스틱 핵의 화면 좌표 (렌더링/디버그 시각화용)
    pub stick_pos: Vec2,

    /// `true` 이면 DebugDraw로 조이스틱 원을 시각화한다.
    pub visible: bool,

    /// 현재 이 조이스틱을 조작 중인 터치 포인트 ID
    touch_id: Option<u64>,
}

impl VirtualJoystick {
    /// 중심 좌표와 반경으로 새 가상 조이스틱을 만든다.
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self {
            center,
            radius,
            output: Vec2::ZERO,
            stick_pos: center,
            visible: true,
            touch_id: None,
        }
    }

    /// 매 프레임 `TouchState` 로 조이스틱 상태를 업데이트한다.
    ///
    /// `TouchState::flush()` 전에 호출해야 한다.
    pub fn update(&mut self, touch_state: &TouchState) {
        // 1. touch_id가 없으면: began 목록에서 반경 안의 터치 찾아 할당
        if self.touch_id.is_none() {
            for &(id, pos) in &touch_state.began {
                if (pos - self.center).length() <= self.radius {
                    self.touch_id = Some(id);
                    self.update_stick(pos);
                    break;
                }
            }
        }

        // 2. touch_id가 있으면: 해당 포인트의 현재 위치 추적
        if let Some(active_id) = self.touch_id {
            // ended 이벤트 확인
            let is_ended = touch_state
                .ended
                .iter()
                .any(|&(id, _)| id == active_id);

            if is_ended {
                self.touch_id = None;
                self.output = Vec2::ZERO;
                self.stick_pos = self.center;
            } else {
                // 현재 활성 포인트 위치 찾기
                let pos = touch_state
                    .active_touches()
                    .find(|(id, _)| *id == active_id)
                    .map(|(_, pos)| pos);

                if let Some(pos) = pos {
                    self.update_stick(pos);
                }
            }
        }
    }

    /// 스틱 위치와 output 벡터를 주어진 터치 위치로 갱신한다.
    fn update_stick(&mut self, pos: Vec2) {
        let delta = pos - self.center;
        let magnitude = delta.length();

        if magnitude < f32::EPSILON {
            self.output = Vec2::ZERO;
            self.stick_pos = self.center;
        } else if magnitude > self.radius {
            // 반경 밖: 방향만 유지
            self.output = delta / magnitude; // normalize
            self.stick_pos = self.center + self.output * self.radius;
        } else {
            // 반경 안: 0..1 정규화
            self.output = delta / self.radius;
            self.stick_pos = pos;
        }
    }

    /// 조이스틱이 현재 눌려 있는지 여부.
    pub fn is_active(&self) -> bool {
        self.touch_id.is_some()
    }

    /// `TouchState` 의 원시 데이터를 직접 전달해 업데이트한다.
    ///
    /// 시스템에서 `world.resource::<TouchState>()` 와 `world.get_mut::<VirtualJoystick>()`
    /// 를 동시에 borrow 할 수 없을 때 사용한다.
    /// 먼저 터치 데이터를 owned 값으로 복사한 후 `world.get_mut` 으로 이 메서드를 호출한다.
    ///
    /// # 인수
    /// - `began`: 이번 프레임 시작 터치 `(id, 위치)`
    /// - `ended`: 이번 프레임 종료 터치 `(id, 위치)`
    /// - `active`: 현재 활성 터치 `(id, 위치)`
    pub fn update_raw(
        &mut self,
        began: &[(u64, Vec2)],
        ended: &[(u64, Vec2)],
        active: &[(u64, Vec2)],
    ) {
        // 1. touch_id가 없으면: began 에서 반경 안의 터치 찾아 할당
        if self.touch_id.is_none() {
            for &(id, pos) in began {
                if (pos - self.center).length() <= self.radius {
                    self.touch_id = Some(id);
                    self.update_stick(pos);
                    break;
                }
            }
        }

        // 2. touch_id가 있으면: 현재 위치 추적
        if let Some(active_id) = self.touch_id {
            let is_ended = ended.iter().any(|&(id, _)| id == active_id);
            if is_ended {
                self.touch_id = None;
                self.output = Vec2::ZERO;
                self.stick_pos = self.center;
            } else if let Some(&(_, pos)) = active.iter().find(|(id, _)| *id == active_id) {
                self.update_stick(pos);
            }
        }
    }

    /// 데드존 적용된 출력 반환.
    ///
    /// `deadzone` 범위 안의 작은 입력은 `Vec2::ZERO` 로 처리한다.
    pub fn output_with_deadzone(&self, deadzone: f32) -> Vec2 {
        if self.output.length() < deadzone {
            Vec2::ZERO
        } else {
            self.output
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::TouchState;

    #[test]
    fn joystick_activates_on_touch_within_radius() {
        let mut joy = VirtualJoystick::new(Vec2::new(100.0, 100.0), 60.0);
        let mut ts = TouchState::default();

        ts.on_touch_started(0, Vec2::new(110.0, 100.0)); // 반경 안
        joy.update(&ts);

        assert!(joy.is_active());
        assert!(joy.output.length() > 0.0);
    }

    #[test]
    fn joystick_ignores_touch_outside_radius() {
        let mut joy = VirtualJoystick::new(Vec2::new(100.0, 100.0), 60.0);
        let mut ts = TouchState::default();

        ts.on_touch_started(0, Vec2::new(300.0, 300.0)); // 반경 밖
        joy.update(&ts);

        assert!(!joy.is_active());
    }

    #[test]
    fn joystick_resets_on_touch_end() {
        let mut joy = VirtualJoystick::new(Vec2::new(100.0, 100.0), 60.0);
        let mut ts = TouchState::default();

        ts.on_touch_started(0, Vec2::new(110.0, 100.0));
        joy.update(&ts);
        assert!(joy.is_active());

        ts.flush();
        ts.on_touch_ended(0, Vec2::new(110.0, 100.0));
        joy.update(&ts);

        assert!(!joy.is_active());
        assert_eq!(joy.output, Vec2::ZERO);
        assert_eq!(joy.stick_pos, joy.center);
    }

    #[test]
    fn joystick_output_clamped_at_unit_when_outside_radius() {
        let mut joy = VirtualJoystick::new(Vec2::new(0.0, 0.0), 50.0);
        let mut ts = TouchState::default();

        // 반경 안에서 시작해 조이스틱 활성화
        ts.on_touch_started(0, Vec2::new(10.0, 0.0));
        joy.update(&ts);
        assert!(joy.is_active());

        // 다음 프레임: 반경 밖으로 이동
        ts.flush();
        ts.on_touch_started(0, Vec2::new(10.0, 0.0)); // active에 유지
        ts.on_touch_moved(0, Vec2::new(200.0, 0.0));
        joy.update(&ts);

        // output 크기는 1.0 (정규화됨)
        assert!((joy.output.length() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn joystick_output_proportional_inside_radius() {
        let mut joy = VirtualJoystick::new(Vec2::new(0.0, 0.0), 100.0);
        let mut ts = TouchState::default();

        ts.on_touch_started(0, Vec2::new(50.0, 0.0)); // 반경의 절반
        joy.update(&ts);

        // output.x ≈ 0.5
        assert!((joy.output.x - 0.5).abs() < 1e-5);
        assert!(joy.output.y.abs() < 1e-5);
    }

    #[test]
    fn deadzone_suppresses_small_input() {
        let mut joy = VirtualJoystick::new(Vec2::new(0.0, 0.0), 100.0);
        let mut ts = TouchState::default();

        ts.on_touch_started(0, Vec2::new(5.0, 0.0)); // 매우 작은 이동
        joy.update(&ts);

        assert_eq!(joy.output_with_deadzone(0.1), Vec2::ZERO);
    }
}
