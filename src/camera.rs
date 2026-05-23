use glam::{Mat4, Vec2};

/// 2D 카메라 리소스.
///
/// # 좌표 규약 (top-left anchored)
///
/// `position` 은 뷰포트의 **좌상단** 월드 좌표(픽셀 단위)를 가리킨다.
/// 보이는 영역:
///   - X: `[position.x, position.x + width / zoom]`
///   - Y: `[position.y, position.y + height / zoom]`  (Y 아래가 +)
///
/// 플레이어를 화면 중앙에 놓으려면:
///   `camera.position = player_pos - Vec2::new(viewport_w, viewport_h) / (2.0 * zoom)`
///
/// 기본값 `position = Vec2::ZERO, zoom = 1.0` 일 때
/// `view_proj(w, h)` 는 기존 `Mat4::orthographic_rh(0, w, h, 0, -1, 1)` 과 동일하다.
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    /// 뷰포트 좌상단 월드 좌표 (픽셀 단위)
    pub position: Vec2,
    /// 줌 배율. 1.0 = 정상, 2.0 = 2배 확대 (보이는 영역 절반)
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

impl Camera {
    pub fn new(position: Vec2, zoom: f32) -> Self {
        Self { position, zoom }
    }

    /// 화면(픽셀) 좌표를 월드 좌표로 변환한다.
    ///
    /// `screen_pos`: `InputState::cursor()` 가 반환하는 좌상단 기준 픽셀 좌표.
    /// `viewport_w/h`: `ViewportSize` 리소스의 width/height.
    ///
    /// 역연산: world = position + screen / zoom
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        Vec2::new(
            screen_pos.x / self.zoom + self.position.x,
            screen_pos.y / self.zoom + self.position.y,
        )
    }

    /// 뷰포트 크기 `(width, height)` 를 받아 MVP 용 직교 투영 행렬을 반환한다.
    ///
    /// left = position.x,  right = position.x + width/zoom
    /// top  = position.y,  bottom = position.y + height/zoom
    pub fn view_proj(&self, width: f32, height: f32) -> Mat4 {
        let left = self.position.x;
        let right = self.position.x + width / self.zoom;
        let top = self.position.y;
        let bottom = self.position.y + height / self.zoom;
        Mat4::orthographic_rh(left, right, bottom, top, -1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: f32 = 800.0;
    const H: f32 = 600.0;

    #[test]
    fn default_matches_legacy_ortho() {
        let got = Camera::default().view_proj(W, H);
        let expected = Mat4::orthographic_rh(0.0, W, H, 0.0, -1.0, 1.0);
        assert!(
            got.abs_diff_eq(expected, 1e-6),
            "default view_proj differs from legacy ortho\ngot:      {got:?}\nexpected: {expected:?}"
        );
    }

    #[test]
    fn camera_position_translates_view() {
        let cam = Camera::new(Vec2::new(100.0, 50.0), 1.0);
        let m = cam.view_proj(W, H);
        // 카메라가 (100, 50) 이동하면 월드 원점(0,0)은 화면 밖으로 나간다.
        // 즉, view_proj 는 default 와 달라야 한다.
        let default_m = Camera::default().view_proj(W, H);
        assert!(
            !m.abs_diff_eq(default_m, 1e-6),
            "camera position had no effect on view_proj"
        );
        // 직접 검증: 이동한 카메라의 left/right/top/bottom 확인
        let expected = Mat4::orthographic_rh(100.0, 100.0 + W, 50.0 + H, 50.0, -1.0, 1.0);
        assert!(
            m.abs_diff_eq(expected, 1e-6),
            "translated view_proj mismatch\ngot:      {m:?}\nexpected: {expected:?}"
        );
    }

    #[test]
    fn screen_to_world_no_offset_no_zoom() {
        let cam = Camera::default(); // position=(0,0), zoom=1
        let world = cam.screen_to_world(Vec2::new(100.0, 200.0));
        assert_eq!(world, Vec2::new(100.0, 200.0));
    }

    #[test]
    fn screen_to_world_with_camera_offset() {
        let cam = Camera::new(Vec2::new(50.0, 80.0), 1.0);
        let world = cam.screen_to_world(Vec2::new(0.0, 0.0));
        // 화면 좌상단(0,0)은 카메라 position과 같아야 한다
        assert_eq!(world, Vec2::new(50.0, 80.0));
    }

    #[test]
    fn screen_to_world_with_zoom() {
        let cam = Camera::new(Vec2::ZERO, 2.0);
        // zoom=2 → 화면 픽셀 1개 = 월드 0.5 단위
        let world = cam.screen_to_world(Vec2::new(100.0, 60.0));
        assert_eq!(world, Vec2::new(50.0, 30.0));
    }

    #[test]
    fn zoom_scales_visible_region() {
        let cam = Camera::new(Vec2::ZERO, 2.0);
        let m = cam.view_proj(W, H);
        // zoom=2 → 보이는 영역이 절반: right = W/2, bottom = H/2
        let expected = Mat4::orthographic_rh(0.0, W / 2.0, H / 2.0, 0.0, -1.0, 1.0);
        assert!(
            m.abs_diff_eq(expected, 1e-6),
            "zoom=2 view_proj mismatch\ngot:      {m:?}\nexpected: {expected:?}"
        );
    }
}
