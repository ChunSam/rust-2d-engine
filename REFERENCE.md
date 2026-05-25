# rust-2d-engine 레퍼런스

> 버전 v0.44.0 기준. wgpu 기반 2D 게임 엔진.

---

## 목차

1. [빠른 시작](#빠른-시작)
2. [App](#app)
3. [ECS — World, Entity, System](#ecs--world-entity-system)
4. [씬 시스템](#씬-시스템)
5. [내장 리소스](#내장-리소스)
6. [입력](#입력)
7. [렌더링](#렌더링)
8. [애니메이션](#애니메이션)
9. [물리 (Rapier2D)](#물리-rapier2d)
10. [충돌 (경량 그리드)](#충돌-경량-그리드)
11. [오디오](#오디오)
12. [파티클](#파티클)
13. [타일맵](#타일맵)
14. [Timer / Tween](#timer--tween)
15. [카메라](#카메라)
16. [이벤트 시스템](#이벤트-시스템)
17. [UI 위젯](#ui-위젯)
18. [에셋 서버](#에셋-서버)
19. [씬 직렬화 (SceneDef)](#씬-직렬화-scenedef)
20. [저장/불러오기](#저장불러오기)
21. [경로 탐색 (A*)](#경로-탐색-a)
22. [렌더 레이어](#렌더-레이어)
23. [비헤이비어 트리](#비헤이비어-트리)
24. [Inspector Undo/Redo](#inspector-undoredo)
25. [스티어링 행동](#스티어링-행동)
26. [Blackboard](#blackboard)
27. [CommandBuffer](#commandbuffer)
28. [에디터 — 씬 그래프 패널](#에디터--씬-그래프-패널)
29. [에디터 — 컴포넌트 추가/제거](#에디터--컴포넌트-추가제거)
30. [Rhai 스크립팅](#rhai-스크립팅)
31. [2D 라이팅](#2d-라이팅)
32. [ECS 변경 감지](#ecs-변경-감지)
33. [2D 노멀 맵 라이팅](#2d-노멀-맵-라이팅)
34. [카메라 이펙트](#카메라-이펙트)
35. [오브젝트 풀](#오브젝트-풀)
36. [엔티티 복제](#엔티티-복제)
37. [Debug Draw API](#debug-draw-api)
38. [씬 전환 트랜지션](#씬-전환-트랜지션)
39. [타임라인/컷씬](#타임라인컷씬)
40. [좌표 규약](#좌표-규약)

---

## 빠른 시작

```rust
use engine::*;
use winit::keyboard::KeyCode;

struct MySystem;
impl System for MySystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        let input = world.resource::<InputState>().unwrap();
        if input.just_pressed(KeyCode::Escape) {
            world.resource_mut::<ShouldQuit>().unwrap().0 = true;
        }
    }
}

fn main() {
    let mut app = App::new();

    // 창 설정
    app.world.insert_resource(WindowConfig {
        width: 1280,
        height: 720,
        title: "My Game".to_string(),
        clear_color: [0.05, 0.05, 0.1, 1.0],
    });

    // 텍스처 로드
    app.load_texture("assets/player.png");

    // 엔티티 스폰
    let e = app.world.spawn();
    app.world.add_component(e, Transform {
        position: Vec2::new(400.0, 300.0),
        scale: Vec2::splat(64.0),
        rotation: 0.0,
        z: 0.0,
    });
    app.world.add_component(e, Sprite::textured("assets/player.png"));

    app.add_system(MySystem);
    app.run();
}
```

---

## App

`App`은 엔진의 진입점. 시스템·씬·리소스를 조립하고 `run()`으로 게임 루프를 시작한다.

```rust
let mut app = App::new();
```

| 메서드 | 설명 |
|---|---|
| `app.add_system(s)` | 시스템 등록. 매 프레임 등록 순서대로 실행됨 |
| `app.load_texture("path.png")` | PNG 텍스처 로드 예약 (`run()` 전에도 호출 가능) |
| `app.register_event::<E>()` | 이벤트 타입 등록 (자동 flush 포함) |
| `app.set_scene(Box<dyn Scene>)` | 씬을 즉시 전환 (`run()` 전·후 모두 가능) |
| `app.run()` | 이벤트 루프 시작 (블로킹) |

**프레임 실행 순서:**
1. 시스템들 순서대로 `run(dt)` 호출
2. 이벤트 큐 flush
3. 입력 상태 flush
4. 씬 전환 명령 처리
5. 렌더링 (스프라이트 → UI 사각형 → 텍스트)

---

## ECS — World, Entity, System

### World

모든 엔티티·컴포넌트·리소스를 보관하는 중심 저장소.

```rust
// 엔티티 생성/삭제
let e = world.spawn();
world.despawn(e);

// 컴포넌트 추가/조회/제거
world.add_component(e, MyComp { ... });
let c: Option<&MyComp> = world.get::<MyComp>(e);
let c: Option<&mut MyComp> = world.get_mut::<MyComp>(e);
world.remove_component::<MyComp>(e);  // 엔티티는 유지

// 리소스 (전역 싱글턴)
world.insert_resource(MyResource { ... });
let r: Option<&MyResource> = world.resource::<MyResource>();
let r: Option<&mut MyResource> = world.resource_mut::<MyResource>();

// 모든 엔티티 ID 슬라이스
let entities: &[Entity] = world.entities();
```

### 쿼리

| 메서드 | 반환 | 설명 |
|---|---|---|
| `world.query::<A>()` | `(Entity, &A)` | A를 가진 모든 엔티티 |
| `world.query2::<A, B>()` | `(Entity, &A, &B)` | A와 B를 모두 가진 엔티티 |
| `world.query3::<A, B, C>()` | `(Entity, &A, &B, &C)` | A·B·C 모두 |
| `world.query4::<A, B, C, D>()` | `(Entity, &A, &B, &C, &D)` | A·B·C·D 모두 |
| `world.query_opt2::<A, B>()` | `(Entity, &A, Option<&B>)` | A 필수, B는 있으면 Some |

쿼리는 `Iterator`를 반환하므로 `.for_each()`, `.collect()`, `.filter()` 등을 그대로 사용한다.

> **주의:** 같은 World에서 불변 쿼리 중 `get_mut`을 섞으면 borrow 충돌이 발생한다.
> 패턴: 먼저 `.collect()`로 엔티티 목록을 뽑은 뒤 순회하며 `get_mut`을 호출한다.

```rust
// 올바른 패턴
let targets: Vec<Entity> = world.query::<Enemy>().map(|(e, _)| e).collect();
for e in targets {
    if let Some(hp) = world.get_mut::<Health>(e) {
        hp.value -= 10;
    }
}
```

### 쿼리 필터 (With / Without)

```rust
// Sprite가 있는 Transform만
for (e, t) in world.query_with::<Transform, Sprite>() {
    // t: &Transform
}

// Enemy 컴포넌트가 없는 Transform만
for (e, t) in world.query_without::<Transform, Enemy>() {
    // t: &Transform
}
```

아키타입 레벨에서 `TypeId` 포함 여부를 판단하므로 per-entity 필터링보다 효율적이다.

### System 트레잇

```rust
use engine::{System, World};

struct MoveSystem;

impl System for MoveSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // dt: 이전 프레임과의 시간 차이 (초). 최대 0.1초로 클램프됨.
    }
}
```

---

## 씬 시스템

여러 게임 화면(메뉴, 플레이, 게임오버 등)을 씬 트레잇으로 분리한다.

### Scene 트레잇

```rust
use engine::{scene::{Scene, SceneCmd, SceneChange}, ecs::{System, World}};

struct GamePlay;

impl Scene for GamePlay {
    fn on_enter(&mut self, world: &mut World, systems: &mut Vec<Box<dyn System>>) {
        // 엔티티 스폰, 리소스 삽입, 시스템 등록
        systems.push(Box::new(MoveSystem));
    }

    fn on_exit(&mut self, world: &mut World) {
        // 정리 작업 (선택적)
    }
}
```

### 씬 전환

**앱 초기화 시:**
```rust
app.set_scene(Box::new(MenuScene));
```

**시스템 내부에서 (런타임):**
```rust
fn run(&mut self, world: &mut World, _dt: f32) {
    if let Some(sc) = world.resource_mut::<SceneChange>() {
        // 전체 교체 (월드 리셋 포함)
        sc.request(SceneCmd::Replace(Box::new(GamePlay)));

        // 현재 씬 위에 쌓기 (일시정지 메뉴 등)
        sc.request(SceneCmd::Push(Box::new(PauseMenu)));

        // 최상위 씬 제거
        sc.request(SceneCmd::Pop);
    }
}
```

| 명령 | 동작 |
|---|---|
| `Replace(scene)` | 스택 전부 종료 → 월드 리셋 → 새 씬 진입 |
| `Push(scene)` | 현재 월드 유지, 새 씬의 `on_enter`만 호출 |
| `Pop` | 최상위 씬 종료, 해당 씬이 등록한 시스템 제거 |

같은 프레임에 여러 번 `request`하면 **마지막 명령만** 실행된다.

---

## 내장 리소스

`App::new()`가 자동으로 삽입하는 리소스들.

| 타입 | 용도 |
|---|---|
| `InputState` | 키보드·마우스 입력 상태 |
| `GameState` | `Playing` / `Paused` / `GameOver` |
| `ShouldQuit(bool)` | `true`로 설정하면 다음 프레임에 앱 종료 |
| `Camera` | 뷰포트 오프셋·줌 |
| `ViewportSize` | 현재 창 픽셀 크기 (`width`, `height`) |
| `WindowConfig` | 창 제목·크기·배경색 (초기 설정용) |
| `TextQueue` | 텍스트 렌더링 큐 |
| `UiQueue` | UI 사각형 렌더링 큐 |
| `DebugDrawQueue` | 충돌 디버그 사각형 큐 |
| `SceneChange` | 씬 전환 명령 |

**사용자가 직접 삽입하는 리소스:**

```rust
// 폰트 (run() 전에 삽입)
app.world.insert_resource(FontData(std::fs::read("assets/font.ttf").unwrap()));

// 창 설정 (run() 전에 삽입)
app.world.insert_resource(WindowConfig {
    width: 1920, height: 1080,
    title: "My Game".into(),
    clear_color: [0.0, 0.0, 0.0, 1.0],
});

// 해상도 변경 요청 (런타임, 시스템 내부)
world.resource_mut::<PendingResize>().unwrap().0 = Some((1280, 720));
```

---

## 입력

`InputState` 리소스를 통해 키보드·마우스 입력을 읽는다.

```rust
use winit::keyboard::KeyCode;
use winit::event::MouseButton;

fn run(&mut self, world: &mut World, _dt: f32) {
    let input = world.resource::<InputState>().unwrap();

    // 키보드
    input.just_pressed(KeyCode::Space)     // 이번 프레임에 눌림 (1프레임만 true)
    input.is_pressed(KeyCode::ArrowLeft)   // 현재 누르고 있는 중
    input.just_released(KeyCode::Space)    // 이번 프레임에 뗌

    // 마우스
    input.cursor()                                   // Vec2 (스크린 픽셀 좌표)
    input.mouse_just_pressed(MouseButton::Left)      // 이번 프레임에 클릭
    input.is_mouse_pressed(MouseButton::Left)        // 현재 누르고 있는 중
    input.mouse_just_released(MouseButton::Left)     // 이번 프레임에 뗌
    input.scroll()                                   // f32, 프레임마다 초기화

    // 문자 입력 버퍼 (TextInput과 함께 사용)
    input.text_chars()   // &[char], '\x08'=Backspace '\n'=Enter, 프레임마다 초기화
}
```

> 커서 좌표는 스크린 픽셀 기준(좌상단 원점). 월드 좌표로 변환하려면 `Camera::screen_to_world()`를 사용한다.

### InputMap (키 리바인딩)

```rust
use engine::InputMap;
use winit::keyboard::KeyCode;

let mut map = InputMap::new();
map.bind("jump", KeyCode::Space);
map.bind("jump", KeyCode::KeyW);  // 멀티 바인딩 가능

// 시스템에서
let input = world.resource::<InputState>().unwrap();
if map.just_pressed("jump", input) { ... }
if map.is_pressed("move_left", input) { ... }
```

---

## 렌더링

### Transform + Sprite

모든 렌더링 가능한 오브젝트는 `Transform`과 `Sprite` 컴포넌트를 가진다.

```rust
// 텍스처 스프라이트
world.add_component(e, Transform {
    position: Vec2::new(100.0, 200.0),
    scale: Vec2::new(64.0, 64.0),  // 픽셀 크기
    rotation: 0.0,                  // 라디안 (Z축)
    z: 0.0,                         // 깊이 (클수록 앞에 그려짐)
});
world.add_component(e, Sprite::textured("assets/hero.png"));

// 단색 사각형
world.add_component(e, Sprite::colored(1.0, 0.0, 0.0));  // RGB

// 색상 배율 오버라이드
world.add_component(e, Sprite {
    texture: Some("assets/hero.png".into()),
    color: [1.0, 0.5, 0.5, 0.8],  // RGBA 배율 (텍스처와 곱해짐)
});
```

`z` 값이 클수록 앞에 그려진다. 같은 z값이면 스폰 순서에 따름.

### 텍스트 렌더링

`TextQueue` 리소스에 `DrawText`를 넣으면 다음 프레임에 렌더링된다.

```rust
use engine::{DrawText, TextQueue};

fn run(&mut self, world: &mut World, _dt: f32) {
    if let Some(tq) = world.resource_mut::<TextQueue>() {
        tq.push(DrawText {
            text: format!("HP: {}", self.hp),
            position: Vec2::new(10.0, 10.0),  // 스크린 좌표
            size: 24.0,                         // 폰트 크기 (픽셀)
            color: [1.0, 1.0, 1.0, 1.0],       // RGBA
        });
    }
}
```

> 텍스트는 스크린 스페이스에 그려진다 (카메라 영향 없음).

### UI 사각형

`UiQueue`에 `DrawRect`를 넣으면 스프라이트 위, 텍스트 아래 레이어에 그려진다.

```rust
use engine::{DrawRect, UiQueue};

let rect = DrawRect::new(x, y, width, height, [r, g, b, a]).with_z(0.9);
world.resource_mut::<UiQueue>().unwrap().push(rect);
```

---

## 애니메이션

스프라이트시트 애니메이션은 `AnimationPlayer` 컴포넌트로 처리한다.

```rust
use engine::{AnimationClip, AnimationPlayer, UvRect};

// 4열 2행 스프라이트시트, 첫 번째 행 = 달리기 (4프레임)
let run_clip = AnimationClip {
    frames: (0..4).map(|col| UvRect::from_grid(col, 0, 4, 2)).collect(),
    fps: 12.0,
    looping: true,
};

// 두 번째 행 = 점프 (4프레임)
let jump_clip = AnimationClip {
    frames: (0..4).map(|col| UvRect::from_grid(col, 1, 4, 2)).collect(),
    fps: 8.0,
    looping: false,
};

let player = AnimationPlayer::new(vec![run_clip, jump_clip]);
world.add_component(e, player);
world.add_component(e, Sprite::textured("assets/hero_sheet.png"));
```

시스템에서 클립 전환:

```rust
if let Some(anim) = world.get_mut::<AnimationPlayer>(entity) {
    anim.play(1);  // 클립 인덱스 0 = 달리기, 1 = 점프
}
```

`AnimationSystem`을 등록해야 프레임이 자동으로 진행된다:

```rust
app.add_system(AnimationSystem);
```

### 크로스페이드 전환

`play_with_crossfade(clip_index, duration)`으로 지정한 시간(초)에 걸쳐 부드럽게 전환한다.  
전환 중 `BlendWeight` 컴포넌트가 자동으로 갱신되며, 게임 코드에서 알파 보간 등에 활용할 수 있다.

```rust
use engine::{AnimationPlayer, BlendWeight};

// 0.2초 크로스페이드
if let Some(player) = world.get_mut::<AnimationPlayer>(entity) {
    player.play_with_crossfade(1, 0.2);
}

// 전환 진행도 읽기 (0.0 = 이전 클립, 1.0 = 새 클립 / 전환 없으면 1.0)
if let Some(bw) = world.get_mut::<BlendWeight>(entity) {
    // bw.0 : f32 [0.0 ~ 1.0]
}
```

### 1D 블렌드 트리

`BlendTree1D`는 float 파라미터에 따라 자동으로 클립을 선택하고 크로스페이드 전환을 수행한다.  
이동 속도 → 애니메이션 자동 전환 같은 패턴에 유용하다.

```rust
use engine::{BlendEntry, BlendTree1D, BlendTreeSystem};

// 트리 구성 (threshold 오름차순)
let tree = BlendTree1D::new(
    vec![
        BlendEntry { threshold: 0.0, clip_index: 0 },  // idle
        BlendEntry { threshold: 0.3, clip_index: 1 },  // walk
        BlendEntry { threshold: 1.2, clip_index: 2 },  // run
    ],
    0.15,  // 클립 전환 시 크로스페이드 지속 시간 (초)
);
world.add_component(entity, tree);

// 시스템 등록 (BlendTreeSystem은 AnimationSystem 이전에)
app.add_system(Box::new(BlendTreeSystem));
app.add_system(Box::new(AnimationSystem));

// 매 프레임 파라미터 갱신
if let Some(tree) = world.get_mut::<BlendTree1D>(entity) {
    tree.set_param(speed);  // speed에 따라 idle / walk / run 자동 전환
}
```

**등록 순서 요약**

```
BlendTreeSystem → AnimationSystem → StateMachineSystem
```

---

## 물리 (Rapier2D)

`PhysicsWorld`를 시스템 구조체에 직접 소유하거나 ECS 리소스로 삽입한다.

```rust
use engine::{PhysicsBody, PhysicsSystem, PhysicsWorld};
use glam::Vec2;

struct MyPhysicsSystem {
    physics: PhysicsWorld,
}

impl MyPhysicsSystem {
    fn new() -> Self {
        Self {
            physics: PhysicsWorld::new(Vec2::new(0.0, 9.8)),  // 중력
        }
    }
}
```

### 바디 추가

```rust
// 동적 박스 (중력 반응)
let (rb_handle, col_handle) = physics.add_dynamic_box(
    Vec2::new(100.0, 50.0),  // 위치
    16.0, 16.0,               // 반폭, 반높이
    true,                     // 회전 잠금
);

// 정적 박스 (바닥, 벽)
let (rb_handle, col_handle) = physics.add_static_box(
    Vec2::new(400.0, 600.0),
    400.0, 16.0,
);

// 동적 원
let (rb_handle, col_handle) = physics.add_dynamic_circle(
    Vec2::new(200.0, 100.0),
    20.0,   // 반지름
    true,
);
```

`PhysicsBody` 컴포넌트로 엔티티와 연결:

```rust
use engine::physics::body::PhysicsBody;

world.add_component(e, PhysicsBody {
    rigid_body_handle: rb_handle,
    collider_handle: col_handle,
});
```

### 시뮬레이션 진행 및 Transform 동기화

`PhysicsSystem`은 매 프레임 `PhysicsWorld`를 스텝하고 `Transform`을 갱신한다.

```rust
impl System for MyPhysicsSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        self.physics.step(dt);

        let bodies: Vec<(Entity, rapier2d::prelude::RigidBodyHandle)> = world
            .query::<PhysicsBody>()
            .map(|(e, b)| (e, b.rigid_body_handle))
            .collect();

        for (entity, handle) in bodies {
            if let Some(rb) = self.physics.rigid_body(handle) {
                let pos = rb.translation();
                if let Some(tr) = world.get_mut::<Transform>(entity) {
                    tr.position = Vec2::new(pos.x, pos.y);
                }
            }
        }
    }
}
```

### 바디 접근자

```rust
// 힘/속도 적용
if let Some(rb) = physics.rigid_body_mut(handle) {
    rb.set_linvel(rapier2d::math::Vector::new(100.0, 0.0), true);
    rb.apply_impulse(rapier2d::math::Vector::new(0.0, -500.0), true);
}

// 착지 판정
let on_ground = physics.has_contact(col_handle);

// 바디 제거
if let Some(body) = world.get::<PhysicsBody>(entity) {
    physics.remove_body(body);
}
world.despawn(entity);
```

---

## 충돌 (경량 그리드)

rapier2d 없이 AABB 기반 충돌 감지가 필요할 때 사용한다.

```rust
use engine::{Collider, CollisionLayer, CollisionGridSystem, SpatialGrid};

// 콜라이더 컴포넌트
world.add_component(e, Collider {
    half_w: 16.0,
    half_h: 16.0,
    layer: CollisionLayer::Player,
    enabled: true,
});

// SpatialGrid 리소스 삽입
world.insert_resource(SpatialGrid::new(64.0));  // 셀 크기

// 시스템 등록 (충돌 쌍을 SpatialGrid에 채워 넣음)
app.add_system(CollisionGridSystem);
```

충돌 쌍 읽기:

```rust
if let Some(grid) = world.resource::<SpatialGrid>() {
    for (a, b) in grid.pairs() {
        // a, b: Entity
    }
}
```

### 충돌 디버그 시각화

```rust
use engine::{CollisionDebugSystem, DebugConfig};

world.insert_resource(DebugConfig { draw_colliders: true });
app.add_system(CollisionDebugSystem);
```

---

## 오디오

`AudioManager`를 ECS 리소스로 삽입한다. 오디오 장치가 없어도 게임은 계속 실행된다.

```rust
if let Some(audio) = AudioManager::new() {
    app.world.insert_resource(audio);
}
```

### 재생

```rust
fn run(&mut self, world: &mut World, _dt: f32) {
    if let Some(audio) = world.resource_mut::<AudioManager>() {
        // 파일 재생 (WAV 권장)
        audio.play("bgm", "assets/bgm.wav", true);   // repeat=true → 무한 반복
        audio.play("sfx_jump", "assets/jump.wav", false);

        // 정지
        audio.stop("bgm");

        // 볼륨 (0.0 ~ 1.0)
        audio.set_volume("bgm", 0.5);

        // 스테레오 팬 (-1.0 = 좌, 0.0 = 중앙, 1.0 = 우)
        audio.set_pan("sfx_enemy", 0.8);

        // 순수 사인파 톤 (파일 없이 간단한 SFX)
        audio.play_tone("sfx_blip", 440.0, 0.1, 0.3);  // freq, duration, volume
    }
}
```

- 같은 채널에 `play`를 다시 호출하면 이전 소리를 **즉시 중단**하고 새 소리를 재생한다.
- `set_pan`은 **다음 `play()` 호출부터** 적용된다.

---

## 파티클

`ParticleEmitter` 컴포넌트를 엔티티에 붙이고 `ParticleSystem`을 등록한다.

```rust
use engine::{ParticleEmitter, ParticleSystem};
use glam::Vec2;

let emitter = world.spawn();
world.add_component(emitter, Transform {
    position: Vec2::new(200.0, 300.0),
    scale: Vec2::ONE,
    ..Default::default()
});
world.add_component(emitter, ParticleEmitter {
    spawn_rate: 30.0,                          // 초당 파티클 수
    lifetime: 1.5,                             // 생존 시간 (초)
    velocity: Vec2::new(0.0, -80.0),           // 기본 속도
    velocity_spread: Vec2::new(30.0, 20.0),    // 속도 랜덤 범위 (±)
    color_start: [1.0, 0.5, 0.0, 1.0],        // 시작 색상 RGBA
    color_end:   [1.0, 0.0, 0.0, 0.0],        // 끝 색상 (알파 0 → 페이드 아웃)
    size: Vec2::splat(6.0),
    texture: None,   // Some("assets/particle.png") 도 가능
    emit: true,
    ..Default::default()
});

app.add_system(ParticleSystem);
```

방출 중단/재개:

```rust
if let Some(em) = world.get_mut::<ParticleEmitter>(emitter) {
    em.emit = false;
}
```

---

## 타일맵

```rust
use engine::{Tilemap, TilemapAtlas, TilemapSystem};
use glam::Vec2;

let atlas = TilemapAtlas::new("assets/tileset.png", 8, 4);  // 8열 4행

let tiles = vec![
    vec![1, 1, 1, 1, 1],   // 0 = 빈 칸, 1+ = 타일 ID+1
    vec![1, 0, 0, 0, 1],
    vec![1, 0, 2, 0, 1],
    vec![1, 1, 1, 1, 1],
];

let map_entity = world.spawn();
world.add_component(map_entity, Tilemap::new(
    atlas,
    tiles,
    32.0,              // 타일 한 변 크기 (픽셀)
    Vec2::new(0.0, 0.0), // 타일맵 좌상단 세계 좌표
));

app.add_system(TilemapSystem::new());
```

`TilemapSystem`은 타일맵 엔티티가 처음 등장하면 타일 엔티티를 자동 스폰하고, 사라지면 자동 디스폰한다.

---

## Timer / Tween

### Timer

```rust
use engine::Timer;

// 1회 타이머
let mut t = Timer::once(2.0);   // 2초 후 완료

// 반복 타이머
let mut t = Timer::repeating(0.5);  // 0.5초마다 완료

// 시스템에서
t.tick(dt);
t.finished()        // 1회 타이머: 완료 여부
t.just_finished()   // 이번 프레임에 완료됐는지 (반복 포함, 1프레임만 true)
t.fraction()        // 0.0 ~ 1.0 진행률
t.elapsed()         // 경과 시간 (초)
t.reset()           // 처음으로 되돌리기
```

### Tween

f32 값을 시간에 따라 보간한다.

```rust
use engine::{Tween, Easing};

let mut tw = Tween::new(0.0, 100.0, 1.5)      // start, end, duration
    .with_easing(Easing::EaseOutBack);

// 시스템에서
let value = tw.tick(dt);   // 현재 보간값 반환 + 진행
tw.finished()              // 완료 여부
tw.value()                 // tick 없이 현재값 조회
tw.reset()                 // 처음으로 되돌리기
```

| Easing | 설명 |
|---|---|
| `Linear` | 선형 (기본값) |
| `EaseIn` | 처음이 느리고 끝이 빠름 |
| `EaseOut` | 처음이 빠르고 끝이 느림 |
| `EaseInOut` | 양끝이 느리고 중간이 빠름 |
| `EaseInBack` | 뒤로 당겼다가 앞으로 튕김 |
| `EaseOutBack` | 앞으로 갔다가 조금 더 나가고 돌아옴 |

---

## 카메라

`Camera` 리소스는 `App::new()`에서 자동 삽입된다.

```rust
// 카메라를 플레이어 중심으로 이동
fn run(&mut self, world: &mut World, _dt: f32) {
    let vp = world.resource::<ViewportSize>().unwrap();
    let (vw, vh) = (vp.width, vp.height);

    if let Some(tr) = world.get::<Transform>(self.player) {
        let player_pos = tr.position;
        if let Some(cam) = world.resource_mut::<Camera>() {
            cam.position = player_pos - Vec2::new(vw, vh) / (2.0 * cam.zoom);
        }
    }
}
```

### 좌표 변환

```rust
let cam = world.resource::<Camera>().unwrap();
let input = world.resource::<InputState>().unwrap();

// 마우스 커서 → 월드 좌표
let world_pos = cam.screen_to_world(input.cursor());
```

### 줌

```rust
if let Some(cam) = world.resource_mut::<Camera>() {
    cam.zoom = 2.0;  // 2배 확대 (보이는 영역 절반)
}
```

---

## 이벤트 시스템

타입 안전한 프레임 경계 이벤트 버스.

### 등록 (run() 전)

```rust
#[derive(Clone)]
enum GameEvent {
    EnemyKilled { entity: Entity, score: u32 },
    LevelComplete,
}

app.register_event::<GameEvent>();
```

### 송신

```rust
if let Some(events) = world.resource_mut::<Events<GameEvent>>() {
    events.send(GameEvent::EnemyKilled { entity: e, score: 100 });
}
```

### 수신

```rust
if let Some(events) = world.resource::<Events<GameEvent>>() {
    for ev in events.read() {
        match ev {
            GameEvent::EnemyKilled { entity, score } => { ... }
            GameEvent::LevelComplete => { ... }
        }
    }
}
```

- 이벤트는 **프레임 종료 시 자동으로 flush**된다.
- 송신한 프레임의 **이후 시스템**이나 **다음 프레임** 첫 번째 시스템에서 수신 가능하다.

---

## UI 위젯

스크린 스페이스 UI. `LayoutSystem`은 `UiSystem` **이전에** 등록해야 한다.

```rust
app.register_event::<UiEvent>();
app.add_system(LayoutSystem);  // Panel 자식 위치 계산
app.add_system(UiSystem);       // 렌더링 및 이벤트 발행
```

`Panel`을 사용하지 않는다면 `LayoutSystem` 등록은 생략 가능하다.

### Button

```rust
use engine::{Anchor, Button, UiNode};

let btn = world.spawn();
world.add_component(btn, UiNode::new(0.0, 0.0, 160.0, 50.0)
    .with_anchor(Anchor::BottomCenter)
    .with_z(0.9));
world.add_component(btn, Button {
    label: "Start".to_string(),
    font_size: 20.0,
    color_normal:   [0.2, 0.2, 0.8, 1.0],
    color_hovered:  [0.3, 0.3, 1.0, 1.0],
    color_pressed:  [0.1, 0.1, 0.6, 1.0],
    color_disabled: [0.4, 0.4, 0.4, 1.0],
    text_color: [1.0, 1.0, 1.0, 1.0],
    ..Default::default()
});
```

버튼 클릭 처리:

```rust
if let Some(events) = world.resource::<Events<UiEvent>>() {
    for ev in events.read() {
        if let UiEvent::ButtonClicked(entity) = ev {
            if *entity == btn {
                // 클릭 처리
            }
        }
    }
}
```

### TextInput

타이핑 가능한 텍스트 입력 필드. 클릭으로 포커스를 얻고 Enter로 제출한다.

```rust
use engine::{Anchor, TextInput, UiNode};

let field = world.spawn();
world.add_component(field, UiNode::new(100.0, 200.0, 240.0, 36.0)
    .with_anchor(Anchor::TopLeft));
world.add_component(field, TextInput::new("이름 입력...")
    .with_max_len(32)
    .with_font_size(16.0));
```

이벤트 수신:

```rust
if let Some(events) = world.resource::<Events<UiEvent>>() {
    for ev in events.read() {
        match ev {
            UiEvent::TextChanged(entity, text) => {
                // 문자가 입력/삭제될 때마다 발행
            }
            UiEvent::TextSubmitted(entity, text) => {
                // Enter 키 → 포커스 해제 + 최종 텍스트
            }
            UiEvent::TextFocused(entity) => { /* 클릭으로 포커스 획득 */ }
            UiEvent::TextBlurred(entity)  => { /* 포커스 잃음 */ }
            _ => {}
        }
    }
}
```

런타임에서 텍스트를 읽거나 초기화할 때:

```rust
if let Some(ti) = world.get_mut::<TextInput>(field) {
    let current = ti.text.clone();
    ti.text.clear();
    ti.cursor = 0;
}
```

### ScrollView

마우스 휠로 스크롤 가능한 텍스트 목록. 자식 엔티티 없이 내부 `items` Vec으로 렌더링한다.

```rust
use engine::{Anchor, ScrollView, UiNode};

let log = world.spawn();
world.add_component(log, UiNode::new(10.0, 10.0, 300.0, 200.0)
    .with_anchor(Anchor::TopLeft));
world.add_component(log, ScrollView::new()
    .with_items(vec!["항목 1".into(), "항목 2".into()])
    .with_item_height(24.0));
```

런타임에서 항목 추가:

```rust
if let Some(sv) = world.get_mut::<ScrollView>(log) {
    sv.items.push("새 로그 메시지".into());
    // 최하단으로 스크롤
    let total = sv.items.len() as f32 * sv.item_height;
    sv.scroll_offset = total;
}
```

### Panel (레이아웃 컨테이너)

자식 엔티티를 자동으로 세로 또는 가로로 배치한다.
`LayoutSystem`이 `UiSystem` 실행 전에 자식 `UiNode.offset`을 갱신한다.

```rust
use engine::{Anchor, Button, LayoutDir, Panel, UiNode};

// 패널 엔티티
let panel = world.spawn();
world.add_component(panel, UiNode::new(200.0, 150.0, 200.0, 180.0)
    .with_anchor(Anchor::TopLeft));

// 자식 버튼 3개
let btn1 = world.spawn();
world.add_component(btn1, UiNode::new(0.0, 0.0, 184.0, 44.0));  // 위치는 Panel이 덮어씀
world.add_component(btn1, Button::new("계속하기"));

let btn2 = world.spawn();
world.add_component(btn2, UiNode::new(0.0, 0.0, 184.0, 44.0));
world.add_component(btn2, Button::new("설정"));

let btn3 = world.spawn();
world.add_component(btn3, UiNode::new(0.0, 0.0, 184.0, 44.0));
world.add_component(btn3, Button::new("종료"));

// 패널에 자식 등록
world.add_component(panel, Panel::new(LayoutDir::Vertical)
    .with_gap(8.0)
    .with_padding(8.0));
if let Some(p) = world.get_mut::<Panel>(panel) {
    p.children = vec![btn1, btn2, btn3];
}
```

> `Panel`의 자식 엔티티는 `UiNode.anchor`와 `UiNode.offset`이 `LayoutSystem`에 의해 매 프레임 덮어씌워진다. 자식의 `size`는 그대로 유지된다.

### Label

```rust
use engine::{Anchor, Label, UiNode};

let lbl = world.spawn();
world.add_component(lbl, UiNode::new(10.0, 10.0, 200.0, 30.0)
    .with_anchor(Anchor::TopLeft));
world.add_component(lbl, Label {
    text: "Score: 0".to_string(),
    font_size: 18.0,
    color: [1.0, 1.0, 1.0, 1.0],
});

// 런타임 텍스트 변경
if let Some(lbl_comp) = world.get_mut::<Label>(lbl) {
    lbl_comp.text = format!("Score: {}", score);
}
```

### UiEvent 목록

| 이벤트 | 발행 시점 |
|---|---|
| `ButtonClicked(Entity)` | 버튼 클릭 (Pressed → Released in bounds) |
| `TextChanged(Entity, String)` | TextInput 문자 입력·삭제마다 |
| `TextSubmitted(Entity, String)` | TextInput Enter 키 |
| `TextFocused(Entity)` | TextInput 클릭으로 포커스 획득 |
| `TextBlurred(Entity)` | TextInput 포커스 잃음 (다른 곳 클릭 또는 Enter) |

### Anchor 기준점

| 값 | 위치 |
|---|---|
| `TopLeft` | 좌상단 (기본값) |
| `TopCenter` | 상단 중앙 |
| `TopRight` | 우상단 |
| `Center` | 화면 중앙 |
| `BottomLeft` | 좌하단 |
| `BottomCenter` | 하단 중앙 |
| `BottomRight` | 우하단 |

`offset`은 앵커 기준점으로부터의 픽셀 오프셋이다.

---

## 에셋 서버

`AssetServer` 리소스가 이미지·스크립트·아틀라스를 로드·캐싱·핫리로딩한다.  
`App::new()`가 자동으로 삽입하므로 직접 생성할 필요 없다.

### 이미지 로드

```rust
// App 레벨 (run() 전)
let handle: Handle<ImageAsset> = app.load_image("assets/player.png");

// 시스템 내부
let assets = world.resource_mut::<AssetServer>().unwrap();
let handle = assets.load_image("assets/enemy.png");

// Sprite에 적용
world.add_component(e, Sprite { texture: Some(handle.path().into()), ..Default::default() });
```

같은 경로를 다시 호출하면 캐시된 핸들을 반환한다 (파일 I/O 없음).

### 로드 상태 확인 (Phase 32)

파일이 없거나 디코딩에 실패하면 마젠타(1×1) 폴백 텍스처로 대체된다.  
`load_state()`로 성공/실패 여부와 실패 원인을 확인할 수 있다.

```rust
use engine::AssetLoadState;

let assets = world.resource::<AssetServer>().unwrap();
match assets.load_state(&handle) {
    AssetLoadState::Loaded => { /* 정상 */ }
    AssetLoadState::Failed(reason) => {
        eprintln!("텍스처 로드 실패: {reason}");
    }
}

// 실패한 핸들 ID 목록 (디버그용)
let failed: Vec<AssetId> = assets.failed_images();
```

핫 리로딩 후에도 상태가 갱신된다.

### 에셋 목록 조회

```rust
// 현재 로드된 이미지 목록 (에셋 브라우저용)
let list: Vec<ImageEntry> = assets.image_list();
for entry in &list {
    println!("{} — {}×{}", entry.path, entry.width, entry.height);
}

// id로 직접 조회
let img: Option<&ImageAsset> = assets.get_image_by_id(entry.id);
```

| 메서드 | 반환 | 설명 |
|---|---|---|
| `load_image(path)` | `Handle<ImageAsset>` | 로드 또는 캐시 반환 |
| `load_state(&handle)` | `AssetLoadState` | `Loaded` / `Failed(String)` |
| `failed_images()` | `Vec<AssetId>` | 실패한 이미지 id 목록 |
| `get_image(&handle)` | `Option<&ImageAsset>` | CPU 픽셀 데이터 |
| `image_list()` | `Vec<ImageEntry>` | 로드된 이미지 전체 목록 |
| `image_count()` | `usize` | 캐시된 이미지 수 |
| `load_script(path)` | `Handle<ScriptAsset>` | Rhai 스크립트 로드 |
| `load_atlas(path, cols, rows)` | `Handle<TextureAtlas>` | 텍스처 아틀라스 로드 |
| `poll_reloads()` | `Vec<String>` | 변경된 파일 경로 목록 (App이 매 프레임 호출) |

---

## 씬 직렬화 (SceneDef)

`SceneDef`는 레벨 전체를 RON 파일로 저장·복원하는 구조체다.

### 기본 사용

```rust
use engine::{SceneDef, EntityDef, spawn_scene_def, Transform, Sprite, Tag};
use std::path::Path;

// 씬 구성
let scene = SceneDef {
    entities: vec![
        EntityDef {
            tag: Some("ground".into()),
            transform: Some(Transform::new(Vec2::ZERO, Vec2::new(800.0, 32.0), 0.0)),
            sprite: Some(Sprite::colored(0.3, 0.6, 0.3)),
            parent: None,
        },
    ],
    ..SceneDef::default()
};

// 저장 (version 필드 자동 기록)
scene.save(Path::new("levels/level1.ron")).unwrap();

// 로드
let loaded = SceneDef::load(Path::new("levels/level1.ron")).unwrap();
let entities = spawn_scene_def(&mut world, &loaded);
```

### 스키마 버전 관리 (Phase 32)

`SceneDef`에 `version: u32` 필드가 있다. `save()`는 항상 현재 버전(`SCENE_DEF_VERSION = 1`)으로 저장한다.  
구 파일(version 필드 없음)을 로드하면 `version = 0`으로 역직렬화되며, 버전이 다를 경우 `log::warn`을 출력하고 로드를 계속한다.

```ron
// Phase 32 이후 저장된 파일 형식
SceneDef(
    version: 1,
    entities: [
        EntityDef(
            tag: Some("player"),
            transform: Some(Transform( ... )),
            sprite: None,
            parent: None,
        ),
    ],
)
```

```rust
// 버전 상수
use engine::SCENE_DEF_VERSION;  // 현재 = 1
```

### 계층 구조 (parent 필드)

```rust
EntityDef {
    tag: Some("child".into()),
    parent: Some("parent_tag".into()),  // 부모 엔티티의 tag 문자열
    ..Default::default()
}
```

`spawn_scene_def`는 2패스로 스폰한다: 1패스에서 모든 엔티티를 만들고, 2패스에서 parent 링크를 연결한다.

### Inspector에서 사용

에디터(F1) Inspector의 Entities 탭 하단:
- `Path:` 입력 필드에 경로 지정
- `📂 Load Scene` — RON을 읽어 현재 월드의 Transform 엔티티를 교체
- `💾 Save Scene` — 현재 월드를 RON으로 직렬화

---

## 저장/불러오기

`engine::save` 모듈은 RON 기반 세이브 파일을 제공한다.  
`serde::Serialize + serde::Deserialize`를 구현한 임의의 타입을 저장·복원할 수 있다.

```rust
use engine::save::{save, load, load_or_default, exists, delete, save_path};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default)]
struct GameProgress {
    level: u32,
    score: u64,
}

// OS 표준 데이터 디렉토리 아래 경로 생성
let path = save_path("my_game", "progress.ron");

// 저장
save(&path, &GameProgress { level: 3, score: 15000 }).unwrap();

// 불러오기
let progress: GameProgress = load(&path).unwrap();

// 파일 없으면 Default 반환
let progress: GameProgress = load_or_default(&path).unwrap();

// 존재 확인 / 삭제
if exists(&path) { delete(&path).unwrap(); }
```

| 함수 | 설명 |
|---|---|
| `save(path, &T)` | RON 직렬화 후 파일 저장. 부모 디렉토리 자동 생성 |
| `load(path)` | RON 파일 읽어 역직렬화. 파일 없으면 `SaveError::Io(NotFound)` |
| `load_or_default(path)` | 파일 없으면 `T::default()` 반환. 파싱 오류는 전파 |
| `exists(path)` | 파일 존재 여부 확인 |
| `delete(path)` | 파일 삭제. 파일 없으면 `Ok(())` |
| `save_path(app, file)` | OS 데이터 디렉토리 아래 경로 반환 |

---

## 경로 탐색 (A*)

`engine::pathfinding` 모듈은 격자 기반 A* 알고리즘을 제공한다.

### PathGrid

통행 가능 여부를 저장하는 격자. row-major `Vec<bool>` 배열.

```rust
use engine::{PathGrid, find_path};
use glam::IVec2;

// 20×15 격자 생성 (전부 통행 가능)
let mut grid = PathGrid::new(20, 15);

// 장애물 설정
grid.set_walkable(5, 3, false);
grid.set_walkable(5, 4, false);
grid.set_walkable(5, 5, false);

// 통행 가능 여부 확인 (범위 밖이면 false, 패닉 없음)
let ok = grid.is_walkable(5, 3); // false
```

| 메서드 | 설명 |
|---|---|
| `PathGrid::new(w, h)` | 전부 통행 가능 상태로 초기화 |
| `PathGrid::new_blocked(w, h)` | 전부 막힌 상태로 초기화 |
| `set_walkable(x, y, bool)` | 특정 셀 통행 가능 여부 설정 |
| `is_walkable(x, y)` | 통행 가능 여부 반환. 범위 밖 = `false` |

### find_path

```rust
let path: Option<Vec<IVec2>> = find_path(&grid, IVec2::new(0, 0), IVec2::new(19, 14));

match path {
    Some(steps) => {
        // steps: start 미포함, goal 포함
        for cell in &steps {
            println!("→ ({}, {})", cell.x, cell.y);
        }
    }
    None => println!("경로 없음"),
}
```

- 4방향 이동 (상하좌우). 대각선 없음.
- `start == goal` → `Some(vec![goal])`
- 목표 셀이 막혀 있으면 즉시 `None`
- 반환 경로는 **start 미포함, goal 포함**

### 타일맵과 연동 예

```rust
// Tilemap 크기에 맞춰 PathGrid 생성
let grid = PathGrid::new(tilemap.width as i32, tilemap.height as i32);
// ... 타일 종류에 따라 set_walkable 설정

// 적 AI: 매 프레임 또는 경로 갱신 주기마다 호출
if let Some(path) = find_path(&grid, enemy_tile, player_tile) {
    // path[0]이 다음 이동 목표 셀
}
```

---

## 렌더 레이어

`RenderLayer(i32)` 컴포넌트로 스프라이트의 렌더링 순서를 명시적으로 제어한다.

```rust
use engine::RenderLayer;

// 낮은 값 = 먼저 렌더 (배경)
world.add_component(bg_entity, RenderLayer(-10));

// 기본값 = 0
world.add_component(player_entity, RenderLayer(0));

// 높은 값 = 나중에 렌더 (전경 UI 효과 등)
world.add_component(fx_entity, RenderLayer(10));
```

### 배칭 동작

렌더러는 스프라이트를 `(layer, texture_key, z)` 순서로 정렬한 뒤 같은 텍스처가 연속되면 한 번의 드로우 콜로 묶는다.

| 정렬 키 | 의미 |
|---|---|
| `layer` (i32) | `RenderLayer` 값. 없으면 0 |
| `texture_key` (String) | 텍스처 경로. 같은 레이어 내 텍스처 변경 최소화 |
| `z` (f32) | `Transform.z`. 같은 레이어·텍스처 내 미세 순서 |

> **주의**: 같은 레이어에서 서로 다른 텍스처를 사용하는 스프라이트 간 z-정렬은 보장되지 않는다. 엄격한 순서가 필요하면 `RenderLayer`로 레이어를 분리하라.

---

## 비헤이비어 트리

`engine::behavior` 모듈은 AI 행동 트리를 ECS 컴포넌트로 제공한다.

### BehaviorNode 트레잇

커스텀 행동을 구현할 때 이 트레잇을 구현한다.

```rust
use engine::behavior::{BehaviorNode, BehaviorStatus};
use engine::ecs::{World, Entity};

struct ChasePlayer {
    speed: f32,
}

impl BehaviorNode for ChasePlayer {
    fn tick(&mut self, world: &mut World, entity: Entity, dt: f32) -> BehaviorStatus {
        // world에서 플레이어 위치, 자신의 Transform 읽기 가능
        BehaviorStatus::Running
    }
}
```

### 내장 복합 노드

| 타입 | 동작 |
|---|---|
| `Sequence` | 자식을 순서대로 실행. 첫 `Failure`에 즉시 중단 |
| `Selector` | 자식을 순서대로 실행. 첫 `Success`에 즉시 중단 |
| `Inverter` | 자식 결과를 반전 (`Success↔Failure`, `Running` 유지) |
| `AlwaysSucceed` | 자식 결과와 무관하게 항상 `Success` 반환 |

### BehaviorTree 컴포넌트

```rust
use engine::behavior::{BehaviorTree, BehaviorNode, BehaviorStatus, Sequence, Selector, Inverter};

// 트리 구성
let tree = BehaviorTree::new(Box::new(Sequence::new(vec![
    Box::new(CheckPlayerInRange { range: 200.0 }),
    Box::new(Selector::new(vec![
        Box::new(AttackPlayer),
        Box::new(Inverter::new(Box::new(FleePlayer))),
    ])),
])));

// 엔티티에 부착
world.add_component(enemy, tree);
```

### BehaviorSystem 등록

```rust
use engine::behavior::BehaviorSystem;

app.add_system(BehaviorSystem);
```

매 프레임 `BehaviorTree`를 가진 모든 엔티티를 자동으로 tick한다. `BehaviorNode::tick`에서 `world`에 자유롭게 접근할 수 있다 (`take_component` 패턴으로 이중 borrow 없이 처리).

### BehaviorStatus

| 값 | 의미 |
|---|---|
| `Running` | 아직 실행 중. 다음 프레임에도 같은 노드 계속 tick |
| `Success` | 성공 완료 |
| `Failure` | 실패 |

---

## Inspector Undo/Redo

에디터 Inspector (네이티브 전용)에서 Ctrl+Z / Ctrl+Shift+Z로 변경을 되돌릴 수 있다.

지원되는 조작:

| 조작 | 단축키 |
|---|---|
| 되돌리기 | Ctrl+Z |
| 다시 실행 | Ctrl+Shift+Z |

자동으로 히스토리에 기록되는 편집:

- 엔티티 생성 (New Entity 버튼)
- 엔티티 삭제 (Delete 버튼)
- Gizmo 드래그로 엔티티 이동

> WASM 빌드에서는 사용 불가 (`#[cfg(not(target_arch = "wasm32"))]`로 컴파일 제외).

---

## 스티어링 행동

`engine::steering` 모듈은 엔티티 이동 AI를 위한 스티어링 행동 컴포넌트를 제공한다.

### 컴포넌트

| 컴포넌트 | 설명 |
|---|---|
| `SteeringVelocity { velocity: Vec2, max_speed: f32 }` | 스티어링 계산 결과 저장. `SteeringSystem`이 이 값을 읽어 `Transform` 이동 |
| `Seek { target: Vec2, max_speed: f32 }` | 목표 위치를 향해 최대 속도로 직선 이동 |
| `Flee { target: Vec2, max_speed: f32, flee_radius: f32 }` | `flee_radius` 이내 접근 시 도망. 반경 밖이면 정지 |
| `Arrive { target: Vec2, max_speed: f32, slow_radius: f32, stop_radius: f32 }` | 목표에 가까워질수록 감속, `stop_radius` 이내 정지 |
| `Wander { max_speed: f32, change_interval: f32 }` | `change_interval`마다 방향 변경하며 배회 |

### SteeringSystem 등록

```rust
app.add_system(SteeringSystem);  // Seek→Flee→Arrive→Wander 순서 처리 후 Transform 이동
```

### 사용 예

```rust
// 적이 플레이어를 향해 이동
world.add_component(enemy, Seek { target: player_pos, max_speed: 120.0 });
world.add_component(enemy, SteeringVelocity::default());

// 근접 시 도망
world.add_component(minion, Flee { target: player_pos, max_speed: 150.0, flee_radius: 80.0 });
world.add_component(minion, SteeringVelocity::default());

// 목표 지점에 부드럽게 정착
world.add_component(unit, Arrive {
    target: destination,
    max_speed: 100.0,
    slow_radius: 60.0,
    stop_radius: 8.0,
});
world.add_component(unit, SteeringVelocity::default());
```

> 여러 스티어링 컴포넌트를 동시에 붙이면 마지막 처리된 것이 적용된다 (Seek→Flee→Arrive→Wander 우선순위).

---

## Blackboard

`Blackboard`는 비헤이비어 트리 노드 간 공유 상태를 저장하는 독립 ECS 컴포넌트다.

### API

| 메서드 | 설명 |
|---|---|
| `Blackboard::new()` | 빈 Blackboard 생성 |
| `set_bool(key, val)` / `get_bool(key) -> Option<bool>` | bool 값 |
| `set_float(key, val)` / `get_float(key) -> Option<f32>` | f32 값 |
| `set_int(key, val)` / `get_int(key) -> Option<i32>` | i32 값 |
| `set_vec2(key, val)` / `get_vec2(key) -> Option<Vec2>` | Vec2 값 |
| `set_string(key, val)` / `get_string(key) -> Option<&str>` | String 값 |

### 사용 예

```rust
// 엔티티에 Blackboard 부착
world.add_component(enemy, Blackboard::new());

// BehaviorNode::tick 내부에서 접근
fn tick(&mut self, world: &mut World, entity: Entity, _dt: f32) -> BehaviorStatus {
    if let Some(bb) = world.get_mut::<Blackboard>(entity) {
        let in_range = bb.get_bool("player_in_range").unwrap_or(false);
        bb.set_float("last_seen_time", 0.0);
    }
    BehaviorStatus::Success
}
```

---

## CommandBuffer

`Commands`는 시스템 실행 중 엔티티/컴포넌트 변경을 안전하게 지연 예약하는 버퍼다.

쿼리 이터레이터가 살아있는 동안 `world.spawn()` 등을 직접 호출하면 borrow 충돌이 발생한다.
`Commands`는 명령을 클로저로 큐에 쌓아두었다가 `world.apply_commands(cmds)` 로 일괄 적용한다.

### API

| 메서드 | 설명 |
|---|---|
| `Commands::new()` | 빈 버퍼 생성 |
| `spawn(f)` | `f(world, entity)`를 apply 시 실행. 클로저 안에서 컴포넌트 추가 |
| `despawn(entity)` | 엔티티 삭제 예약. 이미 없으면 noop |
| `insert::<T>(entity, comp)` | 컴포넌트 추가 예약 |
| `remove::<T>(entity)` | 컴포넌트 제거 예약 |
| `world.apply_commands(cmds)` | 큐 순서대로 일괄 적용 |

### 사용 예

```rust
struct SpawnSystem;
impl System for SpawnSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let mut cmds = Commands::new();

        // 쿼리 루프 중 삭제/생성 예약
        let dead: Vec<Entity> = world.query::<Health>()
            .filter(|(_, h)| h.0 <= 0)
            .map(|(e, _)| e)
            .collect();

        for e in dead {
            cmds.despawn(e);
            cmds.spawn(|world, ne| {
                world.add_component(ne, Particle::default());
            });
        }

        world.apply_commands(cmds);  // 루프 끝난 뒤 일괄 적용
    }
}
```

---

## 에디터 — 씬 그래프 패널

Inspector의 **"Scene"** 탭에서 엔티티 계층 구조를 TreeView로 확인할 수 있다 (네이티브 전용).

- `Parent` 없는 루트 엔티티부터 `Children` 기준으로 들여쓰기 표시
- 항목 클릭 → Inspector 선택 엔티티 변경
- `Tag` 컴포넌트가 있으면 이름 표시, 없으면 `"Entity {id}"`
- Scene 탭 하단 및 Entities 탭 상단에서 이름(Tag) 인라인 편집 가능

> WASM 빌드에서는 사용 불가 (`#[cfg(not(target_arch = "wasm32"))]`로 컴파일 제외).

---

## 에디터 — 컴포넌트 추가/제거

Inspector의 **Entities** 탭에서 선택된 엔티티의 컴포넌트를 추가하거나 제거할 수 있다 (네이티브 전용).

### 컴포넌트 목록 + 제거

선택 엔티티에 붙어있는 컴포넌트가 목록으로 표시되며, 각 항목 오른쪽의 **✕** 버튼으로 즉시 제거한다. `Transform`은 필수 컴포넌트로 보호되어 제거 불가.

### 컴포넌트 추가

드롭다운(ComboBox)에서 등록된 컴포넌트를 선택하고 **+ Add** 버튼을 클릭하면 해당 타입의 기본값 인스턴스가 엔티티에 추가된다.

기본 등록 컴포넌트: `Sprite`, `RenderLayer`, `ParticleEmitter`, `Blackboard`, `Timer`

### 커스텀 컴포넌트 등록

`App::register_component`로 사용자 정의 컴포넌트를 Inspector에 등록할 수 있다.

```rust
// App 설정 시 등록
app.register_component("Enemy", |world, entity| {
    world.add_component(entity, Enemy { hp: 100, speed: 80.0 });
});

app.register_component("Coin", |world, entity| {
    world.add_component(entity, Coin { value: 10 });
});
```

등록 후 Inspector Add Component 드롭다운에 "Enemy", "Coin"이 표시된다.

### World::register_reflect_named

```rust
// Reflect 트레잇을 구현한 타입을 이름과 함께 등록
world.register_reflect_named::<MyComp>("MyComp");

// 기존 register_reflect와 동일하지만 Inspector 목록 표시 이름 지정 가능
```

> WASM 빌드에서는 사용 불가 (`#[cfg(not(target_arch = "wasm32"))]`로 컴파일 제외).

---

## Rhai 스크립팅

`engine::scripting` 모듈은 Rhai 스크립트 파일을 에셋으로 로드하고 엔티티에 부착해 실행한다.

### 기본 사용

```rust
use engine::{ScriptAsset, ScriptRunner, ScriptingSystem};

// 스크립트 로드
let handle = assets.load_script("assets/scripts/enemy_ai.rhai");

// 엔티티에 ScriptRunner 부착
world.add_component(enemy, ScriptRunner::new(handle));

// 시스템 등록
app.add_system(ScriptingSystem);
```

### 스크립트에서 사용 가능한 함수 (확장 API)

#### Commands

| 함수 | 설명 |
|---|---|
| `spawn_entity() -> i64` | 새 엔티티 생성. 임시 핸들 반환 (음수) |
| `despawn_entity(id)` | 엔티티 삭제 예약 |

#### Blackboard

| 함수 | 설명 |
|---|---|
| `bb_get_bool(key)` / `bb_set_bool(key, val)` | bool 읽기/쓰기 |
| `bb_get_float(key)` / `bb_set_float(key, val)` | f32 읽기/쓰기 |
| `bb_get_int(key)` / `bb_set_int(key, val)` | i32 읽기/쓰기 |

#### Steering

| 함수 | 설명 |
|---|---|
| `seek_target(tx, ty, speed)` | Seek 컴포넌트 설정 |
| `flee_from(tx, ty, speed, radius)` | Flee 컴포넌트 설정 |
| `stop_steering()` | 스티어링 속도 0으로 초기화 |

### 스크립트 예시

```rhai
// enemy_ai.rhai
let in_range = bb_get_bool("player_in_range");
if in_range {
    seek_target(player_x, player_y, 120.0);
} else {
    stop_steering();
    bb_set_float("idle_time", bb_get_float("idle_time") + dt);
}
```

---

## 2D 라이팅

씬 전체에 포인트 라이트 효과를 적용하는 후처리 패스. `AmbientLight` 리소스를 등록하면 자동 활성화된다.

### AmbientLight 리소스

```rust
// 등록만 하면 LightingRenderer가 자동 활성화됨
world.insert_resource(AmbientLight {
    color: [1.0, 0.9, 0.8],   // 따뜻한 환경광
    intensity: 0.05,           // 5% 기본 밝기 (어두운 씬)
});

// 기본값: 흰색 환경광 10%
world.insert_resource(AmbientLight::default());
```

| 필드 | 타입 | 설명 |
|------|------|------|
| `color` | `[f32; 3]` | RGB 환경광 색상 (0.0~1.0) |
| `intensity` | `f32` | 환경광 강도. 0.0 = 완전 어두움, 1.0 = 원본 밝기 |

### PointLight 컴포넌트

`Transform`과 함께 엔티티에 추가한다. 최대 16개까지 동시에 적용된다.

```rust
let light_entity = world.spawn();
world.add_component(light_entity, Transform {
    position: Vec2::new(200.0, 150.0),
    ..Default::default()
});
world.add_component(light_entity, PointLight {
    color: [1.0, 0.8, 0.4],   // 주황빛
    radius: 300.0,             // 월드 좌표 픽셀 반경
    intensity: 2.0,            // 밝기 배율
});
```

| 필드 | 타입 | 기본값 | 설명 |
|------|------|--------|------|
| `color` | `[f32; 3]` | `[1.0, 1.0, 1.0]` | RGB 라이트 색상 |
| `radius` | `f32` | `200.0` | 월드 좌표 픽셀 단위 반경 |
| `intensity` | `f32` | `1.0` | 밝기 배율 (1.0이 기본) |

### PostProcess와 연동 순서

라이팅과 후처리가 모두 활성화된 경우:

```
스프라이트 렌더링
    → PostProcessRenderer.target_view (중간 텍스처)
    → LightingRenderer.run_pass (라이팅 적용)
    → 스왑체인 (화면 출력)
```

라이팅만 활성화된 경우:

```
스프라이트 렌더링
    → 중간 씬 텍스처 (scene_texture_for_lighting)
    → LightingRenderer.run_pass (라이팅 적용)
    → 스왑체인 (화면 출력)
```

> **플랫폼**: 라이팅은 네이티브(native) 전용. WASM 빌드에서는 비활성화된다.

### 완전한 사용 예

```rust
struct SetupSystem;
impl System for SetupSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        // 환경광 등록 (라이팅 활성화)
        world.insert_resource(AmbientLight { color: [1.0, 1.0, 1.0], intensity: 0.05 });

        // 플레이어 주변 따뜻한 빛
        let torch = world.spawn();
        world.add_component(torch, Transform {
            position: Vec2::new(0.0, 0.0),
            ..Default::default()
        });
        world.add_component(torch, PointLight {
            color: [1.0, 0.7, 0.3],
            radius: 250.0,
            intensity: 1.5,
        });
    }
}
```

---

## ECS 변경 감지

이번 프레임에 추가/교체된 컴포넌트만 조회할 수 있다. dirty-flag 패턴을 대체한다.

### API

```rust
// 이번 프레임에 처음 추가된 컴포넌트를 가진 엔티티
for (entity, health) in world.query_added::<Health>() {
    println!("새 엔티티 {:?}: HP={}", entity, health.0);
}

// 이번 프레임에 교체된 컴포넌트를 가진 엔티티
for (entity, transform) in world.query_changed::<Transform>() {
    update_spatial_cache(entity, transform.position);
}

// 명시적 초기화 (App이 매 프레임 자동 호출 — 직접 호출 불필요)
world.clear_change_tracking();
```

### 동작 규칙

| 상황 | `query_added` | `query_changed` |
|------|---------------|-----------------|
| 엔티티에 컴포넌트 첫 추가 | ✓ 포함 | ✗ 없음 |
| 기존 컴포넌트 교체 (`add_component` 재호출) | ✗ 없음 | ✓ 포함 |
| `despawn` 호출 | 자동 제거 | 자동 제거 |
| `remove_component` 호출 | 자동 제거 | 자동 제거 |
| 다음 프레임 (`clear_change_tracking`) | 초기화 | 초기화 |

### 활용 예 — 온보딩 처리

```rust
struct SpawnSetupSystem;
impl System for SpawnSetupSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        // 이번 프레임에 새로 스폰된 Enemy만 초기화
        let new_enemies: Vec<_> = world
            .query_added::<Enemy>()
            .map(|(e, _)| e)
            .collect();
        for entity in new_enemies {
            world.add_component(entity, Blackboard::default());
            world.add_component(entity, SteeringVelocity::default());
        }
    }
}
```

### 활용 예 — Transform 변경 추적

```rust
struct TransformDirtySystem;
impl System for TransformDirtySystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        // 위치가 바뀐 오브젝트만 공간 캐시 업데이트
        let changed: Vec<_> = world
            .query_changed::<Transform>()
            .map(|(e, t)| (e, t.position))
            .collect();
        if let Some(grid) = world.resource_mut::<SpatialGrid>() {
            for (entity, pos) in changed {
                grid.update(entity, pos);
            }
        }
    }
}
```

---

## 2D 노멀 맵 라이팅

Phase 41a의 `PointLight` + `AmbientLight` 라이팅 시스템을 확장해, 스프라이트별 노멀 맵 텍스처로 방향성 조명(Lambert diffuse)을 구현한다.

### 노멀 맵이 없을 때 기본 동작

라이팅 패스는 매 프레임 노멀 버퍼를 평면 노멀 `[0.5, 0.5, 1.0]` (카메라 방향)로 초기화한다. 노멀 맵 없이도 `PointLight`의 `light_height` 값에 따라 방향성 조명 효과를 얻을 수 있다.

### Sprite.normal_texture

```rust
// 노멀 맵이 있는 스프라이트
let mut sprite = Sprite::textured("assets/rock.png");
sprite.normal_texture = Some("assets/rock_normal.png".to_string());
world.add_component(entity, sprite);
```

| 필드 | 타입 | 설명 |
|------|------|------|
| `normal_texture` | `Option<String>` | 노멀 맵 파일 경로 (None이면 평면 노멀) |
| `normal_handle` | `Option<Handle<ImageAsset>>` | 런타임 핸들 (직렬화 제외) |

### PointLight.light_height

광원의 가상 Z 높이. 노멀 맵과 함께 Lambert 조명의 방향 벡터 L의 Z 성분으로 사용된다.

```rust
world.add_component(light_entity, PointLight {
    color: [1.0, 0.9, 0.7],
    radius: 300.0,
    intensity: 2.0,
    light_height: 0.15,  // 작을수록 측광(그라데이션), 클수록 정면광
});
```

| `light_height` | 효과 |
|---|---|
| `0.05` | 강한 측면 조명 — 노멀 맵 요철이 뚜렷하게 표현 |
| `0.15` | 기본값 — 자연스러운 방향성 조명 |
| `0.5~1.0` | 거의 정면광 — 노멀 맵 효과 약해짐 |

### 노멀 맵 파일 형식

- OpenGL 기준 노멀 맵: RGB채널에 `(R=X, G=Y, B=Z)` 저장, 평면은 `(128, 128, 255)`
- 일반 게임 툴(Substance, NormalMap-Online 등) 기본 출력 포맷과 호환

### 렌더 파이프라인 순서

```
스프라이트 렌더링 (scene_tex)
    ↓
노멀 버퍼 초기화 (flat normal [0.5, 0.5, 1.0])
    (추후: 스프라이트 노멀 텍스처 → 노멀 버퍼 렌더)
    ↓
LightingRenderer (scene_tex + normal_buf → Lambert diffuse)
    ↓
화면 출력
```

### 사용 예 — 돌 텍스처에 조명 효과

```rust
struct SetupSystem;
impl System for SetupSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        world.insert_resource(AmbientLight { color: [1.0, 1.0, 1.0], intensity: 0.04 });

        let rock = world.spawn();
        let mut sprite = Sprite::textured("assets/rock.png");
        sprite.normal_texture = Some("assets/rock_normal.png".to_string());
        world.add_component(rock, Transform {
            position: Vec2::new(0.0, 0.0),
            scale: Vec2::splat(128.0),
            ..Default::default()
        });
        world.add_component(rock, sprite);

        // 측면에서 비추는 따뜻한 빛
        let torch = world.spawn();
        world.add_component(torch, Transform {
            position: Vec2::new(-200.0, -100.0),
            ..Default::default()
        });
        world.add_component(torch, PointLight {
            color: [1.0, 0.7, 0.3],
            radius: 400.0,
            intensity: 3.0,
            light_height: 0.08,  // 낮게 — 노멀 맵 요철 강조
        });
    }
}
```

---

## 카메라 이펙트

`Camera` 리소스에 세 가지 이펙트가 내장된다. `App`이 매 프레임 자동으로 `camera.update(dt, follow_pos)`를 호출하므로 별도 시스템 등록 불필요.

### Camera Shake

```rust
// 시스템 내부에서
if let Some(cam) = world.resource_mut::<Camera>() {
    cam.shake(15.0, 0.3); // strength=15px, duration=0.3초
}
```

| 파라미터 | 타입 | 설명 |
|---|---|---|
| `strength` | `f32` | 최대 진폭 (픽셀) |
| `duration` | `f32` | 지속 시간 (초) |

shake offset은 `view_proj()` 내부에서 자동 적용된다. sin/cos 두 주파수 합성으로 자연스러운 흔들림 연출.

### Smooth Follow

```rust
let cam = world.resource_mut::<Camera>().unwrap();
cam.follow_entity = Some(player_entity);
cam.lerp_factor = 6.0; // 초당 lerp 강도. 클수록 빠르게 추적
```

`App::update()`에서 `follow_entity`의 `Transform.position`을 읽어 `lerp_factor * dt`로 보간한다. 엔티티가 삭제되면 마지막 위치에 정지.

| 필드 | 기본값 | 설명 |
|---|---|---|
| `follow_entity` | `None` | 추적할 엔티티 |
| `lerp_factor` | `5.0` | 추적 속도. 1.0 이상이면 dt에 따라 snap에 가까워짐 |

### Zoom Tween

```rust
// 2배로 0.5초 동안 줌인
cam.zoom_to(2.0, 3.0); // target=2.0, speed=3.0 units/sec
```

`zoom`이 `zoom_target`에 도달하면 트윈이 자동 종료된다.

| 파라미터 | 설명 |
|---|---|
| `target_zoom` | 목표 줌 배율 |
| `speed` | 초당 zoom 변화량 (양수) |

### 사용 예 — 플레이어 추적 + 충격 흔들기

```rust
// 초기화
let cam = world.resource_mut::<Camera>().unwrap();
cam.follow_entity = Some(player);
cam.lerp_factor = 5.0;

// 피격 이벤트 처리
for ev in world.resource::<Events<HitEvent>>().unwrap().read() {
    if let Some(cam) = world.resource_mut::<Camera>() {
        cam.shake(20.0, 0.25);
    }
}

// 보스 등장 시 줌인
if boss_spawned {
    if let Some(cam) = world.resource_mut::<Camera>() {
        cam.zoom_to(1.5, 2.0);
    }
}
```

---

## 오브젝트 풀

총알·파티클처럼 자주 생성/소멸되는 엔티티를 재활용한다. 반납된 엔티티는 `Pooled` 마커 컴포넌트로 표시되며, `query_without::<Pooled>()` 로 시스템에서 제외할 수 있다.

### Pool API

```rust
// 리소스로 등록 (최대 64개 재사용)
world.insert_resource(Pool::new(64));

// 획득 — 풀에 있으면 재사용, 없으면 새 스폰
let pool = world.resource_mut::<Pool>().unwrap();
let bullet = pool.acquire(&mut world, |w, e| {
    w.add_component(e, Transform { position: spawn_pos, ..Default::default() });
    w.add_component(e, Bullet { speed: 400.0 });
    w.add_component(e, Sprite::colored(1.0, 1.0, 0.0));
});

// 반납 — 풀에 저장 (capacity 초과 시 despawn)
pool.release(bullet, &mut world);
```

| 메서드 | 설명 |
|---|---|
| `Pool::new(capacity)` | 최대 capacity개 엔티티를 저장하는 풀 생성 |
| `acquire(world, setup)` | 풀에서 엔티티 가져오기. 없으면 spawn. setup 클로저로 초기화 |
| `release(entity, world)` | 엔티티 반납. `Pooled` 마커 추가. 풀 초과 시 despawn |
| `available_count()` | 현재 대기 중인 엔티티 수 |
| `capacity()` | 최대 풀 용량 |
| `clear(world)` | 풀 전체 비우기 + despawn |

### Pooled 마커

반납된 엔티티에 자동 추가되는 마커 컴포넌트.

```rust
// 비활성 엔티티를 렌더/시스템에서 제외
for (e, bullet) in world.query_without::<Bullet, Pooled>() {
    // 활성화된 총알만 처리
}
```

### 사용 예 — 총알 풀

```rust
struct BulletSystem;
impl System for BulletSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // 발사
        if input.just_pressed(KeyCode::Space) {
            if let Some(mut pool) = world.take_resource::<Pool>() {
                let pos = player_transform.position;
                pool.acquire(world, move |w, e| {
                    w.add_component(e, Transform { position: pos, ..Default::default() });
                    w.add_component(e, Bullet { velocity: Vec2::new(0.0, -400.0) });
                });
                world.insert_resource(pool);
            }
        }

        // 화면 밖 총알 회수
        let out_of_bounds: Vec<_> = world
            .query_without::<Bullet, Pooled>()
            .filter(|(_, b)| b.position.y < -100.0)
            .map(|(e, _)| e)
            .collect();
        if !out_of_bounds.is_empty() {
            if let Some(mut pool) = world.take_resource::<Pool>() {
                for e in out_of_bounds {
                    pool.release(e, world);
                }
                world.insert_resource(pool);
            }
        }
    }
}
```

---

## 엔티티 복제

`World::clone_entity(src)` — 등록된 컴포넌트를 복사해 새 엔티티를 생성한다.

### 기본 사용

```rust
let new_entity = world.clone_entity(original_entity);

// 위치를 약간 오프셋
if let Some(t) = world.get_mut::<Transform>(new_entity) {
    t.position += Vec2::new(16.0, 16.0);
}
```

### 복제 가능 타입 등록

기본 등록 타입: `Transform`, `Sprite`, `RenderLayer`, `Tag`, `AnimationPlayer`, `Timer`

커스텀 컴포넌트도 등록 가능:

```rust
// App 초기화 시
app.world.register_clone::<MyComponent>();
```

`register_clone<T>()` 는 `T: Clone + Send + Sync + 'static` 을 요구한다. 등록되지 않은 타입은 복제 시 무시된다.

### Inspector Duplicate 버튼

에디터 Inspector 패널에 "Duplicate" 버튼이 추가되었다. 선택된 엔티티를 복제하고 (16, 16) 오프셋 적용 후 새 엔티티를 선택한다.

---

## Debug Draw API

시스템에서 직접 호출해 화면에 디버그 도형을 그린다. App이 렌더링 후 자동으로 초기화하므로 매 프레임 새로 그리면 된다.

### 기본 사용

```rust
impl System for MySystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        if let Some(dbg) = world.resource_mut::<DebugDraw>() {
            // AABB 박스
            dbg.rect(Vec2::new(0., 0.), Vec2::new(64., 64.), [1., 0., 0., 1.]);
            // 반경 표시
            dbg.circle(player_pos, attack_radius, [0., 1., 0., 0.5]);
            // 경로 시각화
            dbg.line(from, to, [1., 1., 0., 1.]);
            // 포인트 마커
            dbg.cross(waypoint, 8., [0., 0., 1., 1.]);
        }
    }
}
```

### API

| 메서드 | 설명 |
|---|---|
| `rect(min, max, color)` | 축 정렬 사각형 외곽선 |
| `line(start, end, color)` | 직선 (두께 1.5px) |
| `line_thick(start, end, color, thickness)` | 두께 지정 직선 |
| `circle(center, radius, color)` | 원 (24각형 근사) |
| `cross(pos, size, color)` | 십자 마커 |
| `clear()` | 도형 초기화 (App이 자동 호출) |

모든 도형은 z=999로 렌더링되어 게임 오브젝트 위에 표시된다. 색상은 `[r, g, b, a]` (0.0~1.0).

---

## 씬 전환 트랜지션

`FadeTransition` 리소스를 등록하면 App이 전체 화면 컬러 오버레이를 자동으로 애니메이션한다.

### 페이드 아웃 (화면이 어두워짐)

```rust
// 0.5초 동안 검정으로 페이드 아웃
world.insert_resource(FadeTransition::fade_out(0.5));

// 완료 여부 확인
if let Some(fade) = world.resource::<FadeTransition>() {
    if fade.finished {
        // 씬 전환 실행
        world.resource_mut::<SceneChange>().unwrap().0 =
            Some(SceneCmd::Replace(Box::new(NextScene)));
    }
}
```

### 페이드 인 (화면이 밝아짐)

```rust
// 씬 시작 시 0.3초 페이드 인
world.insert_resource(FadeTransition::fade_in(0.3));
```

### 커스텀 색상

```rust
world.insert_resource(
    FadeTransition::fade_out(0.8).with_color(1.0, 1.0, 1.0) // 흰색 페이드
);
```

### API

| 메서드 | 설명 |
|---|---|
| `FadeTransition::fade_out(duration)` | 투명 → 불투명 (검정) |
| `FadeTransition::fade_in(duration)` | 불투명 → 투명 |
| `.with_color(r, g, b)` | 오버레이 색상 변경 (기본: 검정) |
| `fade.finished` | 페이드 완료 여부 |
| `fade.alpha` | 현재 알파값 (0.0~1.0) |

> **플랫폼**: 네이티브 전용. WASM 빌드에서는 비활성화.

### 씬 전환 패턴 (페이드 아웃 → 전환 → 페이드 인)

```rust
struct GameScene { fade_out_done: bool }

impl Scene for GameScene {
    fn on_enter(&mut self, world: &mut World, app: &mut App) {
        // 씬 진입 시 페이드 인
        world.insert_resource(FadeTransition::fade_in(0.4));
        self.fade_out_done = false;
    }
}

struct GameSystem;
impl System for GameSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        // 특정 조건에서 페이드 아웃 → 씬 전환
        if player_dead && !fade_started {
            world.insert_resource(FadeTransition::fade_out(0.6));
        }
        if let Some(fade) = world.resource::<FadeTransition>() {
            if fade.finished && fade.alpha > 0.5 {
                // 페이드 아웃 완료 → 씬 전환
                world.resource_mut::<SceneChange>().unwrap().0 =
                    Some(SceneCmd::Replace(Box::new(GameOverScene)));
            }
        }
    }
}
```

---

## 타임라인/컷씬

`Timeline` ECS 컴포넌트로 엔티티의 위치·회전·색상을 키프레임 기반으로 애니메이션한다.

### 기본 사용

```rust
use engine::{Timeline, TimelineSystem, Easing};

// 2초 타임라인 생성
let mut tl = Timeline::new(2.0);

// 위치 트랙 추가
tl.position.add(0.0, Vec2::new(0., 0.), Easing::Linear);
tl.position.add(1.0, Vec2::new(300., 0.), Easing::EaseOut);
tl.position.add(2.0, Vec2::new(300., 200.), Easing::EaseInOut);

// 색상 페이드
tl.color.add(0.0, [1., 0., 0., 1.], Easing::Linear);  // 빨강
tl.color.add(2.0, [0., 0., 1., 1.], Easing::Linear);  // 파랑

world.add_component(entity, tl);
app.add_system(TimelineSystem);
```

### 트랙 종류

| 트랙 | 타입 | 적용 대상 |
|------|------|-----------|
| `position` | `Track<Vec2>` | `Transform.position` |
| `rotation` | `Track<f32>` | `Transform.rotation` (라디안) |
| `scale` | `Track<Vec2>` | `Transform.scale` |
| `color` | `Track<[f32;4]>` | `Sprite.color` |
| `alpha` | `Track<f32>` | `Sprite.color[3]` |

### Timeline 제어

```rust
let tl = world.get_mut::<Timeline>(entity).unwrap();
tl.pause();    // 일시 정지
tl.play();     // 재생 재개
tl.restart();  // 처음부터 재생
tl.looping = true; // 반복 재생

// 완료 확인
if tl.is_finished() { ... }
```

### 반복 재생

```rust
let mut tl = Timeline::new(1.5).looping(); // builder 패턴
tl.position.add(0.0, Vec2::new(0., 0.), Easing::EaseIn);
tl.position.add(1.5, Vec2::new(0., 100.), Easing::EaseOut);
```

### Easing 종류

`Easing::Linear`, `EaseIn`, `EaseOut`, `EaseInOut`, `EaseInCubic`, `EaseOutBounce` 등 — `src/tween.rs` 참조.

### 카메라 컷씬 예

```rust
// 카메라 엔티티에 타임라인 추가
let cam_entity = world.spawn();
world.add_component(cam_entity, Transform::default());

let mut tl = Timeline::new(4.0);
tl.position.add(0.0, Vec2::new(0., 0.), Easing::EaseInOut);
tl.position.add(2.0, Vec2::new(500., 300.), Easing::EaseInOut);
tl.position.add(4.0, Vec2::new(0., 0.), Easing::EaseInOut);
world.add_component(cam_entity, tl);

// Camera 리소스를 cam_entity Transform과 동기화하는 별도 시스템 추가
```

---

## 좌표 규약

```
(0, 0) ──────────────────→ X+
  │
  │   (스크린/월드 모두 Y↓ 기준)
  │
  ↓ Y+
```

- **월드 좌표**: Transform.position, 물리 위치 등. 카메라에 의해 이동된 뷰에서 렌더링.
- **스크린 좌표**: InputState.cursor(), TextQueue, UiQueue. 항상 좌상단 원점 고정.
- **카메라 position**: 뷰포트 **좌상단**이 가리키는 월드 좌표.
- **Transform.z / UiNode.z**: 값이 클수록 **앞에** 그려진다. 타일맵 기본값 `-1.0`.
- **물리 중력**: `Vec2::new(0.0, 9.8)` → Y 아래 방향이 +.
