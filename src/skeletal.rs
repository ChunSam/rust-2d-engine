//! 2D 컷아웃(리그드) 스켈레탈 애니메이션.
//!
//! 본은 계층 엔티티(`Transform` + [`Parent`](crate::hierarchy::Parent))이고, 각 본에
//! 스프라이트 조각을 붙인다. [`SkeletalClip`]은 본의 **로컬 `Transform`**을 키프레임으로
//! 움직이며, 이후 자동 실행되는 [`HierarchySystem`](crate::hierarchy::HierarchySystem)이
//! `GlobalTransform`을 합성한다. 렌더러는 `GlobalTransform`을 우선 사용하므로 본 스프라이트는
//! 별도 렌더 변경 없이 그려진다.
//!
//! ```no_run
//! use engine::{App, SkeletonBuilder, SkeletalClip, BoneTrack, BoneKeyframe,
//!     SkeletalAnimationSystem, components::{Transform, Sprite}};
//! use glam::Vec2;
//!
//! let mut app = App::new();
//! let mut b = SkeletonBuilder::new(&mut app.world, "hip",
//!     Transform { position: Vec2::new(480.0, 270.0), ..Default::default() });
//! b.add_bone(&mut app.world, "torso", "hip",
//!     Transform { position: Vec2::new(0.0, 40.0), ..Default::default() },
//!     Some(Sprite::colored(0.8, 0.7, 0.6)));
//! let clip = SkeletalClip {
//!     name: "idle".into(), duration: 1.0, looping: true,
//!     tracks: vec![BoneTrack { bone: "torso".into(), keys: vec![
//!         BoneKeyframe { time: 0.0, position: Vec2::new(0.0, 40.0), rotation: 0.0, scale: Vec2::ONE },
//!         BoneKeyframe { time: 0.5, position: Vec2::new(0.0, 40.0), rotation: 0.2, scale: Vec2::ONE },
//!         BoneKeyframe { time: 1.0, position: Vec2::new(0.0, 40.0), rotation: 0.0, scale: Vec2::ONE },
//!     ]}],
//! };
//! b.finish(&mut app.world, vec![clip]);
//! app.add_system(SkeletalAnimationSystem);
//! ```

use std::collections::HashMap;
use std::f32::consts::{PI, TAU};

use glam::Vec2;

use crate::components::{Sprite, Transform};
use crate::ecs::{Entity, System, World};
use crate::hierarchy::attach;

/// 한 본의 한 시점 포즈(로컬 `Transform` 값).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoneKeyframe {
    /// 클립 시작 기준 시간(초).
    pub time: f32,
    pub position: Vec2,
    /// 라디안, Z축.
    pub rotation: f32,
    pub scale: Vec2,
}

/// 한 본의 키프레임 트랙. `keys`는 `time` 오름차순이어야 한다.
#[derive(Debug, Clone)]
pub struct BoneTrack {
    /// [`SkeletalAnimator`]의 본 이름 맵 키.
    pub bone: String,
    pub keys: Vec<BoneKeyframe>,
}

impl BoneTrack {
    /// 주어진 시간의 포즈를 보간한다. 위치/스케일은 선형, 회전은 최단 경로 각도 보간.
    ///
    /// 키가 없으면 `None`, 범위를 벗어나면 양 끝 키로 클램프한다.
    pub fn sample(&self, time: f32) -> Option<(Vec2, f32, Vec2)> {
        if self.keys.is_empty() {
            return None;
        }
        if time <= self.keys[0].time {
            let k = &self.keys[0];
            return Some((k.position, k.rotation, k.scale));
        }
        let last = self.keys.last().unwrap();
        if time >= last.time {
            return Some((last.position, last.rotation, last.scale));
        }
        // time을 감싸는 두 키 [a, b] 탐색
        let i = self
            .keys
            .iter()
            .position(|k| k.time > time)
            .unwrap_or(self.keys.len() - 1);
        let a = &self.keys[i - 1];
        let b = &self.keys[i];
        let span = b.time - a.time;
        let t = if span > f32::EPSILON {
            (time - a.time) / span
        } else {
            0.0
        };
        Some((
            a.position.lerp(b.position, t),
            lerp_angle(a.rotation, b.rotation, t),
            a.scale.lerp(b.scale, t),
        ))
    }
}

/// 본 트랙 모음으로 구성된 하나의 스켈레탈 애니메이션 클립.
#[derive(Debug, Clone)]
pub struct SkeletalClip {
    pub name: String,
    /// 클립 길이(초).
    pub duration: f32,
    pub looping: bool,
    pub tracks: Vec<BoneTrack>,
}

