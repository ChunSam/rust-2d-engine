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

// ─── 블렌드 가중치 컴포넌트 ───────────────────────────────────────────────────

/// 크로스페이드 진행도를 나타내는 컴포넌트. `AnimationSystem`이 매 프레임 갱신한다.
///
/// - `1.0`: 크로스페이드 없음 (또는 완료)
/// - `0.0 ~ 1.0`: 전환 진행 중 (0 = from 클립, 1 = to 클립)
///
/// 게임 코드에서 스프라이트 알파 보간 등에 활용할 수 있다.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlendWeight(pub f32);

// ─── 크로스페이드 상태 ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct CrossfadeState {
    pub to_clip: usize,
    pub to_frame: usize,
    pub to_timer: f32,
    pub elapsed: f32,
    pub duration: f32,
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
    pub(crate) crossfade: Option<CrossfadeState>,
}

impl AnimationPlayer {
    pub fn new(clips: Vec<AnimationClip>) -> Self {
        Self {
            clips,
            current_clip: 0,
            current_frame: 0,
            timer: 0.0,
            crossfade: None,
        }
    }

    /// 클립을 즉시 전환한다. 이미 재생 중인 클립이면 아무것도 하지 않는다.
    pub fn play(&mut self, clip_index: usize) {
        if self.current_clip != clip_index {
            self.current_clip = clip_index;
            self.current_frame = 0;
            self.timer = 0.0;
            self.crossfade = None;
        }
    }

    /// `duration`(초) 동안 부드럽게 크로스페이드하며 클립을 전환한다.
    ///
    /// 전환 중에는 `BlendWeight` 컴포넌트가 0.0→1.0으로 갱신된다.
    /// `duration <= 0.0`이면 즉시 전환(`play`와 동일).
    pub fn play_with_crossfade(&mut self, clip_index: usize, duration: f32) {
        if self.current_clip == clip_index {
            return;
        }
        if duration <= 0.0 {
            self.play(clip_index);
            return;
        }
        self.crossfade = Some(CrossfadeState {
            to_clip: clip_index,
            to_frame: 0,
            to_timer: 0.0,
            elapsed: 0.0,
            duration,
        });
    }

    /// 크로스페이드 진행도 [0.0..=1.0]. 전환 중이 아니면 `1.0`.
    pub fn blend_weight(&self) -> f32 {
        match &self.crossfade {
            None => 1.0,
            Some(cf) => (cf.elapsed / cf.duration).clamp(0.0, 1.0),
        }
    }

    /// 현재 크로스페이드 전환 중인지 여부.
    pub fn is_crossfading(&self) -> bool {
        self.crossfade.is_some()
    }

    /// 현재 프레임의 UV를 반환한다. 클립·프레임이 없으면 전체 텍스처를 사용한다.
    pub fn current_uv(&self) -> UvRect {
        self.clips
            .get(self.current_clip)
            .and_then(|c| c.frames.get(self.current_frame))
            .copied()
            .unwrap_or(UvRect::FULL)
    }

    /// 현재 클립이 끝났는지 반환한다. 루핑 클립은 항상 false.
    pub fn is_finished(&self) -> bool {
        let Some(clip) = self.clips.get(self.current_clip) else {
            return true;
        };
        if clip.looping || clip.frames.is_empty() {
            return false;
        }
        self.current_frame >= clip.frames.len() - 1
    }
}
