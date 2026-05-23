// ─── UV 좌표 ──────────────────────────────────────────────────────────────────

/// 텍스처 내 한 프레임 영역을 UV 좌표로 표현
///
/// 예) 4열 2행 스프라이트시트의 (2열, 1행) 프레임:
/// `UvRect::from_grid(2, 1, 4, 2)`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UvRect {
    pub u_offset: f32,
    pub v_offset: f32,
    pub u_size: f32,
    pub v_size: f32,
}

impl UvRect {
    /// 텍스처 전체를 사용하는 기본값
    pub const FULL: Self = Self {
        u_offset: 0.0,
        v_offset: 0.0,
        u_size: 1.0,
        v_size: 1.0,
    };

    /// 그리드 형태 스프라이트시트에서 특정 프레임의 UV를 계산한다.
    pub fn from_grid(col: u32, row: u32, cols: u32, rows: u32) -> Self {
        let u_size = 1.0 / cols as f32;
        let v_size = 1.0 / rows as f32;
        Self {
            u_offset: col as f32 * u_size,
            v_offset: row as f32 * v_size,
            u_size,
            v_size,
        }
    }
}

// ─── 애니메이션 데이터 ────────────────────────────────────────────────────────

/// 하나의 애니메이션 클립: 프레임 목록과 재생 속도
#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub frames: Vec<UvRect>,
    pub fps: f32,
    pub looping: bool,
}

/// 엔티티에 붙이는 애니메이션 플레이어 컴포넌트
#[derive(Debug, Clone)]
pub struct AnimationPlayer {
    pub clips: Vec<AnimationClip>,
    pub current_clip: usize,
    pub current_frame: usize,
    /// 다음 프레임까지 누적된 시간(초)
    pub timer: f32,
}

impl AnimationPlayer {
    pub fn new(clips: Vec<AnimationClip>) -> Self {
        Self {
            clips,
            current_clip: 0,
            current_frame: 0,
            timer: 0.0,
        }
    }

    /// 클립을 전환한다. 이미 재생 중인 클립이면 아무것도 하지 않는다.
    pub fn play(&mut self, clip_index: usize) {
        if self.current_clip != clip_index {
            self.current_clip = clip_index;
            self.current_frame = 0;
            self.timer = 0.0;
        }
    }

    /// 현재 프레임의 UV를 반환한다. 클립·프레임이 없으면 전체 텍스처를 사용한다.
    pub fn current_uv(&self) -> UvRect {
        self.clips
            .get(self.current_clip)
            .and_then(|c| c.frames.get(self.current_frame))
            .copied()
            .unwrap_or(UvRect::FULL)
    }
}