/// 스켈레톤 루트 엔티티에 부착하는 애니메이터 컴포넌트.
///
/// [`SkeletalAnimationSystem`]이 매 프레임 `time`을 진행하고, 현재 클립의 각 트랙을 샘플해
/// 해당 본 엔티티의 로컬 `Transform`을 갱신한다.
#[derive(Debug, Clone)]
pub struct SkeletalAnimator {
    pub clips: Vec<SkeletalClip>,
    pub current: usize,
    /// 현재 클립 내 재생 시간(초).
    pub time: f32,
    /// 재생 속도 배율.
    pub speed: f32,
    pub playing: bool,
    /// 본 이름 → 엔티티. [`SkeletonBuilder`]가 채운다.
    pub bones: HashMap<String, Entity>,
}

impl SkeletalAnimator {
    pub fn new(clips: Vec<SkeletalClip>, bones: HashMap<String, Entity>) -> Self {
        Self {
            clips,
            current: 0,
            time: 0.0,
            speed: 1.0,
            playing: true,
            bones,
        }
    }

    /// 클립을 전환하고 시간을 0으로 되돌린다. 이미 재생 중인 클립이면 무시한다.
    pub fn play(&mut self, clip_index: usize) {
        if self.current != clip_index {
            self.current = clip_index;
            self.time = 0.0;
            self.playing = true;
        }
    }

    /// 이름으로 클립을 전환한다. 찾으면 `true`.
    pub fn play_named(&mut self, name: &str) -> bool {
        if let Some(i) = self.clips.iter().position(|c| c.name == name) {
            self.play(i);
            true
        } else {
            false
        }
    }

    /// non-looping 클립이 끝까지 재생됐는지. looping 클립은 항상 `false`.
    pub fn is_finished(&self) -> bool {
        match self.clips.get(self.current) {
            Some(c) => !c.looping && self.time >= c.duration,
            None => true,
        }
    }
}

/// 매 프레임 [`SkeletalAnimator`]를 진행하고 본의 로컬 `Transform`을 갱신하는 시스템.
///
/// 유저 시스템 단계에서 실행되어야 한다(`app.add_system(SkeletalAnimationSystem)`).
/// 이후 자동 실행되는 `HierarchySystem`이 `GlobalTransform`을 합성한다.
pub struct SkeletalAnimationSystem;

impl System for SkeletalAnimationSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        let animators: Vec<Entity> = world.query::<SkeletalAnimator>().map(|(e, _)| e).collect();

        for animator_entity in animators {
            // 1) 시간 진행 + 샘플 수집 (animator 빌림은 이 블록 안에서 끝낸다)
            let samples: Vec<(Entity, Vec2, f32, Vec2)> = {
                let Some(anim) = world.get_mut::<SkeletalAnimator>(animator_entity) else {
                    continue;
                };
                if !anim.playing {
                    continue;
                }
                let Some(clip) = anim.clips.get(anim.current) else {
                    continue;
                };
                let duration = clip.duration;
                anim.time += dt * anim.speed;
                if clip.looping {
                    if duration > f32::EPSILON {
                        anim.time = anim.time.rem_euclid(duration);
                    }
                } else if anim.time >= duration {
                    anim.time = duration;
                }
                let time = anim.time;

                // borrow 우회: 트랙을 다시 읽어 (bone_entity, TRS) 목록을 만든다
                let clip = &anim.clips[anim.current];
                clip.tracks
                    .iter()
                    .filter_map(|track| {
                        let bone = *anim.bones.get(&track.bone)?;
                        let (p, r, s) = track.sample(time)?;
                        Some((bone, p, r, s))
                    })
                    .collect()
            };

            // 2) animator 빌림 해제 후 각 본 Transform 갱신
            for (bone, position, rotation, scale) in samples {
                if let Some(t) = world.get_mut::<Transform>(bone) {
                    t.position = position;
                    t.rotation = rotation;
                    t.scale = scale;
                }
            }
        }
    }
}

/// 본 계층을 스폰하고 이름→엔티티 맵을 구성하는 저작 헬퍼.
///
/// 내부적으로 [`attach`](crate::hierarchy::attach)를 사용해 `Parent`/`Children`을 관리한다.
pub struct SkeletonBuilder {
    root: Entity,
    bones: HashMap<String, Entity>,
}

impl SkeletonBuilder {
    /// 루트 본을 스폰한다. `root_transform`은 스켈레톤 전체의 월드 기준 위치다.
    pub fn new(world: &mut World, root_name: impl Into<String>, root_transform: Transform) -> Self {
        let root = world.spawn();
        world.add_component(root, root_transform);
        let mut bones = HashMap::new();
        bones.insert(root_name.into(), root);
        Self { root, bones }
    }

    /// 루트 엔티티.
    pub fn root(&self) -> Entity {
        self.root
    }

    /// 이미 추가된 본 엔티티를 이름으로 조회한다.
    pub fn bone(&self, name: &str) -> Option<Entity> {
        self.bones.get(name).copied()
    }

