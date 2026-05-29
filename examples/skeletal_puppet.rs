//! 2D 컷아웃 스켈레탈 애니메이션 데모.
//!
//! 색 사각형 본만으로 휴머노이드 퍼펫을 구성한다(아트 에셋 불필요). 깊이 5의 본 체인
//! (hip→torso→upper_arm→forearm→hand)을 포함해 `HierarchySystem`의 임의 깊이 전파를 검증한다.
//!
//! 조작: Space = idle ↔ wave 토글, Esc = 종료.
//!
//! 실행: `cargo run --example skeletal_puppet`

use engine::{
    App, BoneKeyframe, BoneTrack, InputState, KeyCode, ShouldQuit, SkeletalAnimationSystem,
    SkeletalAnimator, SkeletalClip, SkeletonBuilder, Sprite, System, Transform, Vec2, WindowConfig,
    World,
};

/// 관절 본(scale=1, 스프라이트 없음)에 시각용 사각형을 자식으로 붙인다.
///
/// 관절 scale을 1로 유지하면 계층 합성에서 스케일이 곱해지며 폭발하지 않는다.
/// 시각 자식은 leaf이므로 자신의 크기(scale)만 가진다.
fn add_visual(
    builder: &mut SkeletonBuilder,
    world: &mut World,
    joint: &str,
    size: Vec2,
    offset: Vec2,
    color: [f32; 3],
) {
    builder.add_bone(
        world,
        format!("{joint}_visual"),
        joint,
        Transform {
            position: offset,
            scale: size,
            rotation: 0.0,
            z: 0.0,
        },
        Some(Sprite::colored(color[0], color[1], color[2])),
    );
}

/// 한 관절의 단일 트랙(회전만 키프레임)을 만든다.
fn rot_track(joint: &str, keys: &[(f32, f32)]) -> BoneTrack {
    BoneTrack {
        bone: joint.to_string(),
        keys: keys
            .iter()
            .map(|&(time, rot)| BoneKeyframe {
                time,
                position: Vec2::ZERO,
                rotation: rot,
                scale: Vec2::ONE,
            })
            .collect(),
    }
}

/// Space로 idle ↔ wave 토글, Esc로 종료.
struct ControlSystem;

impl System for ControlSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let (toggle, quit) = match world.resource::<InputState>() {
            Some(i) => (
                i.just_pressed(KeyCode::Space),
                i.just_pressed(KeyCode::Escape),
            ),
            None => (false, false),
        };
        if quit {
            if let Some(q) = world.resource_mut::<ShouldQuit>() {
                q.0 = true;
            }
        }
        if toggle {
            let roots: Vec<_> = world.query::<SkeletalAnimator>().map(|(e, _)| e).collect();
            for e in roots {
                if let Some(anim) = world.get_mut::<SkeletalAnimator>(e) {
                    let next = if anim.current == 0 { "wave" } else { "idle" };
                    anim.play_named(next);
                }
            }
        }
    }
}

