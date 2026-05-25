# rust-2d-engine 레퍼런스

> 버전 v0.32.0 기준. wgpu 기반 2D 게임 엔진.

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
21. [좌표 규약](#좌표-규약)

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
