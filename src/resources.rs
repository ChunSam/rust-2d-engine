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

/// 현재 게임 좌표계 기준 뷰포트 크기.
///
/// 네이티브 Retina/HiDPI 환경에서는 GPU 서피스가 물리 픽셀이고 게임 좌표는
/// 논리 픽셀이다. 이 값을 논리 픽셀로 유지해야 스프라이트와 UI가 의도한 크기로 보인다.
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

/// 논리 픽셀 1개가 몇 물리 픽셀인지 나타내는 배율.
#[derive(Debug, Clone, Copy)]
pub struct DisplayScaleFactor(pub f32);

impl Default for DisplayScaleFactor {
    fn default() -> Self {
        Self(1.0)
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

// ─── 렌더링 최적화 ──────────────────────────────────────────────────────────

/// 뷰 프러스텀 컬링 + 거리 기반 LOD 설정.
///
/// `App::run()` 전에 삽입하거나, 시스템 내에서 `world.resource_mut::<CullConfig>()` 로 조작.
/// 삽입하지 않으면 엔진 기본값(`frustum_culling: true, min_pixel_size: 0.0`)이 적용된다.
///
/// ```text
/// world.insert_resource(CullConfig {
///     frustum_culling: true,
///     min_pixel_size: 1.0,  // 화면 1px 미만 스프라이트 스킵
/// });
/// ```
#[derive(Debug, Clone, Copy)]
pub struct CullConfig {
    /// true이면 카메라 뷰포트 밖 스프라이트를 GPU 제출 전에 컬링한다.
    pub frustum_culling: bool,
    /// 화면 픽셀 단위 스프라이트 크기(min(w,h))가 이 값 미만이면 렌더링 스킵.
    /// `0.0`이면 거리 LOD 비활성화.
    pub min_pixel_size: f32,
}

impl Default for CullConfig {
    fn default() -> Self {
        Self {
            frustum_culling: true,
            min_pixel_size: 0.0,
        }
    }
}

// ─── 라이팅 ─────────────────────────────────────────────────────────────────

/// 씬 전체 환경광 리소스.
///
/// `world.insert_resource(AmbientLight::default())` 으로 등록하면
/// `LightingRenderer`가 활성화된다. `PointLight` 컴포넌트와 함께 사용한다.
///
/// ```rust,no_run
/// # use engine::{App, AmbientLight};
/// # let mut app = App::new();
/// app.world.insert_resource(AmbientLight {
///     color: [0.2, 0.2, 0.3],
///     intensity: 0.05,
/// });
/// ```
#[derive(Debug, Clone, Copy)]
pub struct AmbientLight {
    /// 환경광 RGB 색상 (0.0~1.0)
    pub color: [f32; 3],
    /// 0.0 = 완전 어두움, 1.0 = 원본 밝기
    pub intensity: f32,
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0],
            intensity: 0.1,
        }
    }
}

/// Inspector에서 현재 선택된 엔티티를 World 리소스로 노출한다.
///
/// `App`이 매 프레임 `inspector_selected`와 동기화한다.
/// 시스템에서 읽어 선택 강조, 경로 계획 등 에디터 연동에 사용한다.
///
/// ```text
/// if let Some(e) = world.resource::<SelectedEntity>().and_then(|s| s.0) {
///     // e 가 현재 Inspector에서 선택된 엔티티
/// }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct SelectedEntity(pub Option<crate::ecs::world::Entity>);

// ─── 프로파일러 ─────────────────────────────────────────────────────────────

/// 시스템 하나의 프로파일링 항목.
#[derive(Debug, Clone, Default)]
pub struct SystemProfile {
    pub name: String,
    /// 직전 프레임 실행 시간 (마이크로초).
    pub last_us: u64,
    /// 최근 60프레임 지수 이동 평균 (마이크로초).
    pub avg_us: f32,
}

/// 렌더러 패스 통계.
#[derive(Debug, Clone, Copy, Default)]
pub struct RenderStats {
    /// 텍스처 전환 횟수 (draw call 수).
    pub draw_calls: u32,
    /// GPU에 제출된 스프라이트 인스턴스 수.
    pub sprites_rendered: u32,
    /// 뷰 컬링/LOD로 스킵된 스프라이트 수.
    pub sprites_culled: u32,
}

/// 프로파일러 전체 데이터. `App`이 매 프레임 갱신하고 Engine Stats 패널이 읽는다.
#[derive(Debug, Clone, Default)]
pub struct ProfilerData {
    pub systems: Vec<SystemProfile>,
    pub render: RenderStats,
    /// 전체 프레임 시간 (ms).
    pub frame_ms: f32,
}

impl ProfilerData {
    /// EMA α = 1/60
    const ALPHA: f32 = 1.0 / 60.0;

    /// 시스템 실행 결과를 기록한다. idx 가 범위를 벗어나면 자동 확장.
    pub fn record_system(&mut self, idx: usize, name: &str, elapsed_us: u64) {
        if idx >= self.systems.len() {
            self.systems.resize(idx + 1, SystemProfile::default());
        }
        let s = &mut self.systems[idx];
        s.name = name.to_string();
        s.last_us = elapsed_us;
        s.avg_us = s.avg_us * (1.0 - Self::ALPHA) + elapsed_us as f32 * Self::ALPHA;
    }
}