fn main() {
    let mut app = App::new();
    app.world.insert_resource(WindowConfig {
        title: "skeletal puppet — Space: idle/wave, Esc: quit".to_string(),
        width: 960,
        height: 540,
        clear_color: [0.06, 0.07, 0.10, 1.0],
    });

    // ── 관절 본 (scale=1, 위치/회전만) ──────────────────────────────────────────
    let joint = |x: f32, y: f32| Transform {
        position: Vec2::new(x, y),
        scale: Vec2::ONE,
        rotation: 0.0,
        z: 0.0,
    };

    let mut b = SkeletonBuilder::new(&mut app.world, "hip", joint(480.0, 200.0));
    b.add_bone(&mut app.world, "torso", "hip", joint(0.0, 30.0), None);
    b.add_bone(&mut app.world, "head", "torso", joint(0.0, 95.0), None);
    // 오른팔 체인: 깊이 hip→torso→r_upper_arm→r_forearm→r_hand (5)
    b.add_bone(
        &mut app.world,
        "r_upper_arm",
        "torso",
        joint(35.0, 75.0),
        None,
    );
    b.add_bone(
        &mut app.world,
        "r_forearm",
        "r_upper_arm",
        joint(0.0, -45.0),
        None,
    );
    b.add_bone(
        &mut app.world,
        "r_hand",
        "r_forearm",
        joint(0.0, -40.0),
        None,
    );
    // 왼팔
    b.add_bone(
        &mut app.world,
        "l_upper_arm",
        "torso",
        joint(-35.0, 75.0),
        None,
    );
    b.add_bone(
        &mut app.world,
        "l_forearm",
        "l_upper_arm",
        joint(0.0, -45.0),
        None,
    );
    // 다리
    b.add_bone(&mut app.world, "l_leg", "hip", joint(-18.0, -10.0), None);
    b.add_bone(&mut app.world, "r_leg", "hip", joint(18.0, -10.0), None);

    // ── 시각용 사각형 ────────────────────────────────────────────────────────────
    let skin = [0.90, 0.78, 0.65];
    let shirt = [0.30, 0.55, 0.85];
    let pants = [0.25, 0.28, 0.35];
    add_visual(
        &mut b,
        &mut app.world,
        "torso",
        Vec2::new(50.0, 80.0),
        Vec2::new(0.0, 30.0),
        shirt,
    );
    add_visual(
        &mut b,
        &mut app.world,
        "head",
        Vec2::new(44.0, 44.0),
        Vec2::ZERO,
        skin,
    );
    add_visual(
        &mut b,
        &mut app.world,
        "r_upper_arm",
        Vec2::new(16.0, 45.0),
        Vec2::new(0.0, -22.0),
        shirt,
    );
    add_visual(
        &mut b,
        &mut app.world,
        "r_forearm",
        Vec2::new(14.0, 40.0),
        Vec2::new(0.0, -20.0),
        skin,
    );
    add_visual(
        &mut b,
        &mut app.world,
        "r_hand",
        Vec2::new(16.0, 16.0),
        Vec2::ZERO,
        skin,
    );
    add_visual(
        &mut b,
        &mut app.world,
        "l_upper_arm",
        Vec2::new(16.0, 45.0),
        Vec2::new(0.0, -22.0),
        shirt,
    );
    add_visual(
        &mut b,
        &mut app.world,
        "l_forearm",
        Vec2::new(14.0, 40.0),
        Vec2::new(0.0, -20.0),
        skin,
    );
    add_visual(
        &mut b,
        &mut app.world,
        "l_leg",
        Vec2::new(18.0, 60.0),
        Vec2::new(0.0, -30.0),
        pants,
    );
    add_visual(
        &mut b,
        &mut app.world,
        "r_leg",
        Vec2::new(18.0, 60.0),
        Vec2::new(0.0, -30.0),
        pants,
    );

    // ── 클립 ─────────────────────────────────────────────────────────────────────
    // idle: 몸통이 천천히 좌우로 흔들리고 팔은 살짝 흔들림 (looping)
    let idle = SkeletalClip {
        name: "idle".into(),
        duration: 2.0,
        looping: true,
        tracks: vec![
            rot_track("torso", &[(0.0, -0.04), (1.0, 0.04), (2.0, -0.04)]),
            rot_track("r_upper_arm", &[(0.0, 0.05), (1.0, -0.05), (2.0, 0.05)]),
            rot_track("l_upper_arm", &[(0.0, -0.05), (1.0, 0.05), (2.0, -0.05)]),
        ],
    };
    // wave: 오른팔을 들어 forearm을 좌우로 흔든다 (looping)
    let wave = SkeletalClip {
        name: "wave".into(),
        duration: 1.0,
        looping: true,
        tracks: vec![
            // 오른 위팔을 위로 들어올림 (약 -2.6 rad ≈ 머리 옆까지)
            rot_track("r_upper_arm", &[(0.0, -2.6), (1.0, -2.6)]),
            // forearm을 좌우로 흔들기
            rot_track("r_forearm", &[(0.0, -0.5), (0.5, 0.5), (1.0, -0.5)]),
            rot_track("torso", &[(0.0, 0.0), (0.5, 0.03), (1.0, 0.0)]),
        ],
    };

    b.finish(&mut app.world, vec![idle, wave]);

    app.add_system(SkeletalAnimationSystem);
    app.add_system(ControlSystem);
    app.run();
}
