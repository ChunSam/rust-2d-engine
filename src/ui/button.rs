/// 버튼의 상호작용 상태
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ButtonState {
    #[default]
    Normal,
    Hovered,
    Pressed,
    Disabled,
}

/// 클릭 가능한 버튼 컴포넌트.
///
/// `UiNode` 와 함께 엔티티에 붙여 사용한다.
/// `UiSystem` 이 매 프레임 히트 테스트 후 `state` 를 갱신하고
/// 배경 사각형 + 레이블 텍스트를 렌더링한다.
pub struct Button {
    pub label: String,
    pub state: ButtonState,
    pub color_normal: [f32; 4],
    pub color_hovered: [f32; 4],
    pub color_pressed: [f32; 4],
    pub color_disabled: [f32; 4],
    pub text_color: [u8; 4],
    pub font_size: f32,
}

impl Button {
    /// 기본 색상 preset 으로 버튼을 생성한다.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            state: ButtonState::Normal,
            color_normal: [0.20, 0.20, 0.25, 1.0],
            color_hovered: [0.30, 0.30, 0.40, 1.0],
            color_pressed: [0.12, 0.12, 0.18, 1.0],
            color_disabled: [0.15, 0.15, 0.15, 0.6],
            text_color: [220, 220, 220, 255],
            font_size: 18.0,
        }
    }

    /// 현재 상태에 대응하는 배경 색상을 반환한다.
    pub fn current_color(&self) -> [f32; 4] {
        match self.state {
            ButtonState::Normal => self.color_normal,
            ButtonState::Hovered => self.color_hovered,
            ButtonState::Pressed => self.color_pressed,
            ButtonState::Disabled => self.color_disabled,
        }
    }

    pub fn is_interactive(&self) -> bool {
        self.state != ButtonState::Disabled
    }
}
