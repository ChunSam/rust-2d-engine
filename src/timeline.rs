//! 타임라인/컷씬 시스템 — 키프레임 기반 엔티티 애니메이션.
//!
//! # 사용 예
//! ```rust,ignore
//! use engine::{Timeline, TimelineSystem, Easing};
//! use glam::Vec2;
//!
//! let mut tl = Timeline::new(2.0);
//! tl.position.add(0.0, Vec2::ZERO, Easing::Linear);
//! tl.position.add(1.0, Vec2::new(200.0, 0.0), Easing::EaseInOut);
//! tl.position.add(2.0, Vec2::new(200.0, 200.0), Easing::EaseOut);
//! world.add_component(entity, tl);
//! app.add_system(TimelineSystem);
//! ```

use crate::tween::Easing;

// ── Lerp 트레잇 ─────────────────────────────────────────────────────────────

/// 두 값 사이를 선형 보간하는 트레잇.
pub trait Lerp: Clone {
    fn lerp(a: &Self, b: &Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(a: &f32, b: &f32, t: f32) -> f32 {
        a + (b - a) * t
    }
}

impl Lerp for glam::Vec2 {
    fn lerp(a: &glam::Vec2, b: &glam::Vec2, t: f32) -> glam::Vec2 {
        a.lerp(*b, t)
    }
}

impl Lerp for [f32; 4] {
    fn lerp(a: &[f32; 4], b: &[f32; 4], t: f32) -> [f32; 4] {
        [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
            a[3] + (b[3] - a[3]) * t,
        ]
    }
}

// ── Keyframe<T> ──────────────────────────────────────────────────────────────

/// 특정 시간에 특정 값을 갖도록 하는 키프레임.
#[derive(Debug, Clone)]
pub struct Keyframe<T: Clone> {
    pub time: f32,
    pub value: T,
    pub easing: Easing,
}

// ── Track<T> ─────────────────────────────────────────────────────────────────

/// 같은 타입의 키프레임 시퀀스. 시간 순으로 정렬 유지.
#[derive(Debug, Clone)]
pub struct Track<T: Clone + Lerp> {
    keyframes: Vec<Keyframe<T>>,
}

impl<T: Clone + Lerp> Track<T> {
    pub fn new() -> Self {
        Self {
            keyframes: Vec::new(),
        }
    }

    pub fn with_keyframes(keyframes: Vec<Keyframe<T>>) -> Self {
        let mut track = Self { keyframes };
        track
            .keyframes
            .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        track
    }

    /// 키프레임을 추가한다. 삽입 후 시간 순으로 재정렬.
    pub fn add(&mut self, time: f32, value: T, easing: Easing) -> &mut Self {
        self.keyframes.push(Keyframe { time, value, easing });
        self.keyframes
            .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        self
    }

    /// 시간 `t`에서 보간된 값을 반환한다.
    /// 키프레임이 없으면 `None`, 범위 밖이면 첫/마지막 값을 클램프.
    pub fn sample(&self, t: f32) -> Option<T> {
        if self.keyframes.is_empty() {
            return None;
        }

        // Before first keyframe — clamp to first value
        if t <= self.keyframes[0].time {
            return Some(self.keyframes[0].value.clone());
        }

        // After last keyframe — clamp to last value
        let last = self.keyframes.last().unwrap();
        if t >= last.time {
            return Some(last.value.clone());
        }

        // Find the last keyframe with time <= t
        let idx = self.keyframes.iter().rposition(|kf| kf.time <= t).unwrap();
        let a = &self.keyframes[idx];
        let b = &self.keyframes[idx + 1];

        let span = b.time - a.time;
        let local_t = if span > 1e-6 {
            (t - a.time) / span
        } else {
            1.0
        };
        let eased_t = a.easing.apply(local_t);

        Some(T::lerp(&a.value, &b.value, eased_t))
    }

