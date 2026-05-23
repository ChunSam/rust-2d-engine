use glam::Vec2;

// ─── 디버그 드로우 큐 ─────────────────────────────────────────────────────────

/// 충돌 디버그 시각화용 순수 데이터 사각형 (렌더러 타입 미포함).
#[derive(Debug, Clone, Copy)]
pub struct DebugRect {
    pub min: Vec2,
    pub max: Vec2,
    pub color: [f32; 4],
    pub z: f32,
}

/// 디버그 렌더링 큐. `CollisionDebugSystem`이 채우고, `App`이 drain해 `UiQueue`로 변환한다.
#[derive(Debug, Clone, Default)]
pub struct DebugDrawQueue {
    pub items: Vec<DebugRect>,
}

// ─── 게임 상태 리소스 ────────────────────────────────────────────────────────

/// 게임 상태 머신 값 (ECS 리소스로 삽입)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameState {
    Playing,
    Paused,
    GameOver,
}

/// 게임 루프 종료 요청 리소스. 시스템이 true 로 설정하면 App 이 다음 프레임에 종료.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShouldQuit(pub bool);

// ─── 뷰포트 / 창 설정 ────────────────────────────────────────────────────────

/// 현재 GPU 서피스(= 창 클라이언트 영역) 픽셀 크기.
#[derive(Debug, Clone, Copy)]
pub struct ViewportSize {
    pub width: f32,
    pub height: f32,
}

impl Default for ViewportSize {
    fn default() -> Self {
        Self {
            width: 1280.0,
            height: 720.0,
        }
    }
}

impl ViewportSize {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            width: w as f32,
            height: h as f32,
        }
    }
}

/// 창 초기 설정. App::run() 전에 삽입하면 해당 값으로 창이 열린다.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
    /// 배경 clear 색상 (RGBA, wgpu 선형 공간 f64).
    pub clear_color: [f64; 4],
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            title: "Game".to_string(),
            clear_color: [0.08, 0.08, 0.12, 1.0],
        }
    }
}

/// 게임이 사용할 폰트 바이트. App::run() 전에 삽입하면 TextRenderer 가 이를 사용한다.
pub struct FontData(pub Vec<u8>);

/// 해상도 변경 요청. 게임 시스템이 Some((w, h)) 으로 설정하면 App 이 창 크기를 조정한다.
#[derive(Debug, Clone, Copy, Default)]
pub struct PendingResize(pub Option<(u32, u32)>);
