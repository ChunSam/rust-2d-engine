use crate::animation::player::UvRect;
use crate::asset::{Handle, ImageAsset};

/// 균일 그리드 텍스처 아틀라스.
///
/// 하나의 이미지 파일에 고정 크기 그리드로 여러 스프라이트를 배치한 텍스처.
/// `cols × rows` 타일로 나뉘며, 인덱스는 왼쪽 위(0)부터 오른쪽 아래 순서.
///
/// # 예시
/// 4×4 아틀라스: 총 16 타일, index 5 → row 1 col 1
#[derive(Clone, Debug)]
pub struct TextureAtlas {
    /// 아틀라스 전체 이미지 핸들
    pub handle: Handle<ImageAsset>,
    /// 가로 타일 수
    pub cols: u32,
    /// 세로 타일 수
    pub rows: u32,
}

impl TextureAtlas {
    /// index에 해당하는 UV 좌표 (0.0~1.0 정규화).
    ///
    /// index가 범위를 초과하면 `% (cols * rows)` 로 wrap한다.
    pub fn uv_rect(&self, index: u32) -> UvRect {
        let total = self.cols * self.rows;
        let index = if total == 0 { 0 } else { index % total };
        let col = index % self.cols;
        let row = index / self.cols;
        UvRect {
            u_offset: col as f32 / self.cols as f32,
            v_offset: row as f32 / self.rows as f32,
            u_size: 1.0 / self.cols as f32,
            v_size: 1.0 / self.rows as f32,
        }
    }

    /// 이 아틀라스의 이미지 파일 경로 (렌더러 텍스처 캐시 키).
    pub fn texture_path(&self) -> &str {
        self.handle.path()
    }
}

/// 텍스처 아틀라스의 특정 타일을 렌더링하는 컴포넌트.
///
/// `Transform` 컴포넌트와 함께 엔티티에 추가해 사용한다.
/// 기존 `Sprite` 컴포넌트와 동일한 렌더 패스에서 처리되므로
/// z-order, 블렌딩 등 모든 렌더링 동작이 동일하게 적용된다.
///
/// # 예시
/// ```rust,no_run
/// # use engine::{App, AtlasSprite, Transform};
/// # use glam::Vec2;
/// # let mut app = App::new();
/// let atlas = app.load_atlas("assets/characters.png", 4, 4);
/// let e = app.world.spawn();
/// app.world.add_component(e, Transform::default());
/// app.world.add_component(e, AtlasSprite::new(atlas, 3));
/// ```
#[derive(Clone, Debug)]
pub struct AtlasSprite {
    /// 아틀라스 핸들 (AssetServer에서 관리)
    pub atlas: Handle<TextureAtlas>,
    /// 아틀라스 내 타일 인덱스 (0-based, 왼쪽 위→오른쪽 아래)
    pub index: u32,
    /// RGBA 색상 배율 (기본값 흰색 = 원본 텍스처 색상)
    pub color: [f32; 4],
}

impl AtlasSprite {
    pub fn new(atlas: Handle<TextureAtlas>, index: u32) -> Self {
        Self {
            atlas,
            index,
            color: [1.0; 4],
        }
    }

    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}