    /// 트랙의 전체 재생 시간 (마지막 키프레임의 time).
    pub fn duration(&self) -> f32 {
        self.keyframes.last().map(|kf| kf.time).unwrap_or(0.0)
    }

    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }
}

impl<T: Clone + Lerp> Default for Track<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ── Timeline 컴포넌트 ─────────────────────────────────────────────────────────

/// 엔티티에 붙이는 타임라인 컴포넌트. `TimelineSystem`이 매 프레임 구동한다.
///
/// # 예시
/// ```rust,ignore
/// let mut tl = Timeline::new(2.0); // 2초 타임라인
/// tl.position.add(0.0, Vec2::new(0., 0.), Easing::Linear);
/// tl.position.add(1.0, Vec2::new(200., 0.), Easing::EaseInOut);
/// tl.position.add(2.0, Vec2::new(200., 200.), Easing::EaseOut);
/// world.add_component(entity, tl);
/// app.add_system(TimelineSystem);
/// ```
#[derive(Debug, Clone)]
pub struct Timeline {
    /// 타임라인 전체 재생 시간 (초)
    pub duration: f32,
    /// 현재 재생 위치 (초)
    pub time: f32,
    /// 반복 재생 여부
    pub looping: bool,
    /// 재생 중 여부 (false이면 시스템이 시간을 진행하지 않음)
    pub playing: bool,

    // ── 트랙 ──────────────────────────────────────────────────────────────────
    pub position: Track<glam::Vec2>,
    pub rotation: Track<f32>,
    pub scale: Track<glam::Vec2>,
    pub color: Track<[f32; 4]>,
    pub alpha: Track<f32>,
}

impl Timeline {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            time: 0.0,
            looping: false,
            playing: true,
            position: Track::new(),
            rotation: Track::new(),
            scale: Track::new(),
            color: Track::new(),
            alpha: Track::new(),
        }
    }

    /// 반복 재생을 활성화한 채로 반환 (빌더 패턴).
    pub fn looping(mut self) -> Self {
        self.looping = true;
        self
    }

    pub fn play(&mut self) {
        self.playing = true;
    }
    pub fn pause(&mut self) {
        self.playing = false;
    }
    pub fn restart(&mut self) {
        self.time = 0.0;
        self.playing = true;
    }

    /// 비반복 타임라인이 끝까지 재생되었는지.
    pub fn is_finished(&self) -> bool {
        !self.looping && self.time >= self.duration
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new(1.0)
    }
}

// ── TimelineSystem ────────────────────────────────────────────────────────────

pub struct TimelineSystem;

impl crate::ecs::System for TimelineSystem {
    fn name(&self) -> &'static str {
        "TimelineSystem"
    }

    fn run(&mut self, world: &mut crate::ecs::World, dt: f32) {
        use crate::components::{Sprite, Transform};

        // Collect entities with Timeline first to avoid borrow conflicts
        let entities: Vec<crate::ecs::Entity> = world
            .query::<Timeline>()
            .map(|(e, _)| e)
            .collect();

        for entity in entities {
            // Take the timeline out to avoid double-borrow
            let mut tl = match world.take_component::<Timeline>(entity) {
                Some(t) => t,
                None => continue,
            };

            // Advance time
            if tl.playing && !tl.is_finished() {
                tl.time += dt;
                if tl.looping && tl.time > tl.duration {
                    tl.time -= tl.duration;
                } else {
                    tl.time = tl.time.min(tl.duration);
                }
            }

            let t = tl.time;

            // Apply position
            if let Some(pos) = tl.position.sample(t) {
                if let Some(transform) = world.get_mut::<Transform>(entity) {
                    transform.position = pos;
                }
            }

            // Apply rotation
            if let Some(rot) = tl.rotation.sample(t) {
                if let Some(transform) = world.get_mut::<Transform>(entity) {
                    transform.rotation = rot;
                }
            }

            // Apply scale
            if let Some(scale) = tl.scale.sample(t) {
                if let Some(transform) = world.get_mut::<Transform>(entity) {
                    transform.scale = scale;
                }
            }

            // Apply color
            if let Some(color) = tl.color.sample(t) {
                if let Some(sprite) = world.get_mut::<Sprite>(entity) {
                    sprite.color = color;
                }
            }

            // Apply alpha (overrides sprite.color[3])
            if let Some(alpha) = tl.alpha.sample(t) {
                if let Some(sprite) = world.get_mut::<Sprite>(entity) {
                    sprite.color[3] = alpha;
                }
            }

            // Put the timeline back
            world.add_component(entity, tl);
        }
    }
}

