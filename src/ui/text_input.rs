/// 텍스트 입력 위젯 컴포넌트.
///
/// `UiNode` 와 함께 엔티티에 붙여 사용한다.
/// `UiSystem` 이 클릭 시 포커스를 설정하고, 문자 버퍼를 소비해 텍스트를 갱신한다.
pub struct TextInput {
    pub text: String,
    /// UTF-8 byte index
    pub cursor: usize,
    pub focused: bool,
    pub placeholder: String,
    pub max_len: usize,
    /// dt 누적값. 0.5초마다 cursor_visible 토글
    pub cursor_blink: f32,
    pub cursor_visible: bool,

    pub color_normal: [f32; 4],
    pub color_focused: [f32; 4],
    pub text_color: [u8; 4],
    pub font_size: f32,
}

impl TextInput {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            focused: false,
            placeholder: placeholder.into(),
            max_len: 256,
            cursor_blink: 0.0,
            cursor_visible: true,
            color_normal: [0.15, 0.15, 0.20, 1.0],
            color_focused: [0.20, 0.25, 0.35, 1.0],
            text_color: [220, 220, 220, 255],
            font_size: 16.0,
        }
    }

    pub fn with_max_len(mut self, n: usize) -> Self {
        self.max_len = n;
        self
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn current_color(&self) -> [f32; 4] {
        if self.focused {
            self.color_focused
        } else {
            self.color_normal
        }
    }

    /// 커서 바로 앞 문자를 삭제한다 (UTF-8 안전).
    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let mut char_start = self.cursor - 1;
        while !self.text.is_char_boundary(char_start) {
            char_start -= 1;
        }
        self.text.drain(char_start..self.cursor);
        self.cursor = char_start;
    }

    /// 커서 위치에 문자를 삽입한다.
    pub fn insert_char(&mut self, c: char) {
        if self.text.len() < self.max_len {
            self.text.insert(self.cursor, c);
            self.cursor += c.len_utf8();
        }
    }
}
