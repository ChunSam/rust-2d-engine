use std::collections::{HashMap, HashSet};

/// 게임패드 버튼 식별자.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    South,        // A (Xbox) / Cross (PS)
    East,         // B (Xbox) / Circle (PS)
    North,        // Y (Xbox) / Triangle (PS)
    West,         // X (Xbox) / Square (PS)
    LeftBumper,   // LB / L1
    RightBumper,  // RB / R1
    LeftTrigger,  // LT / L2 (디지털)
    RightTrigger, // RT / R2 (디지털)
    Select,
    Start,
    LeftThumb,
    RightThumb,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

/// 게임패드 축 식별자.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,  // L2/LT 아날로그 (0.0 ~ 1.0)
    RightTrigger, // R2/RT 아날로그 (0.0 ~ 1.0)
    DPadX,
    DPadY,
}

struct Slot {
    pressed: HashSet<GamepadButton>,
    just_pressed: HashSet<GamepadButton>,
    just_released: HashSet<GamepadButton>,
    axes: HashMap<GamepadAxis, f32>,
}

impl Slot {
    #[cfg(not(target_arch = "wasm32"))]
    fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
            axes: HashMap::new(),
        }
    }

    fn flush(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }
}

/// 게임패드 입력 상태를 담는 ECS 리소스.
///
/// 최대 4개 게임패드(슬롯 0~3)를 지원한다.
/// `App::new()` 에서 자동 삽입되므로 별도 등록 불필요.
///
/// # 예제
/// ```ignore
/// // 슬롯 0 (첫 번째 연결된 패드)
/// if let Some(gs) = world.resource::<GamepadState>() {
///     if gs.just_pressed(0, GamepadButton::South) { /* 점프 */ }
///     let lx = gs.axis(0, GamepadAxis::LeftStickX);
/// }
/// ```
pub struct GamepadState {
    slots: [Option<Slot>; 4],
    #[cfg(not(target_arch = "wasm32"))]
    id_map: HashMap<gilrs::GamepadId, usize>,
}

impl Default for GamepadState {
    fn default() -> Self {
        Self {
            slots: [None, None, None, None],
            #[cfg(not(target_arch = "wasm32"))]
            id_map: HashMap::new(),
        }
    }
}

impl GamepadState {
    // ── 공개 쿼리 메서드 ──────────────────────────────────────────────────────

    /// `pad` 슬롯이 연결되어 있으면 true.
    pub fn is_connected(&self, pad: usize) -> bool {
        pad < 4 && self.slots[pad].is_some()
    }

    /// 최소 한 개의 게임패드가 연결되어 있으면 true.
    pub fn any_connected(&self) -> bool {
        self.slots.iter().any(|s| s.is_some())
    }

    /// 첫 번째 연결된 게임패드의 슬롯 인덱스.
    pub fn primary(&self) -> Option<usize> {
        self.slots.iter().position(|s| s.is_some())
    }

    /// `pad` 슬롯에서 `button`이 눌려 있으면 true.
    pub fn is_pressed(&self, pad: usize, button: GamepadButton) -> bool {
        self.slot(pad).map_or(false, |s| s.pressed.contains(&button))
    }

    /// `pad` 슬롯에서 `button`이 이번 프레임에 눌렸으면 true.
    pub fn just_pressed(&self, pad: usize, button: GamepadButton) -> bool {
        self.slot(pad).map_or(false, |s| s.just_pressed.contains(&button))
    }

    /// `pad` 슬롯에서 `button`이 이번 프레임에 떼어졌으면 true.
    pub fn just_released(&self, pad: usize, button: GamepadButton) -> bool {
        self.slot(pad).map_or(false, |s| s.just_released.contains(&button))
    }

    /// `pad` 슬롯의 `axis` 값 (−1.0 ~ 1.0, 데드존 미적용).
    pub fn axis(&self, pad: usize, axis: GamepadAxis) -> f32 {
        self.slot(pad)
            .and_then(|s| s.axes.get(&axis).copied())
            .unwrap_or(0.0)
    }

