use glam::Vec2;

use crate::resources::ViewportSize;

/// UI 노드의 기준점. 뷰포트 모서리 또는 중심을 기준으로 위치를 계산한다.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Anchor {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,
    Center,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// 스크린 스페이스 UI 위치/크기 컴포넌트.
///
/// `offset` 은 `anchor` 기준점으로부터의 픽셀 오프셋이다.
/// 렌더링 순서(z)는 다른 UI 노드 간의 상대적 깊이를 결정한다.
pub struct UiNode {
    /// 앵커 기준점으로부터의 픽셀 오프셋 (좌상단 기준)
    pub offset: Vec2,
    /// 노드의 너비·높이 (픽셀)
    pub size: Vec2,
    /// 렌더링 깊이. 값이 클수록 앞에 그려진다 (0.0 ~ 1.0 권장)
    pub z: f32,
    pub anchor: Anchor,
    pub visible: bool,
}

impl UiNode {
    /// 좌상단 기준, z=0.9 기본값으로 노드를 생성한다.
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            offset: Vec2::new(x, y),
            size: Vec2::new(w, h),
            z: 0.9,
            anchor: Anchor::TopLeft,
            visible: true,
        }
    }

    pub fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn with_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    /// 뷰포트 크기를 받아 노드의 스크린 절대 좌상단 픽셀 좌표를 반환한다.
    pub fn screen_pos(&self, viewport: &ViewportSize) -> Vec2 {
        let (vw, vh) = (viewport.width, viewport.height);
        let (w, h) = (self.size.x, self.size.y);
        let base = match self.anchor {
            Anchor::TopLeft => Vec2::ZERO,
            Anchor::TopCenter => Vec2::new((vw - w) / 2.0, 0.0),
            Anchor::TopRight => Vec2::new(vw - w, 0.0),
            Anchor::Center => Vec2::new((vw - w) / 2.0, (vh - h) / 2.0),
            Anchor::BottomLeft => Vec2::new(0.0, vh - h),
            Anchor::BottomCenter => Vec2::new((vw - w) / 2.0, vh - h),
            Anchor::BottomRight => Vec2::new(vw - w, vh - h),
        };
        base + self.offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_anchor_positions_correctly() {
        let vp = ViewportSize {
            width: 800.0,
            height: 600.0,
        };
        let node = UiNode::new(0.0, 0.0, 200.0, 50.0).with_anchor(Anchor::Center);
        let pos = node.screen_pos(&vp);
        assert_eq!(pos, Vec2::new(300.0, 275.0));
    }

    #[test]
    fn bottom_right_anchor() {
        let vp = ViewportSize {
            width: 800.0,
            height: 600.0,
        };
        let node = UiNode::new(-10.0, -10.0, 100.0, 40.0).with_anchor(Anchor::BottomRight);
        let pos = node.screen_pos(&vp);
        assert_eq!(pos, Vec2::new(690.0, 550.0));
    }
}
