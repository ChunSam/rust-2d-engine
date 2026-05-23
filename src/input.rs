use glam::Vec2;
use std::collections::HashSet;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

/// 키보드·마우스 상태를 담는 ECS 리소스
///
/// World에 삽입 후 시스템에서 `world.resource::<InputState>()` 로 접근.
pub struct InputState {
    // ── 키보드 ──────────────────────────────────────────────────────────────
    pressed: HashSet<KeyCode>,
    just_pressed: HashSet<KeyCode>,
    just_released: HashSet<KeyCode>,

    // ── 마우스 ──────────────────────────────────────────────────────────────
    /// 윈도우 좌상단 (0,0) 기준 픽셀 좌표
    cursor: Vec2,
    /// 누르고 있는 상태  [Left=0, Right=1, Middle=2]
    mouse_pressed: [bool; 3],
    /// 누른 프레임에만 true
    mouse_just_pressed: [bool; 3],
    /// 뗀 프레임에만 true
    mouse_just_released: [bool; 3],
    /// 한 프레임 동안 누적된 휠 델타. flush 시 0 으로 리셋
    scroll: f32,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            pressed: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
            cursor: Vec2::ZERO,
            mouse_pressed: [false; 3],
            mouse_just_pressed: [false; 3],
            mouse_just_released: [false; 3],
            scroll: 0.0,
        }
    }
}

impl InputState {
    // ── 키보드 공개 메서드 ────────────────────────────────────────────────────

    /// 키를 누른 순간 true (1프레임만)
    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    /// 키를 누르고 있는 동안 true
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    /// 키를 뗀 순간 true (1프레임만)
    pub fn just_released(&self, key: KeyCode) -> bool {
        self.just_released.contains(&key)
    }

    // ── 마우스 공개 메서드 ────────────────────────────────────────────────────

    /// 현재 커서 좌표 (윈도우 좌상단 기준 픽셀)
    pub fn cursor(&self) -> Vec2 {
        self.cursor
    }

    /// 마우스 버튼을 누르고 있는 동안 true
    pub fn is_mouse_pressed(&self, btn: MouseButton) -> bool {
        mouse_button_index(btn).map_or(false, |i| self.mouse_pressed[i])
    }

    /// 마우스 버튼을 누른 순간 true (1프레임만)
    pub fn mouse_just_pressed(&self, btn: MouseButton) -> bool {
        mouse_button_index(btn).map_or(false, |i| self.mouse_just_pressed[i])
    }

    /// 마우스 버튼을 뗀 순간 true (1프레임만)
    pub fn mouse_just_released(&self, btn: MouseButton) -> bool {
        mouse_button_index(btn).map_or(false, |i| self.mouse_just_released[i])
    }

    /// 이번 프레임 누적 휠 델타 (line 단위)
    pub fn scroll(&self) -> f32 {
        self.scroll
    }

    // ── 내부 업데이트 (App에서만 호출) ───────────────────────────────────────

    pub(crate) fn press(&mut self, key: KeyCode) {
        if self.pressed.insert(key) {
            self.just_pressed.insert(key);
        }
    }

    pub(crate) fn release(&mut self, key: KeyCode) {
        self.pressed.remove(&key);
        self.just_released.insert(key);
    }

    pub(crate) fn set_cursor(&mut self, pos: Vec2) {
        self.cursor = pos;
    }

    /// 이미 눌린 상태면 just_pressed 를 다시 켜지 않는다 (winit auto-repeat 회피)
    pub(crate) fn press_mouse(&mut self, btn: MouseButton) {
        if let Some(i) = mouse_button_index(btn) {
            if !self.mouse_pressed[i] {
                self.mouse_pressed[i] = true;
                self.mouse_just_pressed[i] = true;
            }
        }
    }

    pub(crate) fn release_mouse(&mut self, btn: MouseButton) {
        if let Some(i) = mouse_button_index(btn) {
            self.mouse_pressed[i] = false;
            self.mouse_just_released[i] = true;
        }
    }