    /// 본을 추가하고 `parent_name` 본에 붙인다. `sprite`가 있으면 함께 부착한다.
    ///
    /// `parent_name`이 아직 없으면 루트에 붙인다.
    pub fn add_bone(
        &mut self,
        world: &mut World,
        name: impl Into<String>,
        parent_name: &str,
        local_transform: Transform,
        sprite: Option<Sprite>,
    ) -> Entity {
        let parent = self.bones.get(parent_name).copied().unwrap_or(self.root);
        let bone = world.spawn();
        world.add_component(bone, local_transform);
        if let Some(s) = sprite {
            world.add_component(bone, s);
        }
        attach(world, bone, parent);
        self.bones.insert(name.into(), bone);
        bone
    }

    /// 루트에 [`SkeletalAnimator`]를 삽입하고 루트 엔티티를 반환한다.
    pub fn finish(self, world: &mut World, clips: Vec<SkeletalClip>) -> Entity {
        world.add_component(self.root, SkeletalAnimator::new(clips, self.bones));
        self.root
    }
}

/// 최단 경로 각도 선형 보간(라디안).
fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let mut diff = (b - a).rem_euclid(TAU);
    if diff > PI {
        diff -= TAU;
    }
    a + diff * t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kf(time: f32, x: f32, rot: f32) -> BoneKeyframe {
        BoneKeyframe {
            time,
            position: Vec2::new(x, 0.0),
            rotation: rot,
            scale: Vec2::ONE,
        }
    }

    #[test]
    fn sample_clamps_outside_range() {
        let track = BoneTrack {
            bone: "b".into(),
            keys: vec![kf(0.0, 0.0, 0.0), kf(1.0, 10.0, 0.0)],
        };
        assert_eq!(track.sample(-1.0).unwrap().0.x, 0.0);
        assert_eq!(track.sample(2.0).unwrap().0.x, 10.0);
    }

    #[test]
    fn sample_interpolates_midpoint() {
        let track = BoneTrack {
            bone: "b".into(),
            keys: vec![kf(0.0, 0.0, 0.0), kf(2.0, 10.0, 0.0)],
        };
        let (p, _, _) = track.sample(1.0).unwrap();
        assert!((p.x - 5.0).abs() < 1e-3);
    }

    #[test]
    fn lerp_angle_takes_shortest_path() {
        // 350° → 10° 는 +20°가 최단경로(360을 넘지 않고 -340이 아님)
        let a = 350f32.to_radians();
        let b = 10f32.to_radians();
        let mid = lerp_angle(a, b, 0.5);
        // 중간값은 0°(=360°) 근처여야 한다
        let mid_deg = mid.to_degrees().rem_euclid(360.0);
        assert!(
            !(5.0..=355.0).contains(&mid_deg),
            "expected ~0deg, got {mid_deg}"
        );
    }

    #[test]
    fn empty_track_samples_none() {
        let track = BoneTrack {
            bone: "b".into(),
            keys: vec![],
        };
        assert!(track.sample(0.5).is_none());
    }

    #[test]
    fn system_drives_bone_transform_and_loops() {
        let mut world = World::new();
        let mut b = SkeletonBuilder::new(
            &mut world,
            "root",
            Transform {
                position: Vec2::ZERO,
                ..Default::default()
            },
        );
        b.add_bone(
            &mut world,
            "arm",
            "root",
            Transform {
                position: Vec2::ZERO,
                ..Default::default()
            },
            None,
        );
        let clip = SkeletalClip {
            name: "wave".into(),
            duration: 2.0,
            looping: true,
            tracks: vec![BoneTrack {
                bone: "arm".into(),
                keys: vec![kf(0.0, 0.0, 0.0), kf(2.0, 20.0, 0.0)],
            }],
        };
        let arm = b.bone("arm").unwrap();
        let root = b.finish(&mut world, vec![clip]);

        // 1초 진행 → arm.x ≈ 10
        SkeletalAnimationSystem.run(&mut world, 1.0);
        let x = world.get::<Transform>(arm).unwrap().position.x;
        assert!((x - 10.0).abs() < 1e-3, "expected 10, got {x}");

        // 추가 1.5초(누적 2.5 → 루프되어 0.5) → arm.x ≈ 5
        SkeletalAnimationSystem.run(&mut world, 1.5);
        let x = world.get::<Transform>(arm).unwrap().position.x;
        assert!((x - 5.0).abs() < 1e-3, "expected 5 after loop, got {x}");

        // play_named로 전환 확인
        let anim = world.get_mut::<SkeletalAnimator>(root).unwrap();
        assert!(anim.play_named("wave"));
        assert!(!anim.play_named("nope"));
    }
}
