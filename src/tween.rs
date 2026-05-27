use crate::timer::Timer;

/// 보간 곡선.
#[derive(Clone, Debug, Default)]
pub enum Easing {
    /// 선형
    #[default]
    Linear,
    /// 처음이 느리고 끝이 빠름
    EaseIn,
    /// 처음이 빠르고 끝이 느림
    EaseOut,
    /// 양끝이 느리고 중간이 빠름
    EaseInOut,
    /// 뒤로 당겼다가 앞으로 튕김 (시작 오버슈팅)
    EaseInBack,
    /// 앞으로 갔다가 조금 더 나가고 돌아옴 (끝 오버슈팅)
    EaseOutBack,
}

impl Easing {
    /// t(0.0 ~ 1.0)에 이징 곡선을 적용한 값을 반환한다.
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => t * (2.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            Easing::EaseInBack => {
                const C: f32 = 1.701_58;
                t * t * ((C + 1.0) * t - C)
            }
            Easing::EaseOutBack => {
                const C: f32 = 1.701_58;
                let s = t - 1.0;
                s * s * ((C + 1.0) * s + C) + 1.0
            }
        }
    }
}

/// f32 값을 시간에 따라 보간하는 트윈.
///
/// # 사용 예
/// ```rust
/// use engine::{Tween, Easing};
///
/// let mut tween = Tween::new(0.0, 100.0, 1.0).with_easing(Easing::EaseOut);
/// let v = tween.tick(0.5);
/// assert!(v > 50.0); // EaseOut은 초반이 빠름
/// ```
#[derive(Clone, Debug)]
pub struct Tween {
    start: f32,
    end: f32,
    timer: Timer,
    easing: Easing,
}

impl Tween {
    /// start에서 end까지 duration초 동안 선형 보간하는 트윈.
    pub fn new(start: f32, end: f32, duration: f32) -> Self {
        Self {
            start,
            end,
            timer: Timer::once(duration),
            easing: Easing::Linear,
        }
    }

    /// 이징 곡선을 설정한다 (빌더 패턴).
    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// dt만큼 진행하고 현재 보간값을 반환한다.
    pub fn tick(&mut self, dt: f32) -> f32 {
        self.timer.tick(dt);
        self.value()
    }

    /// 현재 보간값 (tick 없이 조회만).
    pub fn value(&self) -> f32 {
        let t = self.easing.apply(self.timer.fraction());
        self.start + (self.end - self.start) * t
    }

    /// 트윈이 완료됐는지.
    pub fn finished(&self) -> bool {
        self.timer.finished()
    }

    /// 진행률 0.0 ~ 1.0.
    pub fn fraction(&self) -> f32 {
        self.timer.fraction()
    }

    /// 트윈을 처음 상태로 되돌린다.
    pub fn reset(&mut self) {
        self.timer.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_midpoint() {
        let mut tw = Tween::new(0.0, 100.0, 2.0);
        let v = tw.tick(1.0);
        assert!((v - 50.0).abs() < 1e-4);
    }

    #[test]
    fn ease_out_faster_start() {
        let mut tw = Tween::new(0.0, 100.0, 1.0).with_easing(Easing::EaseOut);
        let v = tw.tick(0.5);
        assert!(v > 50.0, "EaseOut at t=0.5 should be > 50, got {v}");
    }

    #[test]
    fn finishes_at_end() {
        let mut tw = Tween::new(10.0, 20.0, 1.0);
        tw.tick(2.0);
        assert!(tw.finished());
        assert!((tw.value() - 20.0).abs() < 1e-4);
    }
}
