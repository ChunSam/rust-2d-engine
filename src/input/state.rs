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
    cursor: Vec2,
    mouse_pressed: [bool; 3],
    mouse_just_pressed: [bool; 3],
    mouse_just_released: [bool; 3],
    scroll: f32,
    text_input_chars: Vec<char>,
    ime_preedit: String,
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
            text_input_chars: Vec::new(),
            ime_preedit: String::new(),
        }
    }
}

impl InputState {
    // ── 키보드 공개 메서드 ────────────────────────────────────────────────────

    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    pub fn just_released(&self, key: KeyCode) -> bool {
        self.just_released.contains(&key)
    }

    // ── 마우스 공개 메서드 ────────────────────────────────────────────────────

    pub fn cursor(&self) -> Vec2 {
        self.cursor
    }

    pub fn is_mouse_pressed(&self, btn: MouseButton) -> bool {
        mouse_button_index(btn).map_or(false, |i| self.mouse_pressed[i])
    }

    pub fn mouse_just_pressed(&self, btn: MouseButton) -> bool {
        mouse_button_index(btn).map_or(false, |i| self.mouse_just_pressed[i])
    }

    pub fn mouse_just_released(&self, btn: MouseButton) -> bool {
        mouse_button_index(btn).map_or(false, |i| self.mouse_just_released[i])
    }

    pub fn scroll(&self) -> f32 {
        self.scroll
    }

    /// 이번 프레임에 입력된 문자 슬라이스를 반환한다.
    ///
    /// `'\x08'` = Backspace, `'\n'` = Enter, 나머지 = 일반 문자.
    pub fn text_chars(&self) -> &[char] {
        &self.text_input_chars
    }

    pub fn ime_preedit(&self) -> &str {
        &self.ime_preedit
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

    pub(crate) fn push_char(&mut self, c: char) {
        self.text_input_chars.push(c);
    }

    pub(crate) fn push_text(&mut self, s: &str) {
        self.text_input_chars.extend(s.chars());
    }

    pub(crate) fn push_backspace(&mut self) {
        self.text_input_chars.push('\x08');
    }

    pub(crate) fn push_enter(&mut self) {
        self.text_input_chars.push('\n');
    }

    pub(crate) fn set_ime_preedit(&mut self, preedit: String) {
        self.ime_preedit = preedit;
    }

    pub(crate) fn clear_ime_preedit(&mut self) {
        self.ime_preedit.clear();
    }

    pub(crate) fn flush(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
        self.mouse_just_pressed = [false; 3];
        self.mouse_just_released = [false; 3];
        self.scroll = 0.0;
        self.text_input_chars.clear();
    }
}

fn mouse_button_index(btn: MouseButton) -> Option<usize> {
    match btn {
        MouseButton::Left => Some(0),
        MouseButton::Right => Some(1),
        MouseButton::Middle => Some(2),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mouse_press_sets_state() {
        let mut input = InputState::default();
        input.press_mouse(MouseButton::Left);
        assert!(input.is_mouse_pressed(MouseButton::Left));
        assert!(input.mouse_just_pressed(MouseButton::Left));
        input.flush();
        assert!(input.is_mouse_pressed(MouseButton::Left));
        assert!(!input.mouse_just_pressed(MouseButton::Left));
    }

    #[test]
    fn mouse_release_clears_pressed() {
        let mut input = InputState::default();
        input.press_mouse(MouseButton::Left);
        input.release_mouse(MouseButton::Left);
        assert!(!input.is_mouse_pressed(MouseButton::Left));
        assert!(input.mouse_just_released(MouseButton::Left));
        input.flush();
        assert!(!input.mouse_just_released(MouseButton::Left));
    }

    #[test]
    fn mouse_press_twice_no_repeat() {
        let mut input = InputState::default();
        input.press_mouse(MouseButton::Left);
        input.press_mouse(MouseButton::Left);
        assert!(input.is_mouse_pressed(MouseButton::Left));
        input.flush();
        input.press_mouse(MouseButton::Left);
        assert!(!input.mouse_just_pressed(MouseButton::Left));
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
        assert!((input.scroll() - 3.5).abs() < f32::EPSILON);
        input.flush();
        assert_eq!(input.scroll(), 0.0);
    }

    #[test]
    fn ime_preedit_persists_until_cleared() {
        let mut input = InputState::default();
        input.set_ime_preedit("한".to_string());
        input.push_text("글");
        input.flush();
        assert_eq!(input.ime_preedit(), "한");
        assert!(input.text_chars().is_empty());
        input.clear_ime_preedit();
        assert_eq!(input.ime_preedit(), "");
    }
}
