/// 체크박스 컴포넌트.
///
/// `UiNode` 와 함께 엔티티에 붙여 사용한다.
/// `UiSystem` 이 클릭 시 `checked` 를 토글하고 `UiEvent::CheckBoxToggled` 를 발행한다.
///
/// 렌더: \[박스\] 라벨텍스트
///
/// # 예제
/// ```ignore
/// let entity = world.spawn();
/// world.insert(entity, UiNode::new(50.0, 200.0, 160.0, 24.0));
/// world.insert(entity, CheckBox::new("사운드 켜기"));
/// ```
pub struct CheckBox {
    pub checked: bool,
    pub label: String,
    pub checked_color: [f32; 4],
    pub unchecked_color: [f32; 4],
    pub border_color: [f32; 4],
    pub text_color: [u8; 4],
    pub font_size: f32,
    /// 체크박스 정사각형 한 변의 크기(픽셀). UiNode 높이보다 작아야 한다.
    pub box_size: f32,
}

impl CheckBox {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            checked: false,
            label: label.into(),
            checked_color: [0.28, 0.56, 0.90, 1.0],
            unchecked_color: [0.18, 0.18, 0.22, 1.0],
            border_color: [0.50, 0.52, 0.62, 1.0],
            text_color: [210, 210, 220, 255],
            font_size: 16.0,
            box_size: 20.0,
        }
    }
}
