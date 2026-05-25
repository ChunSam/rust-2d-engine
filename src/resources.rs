use glam::Vec2;

// ─── 패닉 복구 ──────────────────────────────────────────────────────────────

/// 패닉이 발생해 비활성화된 시스템 목록.
///
/// `App`이 `catch_unwind`로 패닉을 포착하면 해당 시스템 이름을 여기에 기록하고
/// 이후 프레임에서 해당 시스템을 건너뛴다.
///
/// ```rust,ignore
/// if let Some(ps) = world.resource::<PanickedSystems>() {
///     for name in &ps.disabled {
///         log::warn!("비활성화된 시스템: {name}");
///     }
/// }
/// ```
#[derive(Default)]
pub struct PanickedSystems {
    /// 패닉으로 비활성화된 시스템 이름 목록.
    pub disabled: Vec<String>,
}

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

// ─── 범용 디버그 드로우 API ──────────────────────────────────────────────────

/// 단일 디버그 도형.
#[derive(Debug, Clone)]
pub enum DebugShape {
    /// 축 정렬 사각형 (외곽선)
    Rect { min: Vec2, max: Vec2, color: [f32; 4] },
    /// 직선 (시작점 → 끝점, 두께 thickness px)
    Line { start: Vec2, end: Vec2, color: [f32; 4], thickness: f32 },
    /// 원 (24각형 근사)
    Circle { center: Vec2, radius: f32, color: [f32; 4] },
    /// 십자 마커 (두 직선 교차)
    Cross { pos: Vec2, size: f32, color: [f32; 4] },
}

/// 매 프레임 디버그 도형을 수집하는 리소스.
///
/// App이 렌더링 후 자동으로 `clear()`를 호출하므로 매 프레임 새로 그리면 된다.
///
/// # 사용 예
/// ```rust,ignore
/// // 시스템 내부에서
/// if let Some(dbg) = world.resource_mut::<DebugDraw>() {
///     dbg.rect(Vec2::new(0., 0.), Vec2::new(64., 64.), [1., 0., 0., 1.]);
///     dbg.circle(player_pos, 32., [0., 1., 0., 0.8]);
///     dbg.line(from, to, [1., 1., 0., 1.]);
/// }
/// ```
#[derive(Debug, Default)]
pub struct DebugDraw {
    pub(crate) shapes: Vec<DebugShape>,
}

impl DebugDraw {
    pub fn new() -> Self {
        Self::default()
    }

    /// 축 정렬 사각형 외곽선을 그린다.
    pub fn rect(&mut self, min: Vec2, max: Vec2, color: [f32; 4]) {
        self.shapes.push(DebugShape::Rect { min, max, color });
    }

    /// 직선을 그린다 (기본 두께 1.5px).
    pub fn line(&mut self, start: Vec2, end: Vec2, color: [f32; 4]) {
        self.shapes.push(DebugShape::Line {
            start,
            end,
            color,
            thickness: 1.5,
        });
    }

    /// 두께를 지정해 직선을 그린다.
    pub fn line_thick(&mut self, start: Vec2, end: Vec2, color: [f32; 4], thickness: f32) {
        self.shapes.push(DebugShape::Line {
            start,
            end,
            color,
            thickness,
        });
    }

    /// 원을 그린다 (24각형 근사).
    pub fn circle(&mut self, center: Vec2, radius: f32, color: [f32; 4]) {
        self.shapes.push(DebugShape::Circle {
            center,
            radius,
            color,
        });
    }

    /// 십자 마커를 그린다.
    pub fn cross(&mut self, pos: Vec2, size: f32, color: [f32; 4]) {
        self.shapes.push(DebugShape::Cross { pos, size, color });
    }

    /// 이번 프레임의 모든 도형을 지운다. App이 렌더링 후 자동 호출.
    pub fn clear(&mut self) {
        self.shapes.clear();
    }

    /// 수집된 도형 슬라이스.
    pub fn shapes(&self) -> &[DebugShape] {
        &self.shapes
    }
}

#[cfg(test)]
mod debug_draw_tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn debug_draw_accumulates_shapes() {
        let mut dbg = DebugDraw::new();
        dbg.rect(Vec2::ZERO, Vec2::ONE * 64., [1., 0., 0., 1.]);
        dbg.circle(Vec2::new(100., 100.), 32., [0., 1., 0., 1.]);
        dbg.line(Vec2::ZERO, Vec2::new(50., 50.), [0., 0., 1., 1.]);
        assert_eq!(dbg.shapes().len(), 3);
    }

    #[test]
    fn debug_draw_clear_empties() {
        let mut dbg = DebugDraw::new();
        dbg.rect(Vec2::ZERO, Vec2::ONE, [1.; 4]);
        dbg.clear();
        assert!(dbg.shapes().is_empty());
    }

    #[test]
    fn debug_draw_cross_is_correct_shape() {
        let mut dbg = DebugDraw::new();
        dbg.cross(Vec2::new(50., 50.), 20., [1.; 4]);
        assert_eq!(dbg.shapes().len(), 1);
        matches!(&dbg.shapes()[0], DebugShape::Cross { .. });
    }

    #[test]
    fn debug_draw_line_thick() {
        let mut dbg = DebugDraw::new();
        dbg.line_thick(Vec2::ZERO, Vec2::new(100., 0.), [1.; 4], 3.0);
        assert_eq!(dbg.shapes().len(), 1);
        if let DebugShape::Line { thickness, .. } = &dbg.shapes()[0] {
            assert_eq!(*thickness, 3.0);
        } else {
            panic!("expected Line shape");
        }
    }
}

