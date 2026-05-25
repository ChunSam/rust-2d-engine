use super::world::World;

/// 매 프레임 실행되는 로직 단위
///
/// `dt` 는 직전 프레임과의 시간 차이(초).
/// 구조체를 impl System 하고 App에 등록하면 자동으로 호출된다.
pub trait System {
    fn run(&mut self, world: &mut World, dt: f32);

    /// 프로파일러 패널에 표시할 시스템 이름. 빈 문자열이면 "anonymous"로 표시된다.
    fn name(&self) -> &'static str {
        ""
    }
}