    fn slot(&self, pad: usize) -> Option<&Slot> {
        self.slots.get(pad)?.as_ref()
    }

    // ── 내부 이벤트 처리 (App에서만 호출) ────────────────────────────────────

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn process_event(&mut self, event: gilrs::Event) {
        use gilrs::EventType;

        let gid = event.id;

        match event.event {
            EventType::Connected => {
                if !self.id_map.contains_key(&gid) {
                    if let Some(idx) = self.slots.iter().position(|s| s.is_none()) {
                        self.slots[idx] = Some(Slot::new());
                        self.id_map.insert(gid, idx);
                    }
                }
            }
            EventType::Disconnected => {
                if let Some(idx) = self.id_map.remove(&gid) {
                    self.slots[idx] = None;
                }
            }
            EventType::ButtonPressed(btn, _) => {
                if let Some(gb) = map_button(btn) {
                    if let Some(slot) = self.slot_mut(gid) {
                        slot.pressed.insert(gb);
                        slot.just_pressed.insert(gb);
                    }
                }
            }
            EventType::ButtonReleased(btn, _) => {
                if let Some(gb) = map_button(btn) {
                    if let Some(slot) = self.slot_mut(gid) {
                        slot.pressed.remove(&gb);
                        slot.just_released.insert(gb);
                    }
                }
            }
            EventType::AxisChanged(axis, value, _) => {
                if let Some(ga) = map_axis(axis) {
                    if let Some(slot) = self.slot_mut(gid) {
                        slot.axes.insert(ga, value);
                    }
                }
            }
            _ => {}
        }
    }

    pub(crate) fn flush(&mut self) {
        for slot in self.slots.iter_mut().flatten() {
            slot.flush();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn slot_mut(&mut self, gid: gilrs::GamepadId) -> Option<&mut Slot> {
        let idx = *self.id_map.get(&gid)?;
        self.slots[idx].as_mut()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn map_button(btn: gilrs::Button) -> Option<GamepadButton> {
    use gilrs::Button;
    Some(match btn {
        Button::South        => GamepadButton::South,
        Button::East         => GamepadButton::East,
        Button::North        => GamepadButton::North,
        Button::West         => GamepadButton::West,
        Button::LeftTrigger  => GamepadButton::LeftBumper,
        Button::RightTrigger => GamepadButton::RightBumper,
        Button::LeftTrigger2  => GamepadButton::LeftTrigger,
        Button::RightTrigger2 => GamepadButton::RightTrigger,
        Button::Select       => GamepadButton::Select,
        Button::Start        => GamepadButton::Start,
        Button::LeftThumb    => GamepadButton::LeftThumb,
        Button::RightThumb   => GamepadButton::RightThumb,
        Button::DPadUp       => GamepadButton::DPadUp,
        Button::DPadDown     => GamepadButton::DPadDown,
        Button::DPadLeft     => GamepadButton::DPadLeft,
        Button::DPadRight    => GamepadButton::DPadRight,
        _ => return None,
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn map_axis(axis: gilrs::Axis) -> Option<GamepadAxis> {
    use gilrs::Axis;
    Some(match axis {
        Axis::LeftStickX  => GamepadAxis::LeftStickX,
        Axis::LeftStickY  => GamepadAxis::LeftStickY,
        Axis::RightStickX => GamepadAxis::RightStickX,
        Axis::RightStickY => GamepadAxis::RightStickY,
        Axis::LeftZ       => GamepadAxis::LeftTrigger,
        Axis::RightZ      => GamepadAxis::RightTrigger,
        Axis::DPadX       => GamepadAxis::DPadX,
        Axis::DPadY       => GamepadAxis::DPadY,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_disconnected() {
        let gs = GamepadState::default();
        assert!(!gs.any_connected());
        assert!(gs.primary().is_none());
    }

    #[test]
    fn axis_returns_zero_when_disconnected() {
        let gs = GamepadState::default();
        assert_eq!(gs.axis(0, GamepadAxis::LeftStickX), 0.0);
    }
}
