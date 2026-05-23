/// 스크롤 가능한 텍스트 목록 위젯.
///
/// `UiNode` 와 함께 엔티티에 붙여 사용한다.
/// 자식 엔티티 없이 `items` Vec 을 직접 렌더링한다.
/// 커서가 위젯 위에 있을 때 마우스 휠로 스크롤한다.
pub struct ScrollView {
    pub items: Vec<String>,
    /// 수직 스크롤 오프셋 (픽셀, 0 = 최상단)
    pub scroll_offset: f32,
    pub item_height: f32,
    pub font_size: f32,
    pub color: [u8; 4],
    pub background_color: [f32; 4],
}

impl ScrollView {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            scroll_offset: 0.0,
            item_height: 24.0,
            font_size: 14.0,
            color: [200, 200, 200, 255],
            background_color: [0.10, 0.10, 0.15, 1.0],
        }
    }

    pub fn with_items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        self
    }

    pub fn with_item_height(mut self, h: f32) -> Self {
        self.item_height = h;
        self
    }

    /// scroll_offset 을 유효 범위로 클램프한다.
    pub fn clamp_scroll(&mut self, view_height: f32) {
        let total = self.items.len() as f32 * self.item_height;
        let max_offset = (total - view_height).max(0.0);
        self.scroll_offset = self.scroll_offset.clamp(0.0, max_offset);
    }
}

impl Default for ScrollView {
    fn default() -> Self {
        Self::new()
    }
}
