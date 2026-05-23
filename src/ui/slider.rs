/// 수평 슬라이더 컴포넌트.
///
/// `UiNode` 와 함께 엔티티에 붙여 사용한다.
/// `UiSystem` 이 드래그 입력을 처리하고 `UiEvent::SliderChanged` 를 발행한다.
///
/// # 예제
/// ```ignore
/// let entity = world.spawn();
/// world.insert(entity, UiNode::new(100.0, 300.0, 200.0, 20.0));
/// world.insert(entity, Slider::new(0.0, 100.0, 50.0));
/// ```
pub struct Slider {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    /// 드래그 중 여부 (내부 상태, 직접 수정 불필요).
    pub(crate) dragging: bool,
    pub track_color: [f32; 4],
    pub fill_color: [f32; 4],
    pub thumb_color: [f32; 4],
    pub thumb_hovered_color: [f32; 4],
    /// 썸 너비(픽셀). 높이는 UiNode.size.y 와 동일하게 렌더된다.
    pub thumb_width: f32,
}

impl Slider {
    pub fn new(min: f32, max: f32, value: f32) -> Self {
        Self {
            value: value.clamp(min, max),
            min,
            max,
            dragging: false,
            track_color:         [0.20, 0.20, 0.25, 1.0],
            fill_color:          [0.28, 0.52, 0.82, 1.0],
            thumb_color:         [0.70, 0.70, 0.82, 1.0],
            thumb_hovered_color: [0.90, 0.90, 1.00, 1.0],
            thumb_width: 14.0,
        }
    }

    /// 현재 값을 [0.0, 1.0] 으로 정규화한다.
    pub fn normalized(&self) -> f32 {
        let range = self.max - self.min;
        if range.abs() < f32::EPSILON {
            0.0
        } else {
            (self.value - self.min) / range
        }
    }

    /// 정규화 값 t ∈ [0, 1] 로 실제 값을 설정한다.
    pub(crate) fn set_normalized(&mut self, t: f32) {
        self.value = self.min + t.clamp(0.0, 1.0) * (self.max - self.min);
    }
}
