use glam::{Mat4, Quat, Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::asset::{Handle, ImageAsset};
use crate::reflect::{Reflect, ReflectValue};

// ─── 렌더 컴포넌트 ────────────────────────────────────────────────────────────

/// 위치·크기·회전을 담는 컴포넌트
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprite {
    /// 텍스처 파일 경로 (None이면 단색 사각형). RON 직렬화 지원.
    pub texture: Option<String>,
    /// RGBA 색상 배율 (흰색 = 텍스처 원본)
    pub color: [f32; 4],
    /// AssetServer를 통해 로드한 이미지 핸들. 직렬화 제외 — 런타임 전용.
    /// `texture`보다 우선 적용된다.
    #[serde(skip)]
    pub image_handle: Option<Handle<ImageAsset>>,
}

impl Sprite {
    pub fn colored(r: f32, g: f32, b: f32) -> Self {
        Self {
            texture: None,
            color: [r, g, b, 1.0],
            image_handle: None,
        }
    }

    pub fn textured(path: impl Into<String>) -> Self {
        Self {
            texture: Some(path.into()),
            color: [1.0; 4],
            image_handle: None,
        }
    }

    /// AssetServer 핸들로 텍스처를 지정한다. `texture` 경로보다 우선 적용된다.
    pub fn with_handle(handle: Handle<ImageAsset>) -> Self {
        Self {
            texture: None,
            color: [1.0; 4],
            image_handle: Some(handle),
        }
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Self::colored(1.0, 1.0, 1.0)
    }
}

// ─── Reflect 구현 ─────────────────────────────────────────────────────────────

impl Reflect for Transform {
    fn fields(&self) -> Vec<(&'static str, ReflectValue)> {
        vec![
            ("x", ReflectValue::F32(self.position.x)),
            ("y", ReflectValue::F32(self.position.y)),
            ("rotation", ReflectValue::F32(self.rotation)),
            ("scale_x", ReflectValue::F32(self.scale.x)),
            ("scale_y", ReflectValue::F32(self.scale.y)),
            ("z", ReflectValue::F32(self.z)),
        ]
    }
    fn set_field(&mut self, name: &str, val: ReflectValue) -> bool {
        match (name, val) {
            ("x", ReflectValue::F32(v)) => {
                self.position.x = v;
                true
            }
            ("y", ReflectValue::F32(v)) => {
                self.position.y = v;
                true
            }
            ("rotation", ReflectValue::F32(v)) => {
                self.rotation = v;
                true
            }
            ("scale_x", ReflectValue::F32(v)) => {
                self.scale.x = v;
                true
            }
            ("scale_y", ReflectValue::F32(v)) => {
                self.scale.y = v;
                true
            }
            ("z", ReflectValue::F32(v)) => {
                self.z = v;
                true
            }
            _ => false,
        }
    }
    fn type_name(&self) -> &'static str {
        "Transform"
    }
}

impl Reflect for Sprite {
    fn fields(&self) -> Vec<(&'static str, ReflectValue)> {
        vec![
            ("color", ReflectValue::Color(self.color)),
            (
                "texture",
                ReflectValue::String(self.texture.clone().unwrap_or_default()),
            ),
        ]
    }
    fn set_field(&mut self, name: &str, val: ReflectValue) -> bool {
        match (name, val) {
            ("color", ReflectValue::Color(c)) => {
                self.color = c;
                true
            }
            ("texture", ReflectValue::String(s)) => {
                self.texture = if s.is_empty() { None } else { Some(s) };
                true
            }
            _ => false,
        }
    }
    fn type_name(&self) -> &'static str {
        "Sprite"
    }
}

// ─── RenderLayer ──────────────────────────────────────────────────────────────

/// 스프라이트 렌더링 레이어 (선택 컴포넌트, 기본값 0).
///
/// 낮은 값이 먼저(뒤에) 그려진다. 같은 layer 안에서는
/// 텍스처 키 기준으로 배칭한 뒤 z 오름차순으로 렌더링한다.
///
/// # 예
/// ```
/// // 배경 레이어 (-1): 게임플레이보다 항상 뒤에 그려짐
/// // 기본 레이어  ( 0): 대부분의 게임오브젝트
/// // 전경 레이어  ( 1): HUD, 이펙트 등 항상 앞에 그려져야 하는 것들
/// world.add_component(bg, RenderLayer(-1));
/// world.add_component(effect, RenderLayer(1));
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct RenderLayer(pub i32);

// ─── 하위 호환 재수출 ─────────────────────────────────────────────────────────
// resources.rs로 이동한 타입들을 engine::components::* 경로로도 접근할 수 있도록 유지.
pub use crate::animation::player::{AnimationClip, AnimationPlayer, UvRect};
pub use crate::resources::{
    FontData, GameState, PendingResize, ShouldQuit, ViewportSize, WindowConfig,
};

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