// ── 단위 테스트 ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tween::Easing;
    use glam::Vec2;

    #[test]
    fn track_sample_before_first_keyframe() {
        let mut track: Track<f32> = Track::new();
        track.add(1.0, 10.0, Easing::Linear);
        assert_eq!(track.sample(0.0), Some(10.0));
    }

    #[test]
    fn track_sample_after_last_keyframe() {
        let mut track: Track<f32> = Track::new();
        track.add(0.0, 0.0, Easing::Linear);
        track.add(1.0, 100.0, Easing::Linear);
        assert_eq!(track.sample(2.0), Some(100.0));
    }

    #[test]
    fn track_linear_interpolation() {
        let mut track: Track<f32> = Track::new();
        track.add(0.0, 0.0, Easing::Linear);
        track.add(1.0, 100.0, Easing::Linear);
        let v = track.sample(0.5).unwrap();
        assert!((v - 50.0).abs() < 1e-4, "expected ~50, got {v}");
    }

    #[test]
    fn track_empty_returns_none() {
        let track: Track<f32> = Track::new();
        assert!(track.sample(0.5).is_none());
    }

    #[test]
    fn timeline_is_finished_when_time_reached() {
        let mut tl = Timeline::new(1.0);
        tl.time = 1.0;
        assert!(tl.is_finished());
    }

    #[test]
    fn timeline_looping_not_finished() {
        let mut tl = Timeline::new(1.0).looping();
        tl.time = 1.5;
        assert!(!tl.is_finished());
    }

    #[test]
    fn track_vec2_lerp() {
        let mut track: Track<Vec2> = Track::new();
        track.add(0.0, Vec2::ZERO, Easing::Linear);
        track.add(1.0, Vec2::new(100.0, 0.0), Easing::Linear);
        let v = track.sample(0.25).unwrap();
        assert!((v.x - 25.0).abs() < 1e-4, "expected x≈25, got {}", v.x);
    }

    #[test]
    fn track_with_keyframes_sorted() {
        // Insert keyframes out of order — should still sort correctly
        let kfs = vec![
            Keyframe { time: 1.0, value: 100.0_f32, easing: Easing::Linear },
            Keyframe { time: 0.0, value: 0.0_f32, easing: Easing::Linear },
        ];
        let track = Track::with_keyframes(kfs);
        let v = track.sample(0.5).unwrap();
        assert!((v - 50.0).abs() < 1e-4, "expected ~50, got {v}");
    }

    #[test]
    fn track_duration() {
        let mut track: Track<f32> = Track::new();
        assert_eq!(track.duration(), 0.0);
        track.add(0.5, 1.0, Easing::Linear);
        track.add(2.0, 5.0, Easing::Linear);
        assert!((track.duration() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn timeline_default_playing() {
        let tl = Timeline::new(1.0);
        assert!(tl.playing);
        assert!(!tl.looping);
        assert!(!tl.is_finished());
    }

    #[test]
    fn timeline_restart_resets_time() {
        let mut tl = Timeline::new(1.0);
        tl.time = 0.9;
        tl.pause();
        tl.restart();
        assert_eq!(tl.time, 0.0);
        assert!(tl.playing);
    }
}