    pub(crate) fn add_scroll(&mut self, delta: f32) {
        self.scroll += delta;
    }

    /// 프레임 끝에 just_* 초기화
    pub(crate) fn flush(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
        self.mouse_just_pressed = [false; 3];
        self.mouse_just_released = [false; 3];
        self.scroll = 0.0;
    }
}

/// MouseButton → 배열 인덱스 (Left=0, Right=1, Middle=2, 그 외 None)
fn mouse_button_index(btn: MouseButton) -> Option<usize> {
    match btn {
        MouseButton::Left => Some(0),
        MouseButton::Right => Some(1),
        MouseButton::Middle => Some(2),
        _ => None,
    }
}

// ─── 단위 테스트 ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mouse_press_sets_state() {
        let mut input = InputState::default();
        input.press_mouse(MouseButton::Left);

        assert!(
            input.is_mouse_pressed(MouseButton::Left),
            "press 후 is_mouse_pressed 가 true 여야 한다"
        );
        assert!(
            input.mouse_just_pressed(MouseButton::Left),
            "press 후 mouse_just_pressed 가 true 여야 한다"
        );

        input.flush();

        assert!(
            input.is_mouse_pressed(MouseButton::Left),
            "flush 후에도 누르고 있으면 is_mouse_pressed 는 true"
        );
        assert!(
            !input.mouse_just_pressed(MouseButton::Left),
            "flush 후 mouse_just_pressed 는 false 여야 한다"
        );
    }

    #[test]
    fn mouse_release_clears_pressed() {
        let mut input = InputState::default();
        input.press_mouse(MouseButton::Left);
        input.release_mouse(MouseButton::Left);

        assert!(
            !input.is_mouse_pressed(MouseButton::Left),
            "release 후 is_mouse_pressed 는 false"
        );
        assert!(
            input.mouse_just_released(MouseButton::Left),
            "release 후 mouse_just_released 는 true"
        );

        input.flush();

        assert!(
            !input.mouse_just_released(MouseButton::Left),
            "flush 후 mouse_just_released 는 false"
        );
    }

    #[test]
    fn mouse_press_twice_no_repeat() {
        let mut input = InputState::default();
        input.press_mouse(MouseButton::Left); // 첫 번째 press → just_pressed=true
        input.press_mouse(MouseButton::Left); // 두 번째 press → 이미 눌렸으므로 just_pressed 를 다시 켜지 않음

        // flush 없이도 두 번째 press 는 just_pressed 를 덮어쓰지 않는다
        // (첫 호출에서 켜진 just_pressed 가 그대로지만, 핵심은 두 번째 press 가 중복 트리거하지 않는 것)
        assert!(
            input.is_mouse_pressed(MouseButton::Left),
            "두 번 눌러도 pressed=true"
        );
        assert!(
            input.mouse_just_pressed(MouseButton::Left),
            "첫 press 의 just_pressed 는 유지"
        );

        // flush 후 just_pressed 리셋, 다시 press → 이미 pressed=true 이므로 just_pressed 켜지지 않음
        input.flush();
        input.press_mouse(MouseButton::Left);
        assert!(
            !input.mouse_just_pressed(MouseButton::Left),
            "이미 눌린 상태에서 press 해도 just_pressed=false"
        );
    }

    #[test]
    fn cursor_updates() {
        let mut input = InputState::default();
        input.set_cursor(Vec2::new(123.0, 45.0));
        assert_eq!(input.cursor(), Vec2::new(123.0, 45.0));
    }

    #[test]
    fn scroll_accumulates_and_resets() {
        let mut input = InputState::default();
        input.add_scroll(1.0);
        input.add_scroll(2.5);
        assert!(
            (input.scroll() - 3.5).abs() < f32::EPSILON,
            "scroll 은 누적되어야 한다"
        );

        input.flush();
        assert_eq!(input.scroll(), 0.0, "flush 후 scroll 은 0.0 이어야 한다");
    }
}
