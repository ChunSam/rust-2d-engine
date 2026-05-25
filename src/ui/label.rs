use crate::renderer::TextAlign;

/// 텍스트 레이블 컴포넌트.
///
/// `UiNode` 와 함께 사용한다. `UiSystem` 이 매 프레임
/// `TextQueue` 에 `DrawText` 를 제출해 렌더링한다.
pub struct Label {
    pub text: String,
    /// RGBA (0~255)
    pub color: [u8; 4],
    pub font_size: f32,
    pub align: TextAlign,
    pub rich: bool,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: [220, 220, 220, 255],
            font_size: 16.0,
            align: TextAlign::Left,
            rich: false,
        }
    }

    pub fn with_color(mut self, color: [u8; 4]) -> Self {
        self.color = color;
        self
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn rich(mut self) -> Self {
        self.rich = true;
        self
    }
}
