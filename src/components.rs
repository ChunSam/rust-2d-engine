use glam::{Mat4, Quat, Vec2, Vec3};

// ─── 렌더 컴포넌트 ────────────────────────────────────────────────────────────

/// 위치·크기·회전을 담는 컴포넌트
#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Vec2,
    pub scale: Vec2,
    /// 회전 각도 (라디안, Z축)
    pub rotation: f32,
    // z 가 클수록 화면에 위로 그려짐 (그림은 작은 z 부터 큰 z 순서로).
    pub z: f32,
}

impl Transform {
    pub fn new(position: Vec2, scale: Vec2, rotation: f32) -> Self {
        Self {
            position,
            scale,
            rotation,
            z: 0.0,
        }
    }

    /// ECS → GPU에 넘길 4×4 모델 행렬 생성
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            Vec3::new(self.scale.x, self.scale.y, 1.0),
            Quat::from_rotation_z(self.rotation),
            Vec3::new(self.position.x, self.position.y, 0.0),
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            scale: Vec2::ONE * 64.0,
            rotation: 0.0,
            z: 0.0,
        }
    }
}

/// 스프라이트 외형을 담는 컴포넌트
#[derive(Debug, Clone)]
pub struct Sprite {
    /// 텍스처 파일 경로 (None이면 단색 사각형)
    pub texture: Option<String>,
    /// RGBA 색상 배율 (흰색 = 텍스처 원본)
    pub color: [f32; 4],
}

impl Sprite {
    pub fn colored(r: f32, g: f32, b: f32) -> Self {
        Self {
            texture: None,
            color: [r, g, b, 1.0],
        }
    }

    pub fn textured(path: impl Into<String>) -> Self {
        Self {
            texture: Some(path.into()),
            color: [1.0; 4],
        }
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Self::colored(1.0, 1.0, 1.0)
    }
}

// ─── 하위 호환 재수출 ─────────────────────────────────────────────────────────
// resources.rs로 이동한 타입들을 engine::components::* 경로로도 접근할 수 있도록 유지.
pub use crate::resources::{FontData, GameState, PendingResize, ShouldQuit, ViewportSize, WindowConfig};
pub use crate::animation::player::{AnimationClip, AnimationPlayer, UvRect};

// ─── 단위 테스트 ───────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_default_z_is_zero() {
        assert_eq!(Transform::default().z, 0.0);
    }

    #[test]
    fn transform_z_assignable() {
        let mut t = Transform::default();
        t.z = 5.0;
        assert_eq!(t.z, 5.0);
    }
}
