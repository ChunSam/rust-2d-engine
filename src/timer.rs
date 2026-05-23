/// 카운트다운 또는 반복 타이머.
///
/// # 사용 예
/// ```rust
/// use engine::Timer;
///
/// let mut t = Timer::once(2.0);
/// t.tick(1.5);
/// assert!(!t.finished());
/// t.tick(0.6);
/// assert!(t.finished());
/// ```
#[derive(Clone, Debug)]
pub struct Timer {
    duration: f32,
    elapsed: f32,
    repeating: bool,
    just_finished: bool,
}

impl Timer {
    /// 지정한 시간(초) 후 한 번만 완료되는 타이머.
    pub fn once(duration: f32) -> Self {
        Self { duration, elapsed: 0.0, repeating: false, just_finished: false }
    }

    /// 지정한 시간(초)마다 반복 완료되는 타이머.
    pub fn repeating(duration: f32) -> Self {
        Self { duration, elapsed: 0.0, repeating: true, just_finished: false }
    }

    /// dt만큼 진행한다. 매 프레임 시스템에서 호출한다.
    pub fn tick(&mut self, dt: f32) {
        if self.finished() {
            self.just_finished = false;
            return;
        }
        self.elapsed += dt;
        if self.elapsed >= self.duration {
            self.just_finished = true;
            if self.repeating {
                self.elapsed -= self.duration;
            } else {
                self.elapsed = self.duration;
            }
        } else {
            self.just_finished = false;
        }
    }

    /// 타이머가 완료됐는지 (반복 타이머는 항상 false).
    pub fn finished(&self) -> bool {
        !self.repeating && self.elapsed >= self.duration
    }

    /// 이 tick에서 방금 완료됐는지 (반복 포함, 1 프레임만 true).
    pub fn just_finished(&self) -> bool {
        self.just_finished
    }

    /// 경과 시간(초).
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// 전체 지속 시간(초).
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// 진행률 0.0 ~ 1.0.
    pub fn fraction(&self) -> f32 {
        if self.duration <= 0.0 {
            1.0
        } else {
            (self.elapsed / self.duration).min(1.0)
        }
    }

    /// 타이머를 처음 상태로 되돌린다.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.just_finished = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn once_finishes() {
        let mut t = Timer::once(1.0);
        t.tick(0.5);
        assert!(!t.finished());
        assert!(!t.just_finished());
        t.tick(0.6);
        assert!(t.finished());
        assert!(t.just_finished());
        // 완료 후 tick해도 just_finished는 false
        t.tick(0.1);
        assert!(t.finished());
        assert!(!t.just_finished());
    }

    #[test]
    fn repeating_wraps() {
        let mut t = Timer::repeating(1.0);
        t.tick(1.1);
        assert!(!t.finished());
        assert!(t.just_finished());
        assert!((t.elapsed() - 0.1).abs() < 1e-5);
    }

    #[test]
    fn fraction_clamps() {
        let mut t = Timer::once(2.0);
        t.tick(1.0);
        assert!((t.fraction() - 0.5).abs() < 1e-5);
        t.tick(5.0);
        assert!((t.fraction() - 1.0).abs() < 1e-5);
    }
}