// ─── 비동기 에셋 로딩 진행 상황 ───────────────────────────────────────────────

/// 비동기 에셋 로딩 진행 상황.
///
/// `App::load_image_async()`로 요청한 이미지의 로딩 완료 비율을 추적한다.
///
/// # 사용 예
/// ```rust,ignore
/// let prog = world.resource::<LoadProgress>().unwrap();
/// draw_bar(prog.fraction()); // 0.0 ~ 1.0
/// if prog.is_complete() { /* 로딩 완료 → 게임 씬으로 전환 */ }
/// ```
#[derive(Debug, Clone, Default)]
pub struct LoadProgress {
    /// 총 비동기 로드 요청 수.
    pub total: usize,
    /// 완료 수 (Loaded 또는 Failed 포함).
    pub loaded: usize,
}

impl LoadProgress {
    /// 0.0 ~ 1.0 사이의 진행률을 반환한다. 요청이 없으면 1.0.
    pub fn fraction(&self) -> f32 {
        if self.total == 0 {
            1.0
        } else {
            self.loaded as f32 / self.total as f32
        }
    }

    /// 모든 비동기 로드가 완료되었으면 true.
    pub fn is_complete(&self) -> bool {
        self.loaded >= self.total
    }
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

// ─── 씬 전환 페이드 이펙트 ──────────────────────────────────────────────────

/// 씬 전환 페이드 이펙트 리소스.
///
/// `FadeState`를 설정하면 App이 자동으로 전체 화면에 색상 오버레이를 애니메이션한다.
///
/// # 사용 예
/// ```rust,ignore
/// // 검정으로 페이드 아웃 (0.5초)
/// world.insert_resource(FadeTransition::fade_out(0.5));
///
/// // 현재 색에서 투명으로 페이드 인 (0.3초)
/// world.insert_resource(FadeTransition::fade_in(0.3));
/// ```
#[derive(Debug, Clone)]
pub struct FadeTransition {
    /// 현재 알파값 (0.0 = 투명, 1.0 = 완전 불투명)
    pub alpha: f32,
    /// 목표 알파값
    pub target_alpha: f32,
    /// 초당 알파 변화량
    pub speed: f32,
    /// 오버레이 RGB 색상
    pub color: [f32; 3],
    /// 페이드 완료 여부 (App이 매 프레임 업데이트)
    pub finished: bool,
}

impl FadeTransition {
    /// 투명 → 불투명으로 페이드 아웃 (화면이 어두워짐)
    pub fn fade_out(duration: f32) -> Self {
        Self {
            alpha: 0.0,
            target_alpha: 1.0,
            speed: 1.0 / duration.max(0.001),
            color: [0.0, 0.0, 0.0],
            finished: false,
        }
    }

    /// 불투명 → 투명으로 페이드 인 (화면이 밝아짐)
    pub fn fade_in(duration: f32) -> Self {
        Self {
            alpha: 1.0,
            target_alpha: 0.0,
            speed: 1.0 / duration.max(0.001),
            color: [0.0, 0.0, 0.0],
            finished: false,
        }
    }

    /// 커스텀 색상으로 페이드
    pub fn with_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = [r, g, b];
        self
    }

    /// 알파값을 dt 초 진행한다. App이 자동 호출.
    pub fn update(&mut self, dt: f32) {
        if self.finished {
            return;
        }
        let diff = self.target_alpha - self.alpha;
        let step = self.speed * dt;
        if diff.abs() <= step {
            self.alpha = self.target_alpha;
            self.finished = true;
        } else {
            self.alpha += diff.signum() * step;
        }
    }
}

impl Default for FadeTransition {
    fn default() -> Self {
        Self {
            alpha: 0.0,
            target_alpha: 0.0,
            speed: 1.0,
            color: [0.0, 0.0, 0.0],
            finished: true,
        }
    }
}

#[cfg(test)]
mod fade_tests {
    use super::*;

    #[test]
    fn fade_out_starts_at_zero() {
        let f = FadeTransition::fade_out(1.0);
        assert_eq!(f.alpha, 0.0);
        assert_eq!(f.target_alpha, 1.0);
        assert!(!f.finished);
    }

    #[test]
    fn fade_update_reaches_target() {
        let mut f = FadeTransition::fade_out(0.5); // speed = 2.0/sec
        f.update(0.6); // > duration → should finish
        assert_eq!(f.alpha, 1.0);
        assert!(f.finished);
    }

    #[test]
    fn fade_update_partial() {
        let mut f = FadeTransition::fade_out(1.0); // speed = 1.0/sec
        f.update(0.3);
        assert!((f.alpha - 0.3).abs() < 1e-5);
        assert!(!f.finished);
    }

    #[test]
    fn fade_finished_does_not_update() {
        let mut f = FadeTransition::default(); // finished = true
        f.update(1.0);
        assert_eq!(f.alpha, 0.0); // no change
    }
}
