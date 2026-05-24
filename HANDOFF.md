# 핸드오프 문서 — rust-2d-engine

작성일: 2026-05-24 (Phase 16 갱신: 2026-05-24)  
엔진 버전: v0.16.0 (태그: v0.3.0, main 브랜치 기준)  
작성자: ChunSam

---

## 프로젝트 개요

wgpu 기반 Rust 2D 게임 엔진. ECS 아키텍처 위에 물리(Rapier2D), 오디오, 파티클, 타일맵, UI, 씬 시스템 등을 갖추고 있다. 별도의 게임 프로젝트(`rust-survivors`)가 이 엔진을 의존성으로 사용한다.

- **저장소**: `https://github.com/ChunSam/rust-2d-engine`
- **브랜치**: `main`
- **엔진 소스 규모**: 약 5,400 LOC (src/ 전체)

---

## 완료된 작업 — Phase별

| Phase | 주요 내용 | 커밋 |
|---|---|---|
| Phase 1 | 오디오 pan, 파티클 시스템, 충돌 디버그 시각화, 타일맵, 입력 리바인딩 | `93d54c4` |
| Phase 2 | ECS `query4` 추가 | `93d54c4` |
| Phase 3 | `PhysicsWorld` 캡슐화 (접근자 메서드, pub(crate) 내부화) | `fa9013c` |
| Phase 4 | `query_opt2`, `Events<E>` 이벤트 시스템, UI Widget System | `767a1d2` |
| Phase 5 | 씬 시스템 (Scene/SceneCmd/SceneChange), Timer, Tween (Easing 6종) | `2147291` |
| Phase 6 | UI 시스템 강화 — TextInput, ScrollView, Panel+LayoutSystem | `e98b893` |
| Phase 7 | CollisionEvent — Rapier NarrowPhase 폴링 → `Events<CollisionEvent>` 브리징 | `b4a931d` |
| Phase 8 | Save/Load 완성 — `load_or_default`, `exists`, `delete`, lib.rs re-export | `01f983b` |
| Phase 9 | ECS Archetype 스토리지 — TypeId HashMap+Vec → Archetype 밀집 컬럼 스토리지 | `a8b49cc` |
| Phase 10 | 포스트프로세싱 — 비네팅, 색수차, 근사 블룸 (PostProcessConfig 리소스) | `a8b49cc` |
| Phase 11 | 오디오 강화 — 위치 오디오, 버스 믹서, 페이드인/아웃 | `a8b49cc` |
| Phase 12 | Transform 계층 — Parent/Children/GlobalTransform, HierarchySystem, attach/detach | `3862f8d` |
| Phase 13 | 물리 레이캐스트 + 캐릭터 컨트롤러 — RaycastHit, add_kinematic_*, move_character | `eee451d` |
| Phase 14 | 애니메이션 상태 머신 — AnimationStateMachine, StateMachineSystem, TransitionCond, AnimParam | `93eb65f` |
| Phase 15 | 게임패드(gilrs) + UI Slider/CheckBox — GamepadState, Slider, CheckBox, UiEvent 확장 | `30d1b9e` |

---

## 현재 구조

```
src/
├── app.rs            엔진 진입점 (winit ApplicationHandler)
├── ecs/
│   ├── world.rs      Entity/Component/Resource 저장소, query1~4, query_opt2
│   ├── events.rs     Events<E> 프레임 경계 이벤트 버스
│   └── system.rs     System 트레잇
├── hierarchy.rs      Parent, Children, GlobalTransform, HierarchySystem, attach/detach  ← Phase 12
├── scene.rs          Scene 트레잇, SceneCmd, SceneChange
├── prefab.rs         Tag, EntityDef, SceneDef, Prefab, spawn_entity_def, spawn_scene_def  ← Phase 16
├── components.rs     Transform, Sprite (Serialize/Deserialize 추가 ← Phase 16)
├── resources.rs      WindowConfig, ViewportSize, GameState, ShouldQuit, ...
├── camera.rs         Camera (position, zoom, screen_to_world)
├── input/
│   ├── state.rs      InputState (키보드, 마우스, 스크롤, 문자 입력 버퍼)
│   ├── gamepad.rs    GamepadState, GamepadButton, GamepadAxis (gilrs 래퍼)  ← Phase 15
│   └── map.rs        InputMap (키 리바인딩)
├── physics/
│   ├── world.rs      PhysicsWorld (Rapier2D 래퍼) + RaycastHit + 레이캐스트/캐릭터 메서드 ← Phase 13
│   ├── body.rs       PhysicsBody 컴포넌트
│   ├── character.rs  CharacterController (KinematicCharacterController 래퍼)           ← Phase 13
│   ├── events.rs     CollisionEvent (Started/Stopped)              ← Phase 7
│   └── system.rs     PhysicsSystem
├── collision/
│   ├── grid.rs       SpatialGrid, CollisionGridSystem
│   ├── query.rs      Collider, CollisionLayer
│   └── debug.rs      CollisionDebugSystem, DebugConfig
├── audio.rs          AudioManager (재생/정지/볼륨/팬/톤)
├── animation/
│   ├── player.rs       AnimationPlayer, AnimationClip, UvRect
│   ├── state_machine.rs AnimationStateMachine, StateMachineSystem, TransitionCond, AnimParam  ← Phase 14
│   └── system.rs       AnimationSystem
├── particle.rs       ParticleEmitter, Particle, ParticleSystem
├── tilemap.rs        Tilemap, TilemapAtlas, TilemapSystem
├── timer.rs          Timer (once/repeating)
├── tween.rs          Tween, Easing
├── ui/
│   ├── node.rs       UiNode, Anchor
│   ├── button.rs     Button, ButtonState
│   ├── label.rs      Label
│   ├── text_input.rs TextInput (커서, 깜빡임, UTF-8 안전 편집)  ← Phase 6
│   ├── scroll_view.rs ScrollView (내부 Vec 기반 스크롤 목록)    ← Phase 6
│   ├── panel.rs      Panel, LayoutDir, LayoutSystem             ← Phase 6
│   ├── slider.rs     Slider (수평 슬라이더)                     ← Phase 15
│   ├── checkbox.rs   CheckBox (토글 체크박스)                   ← Phase 15
│   └── system.rs     UiSystem, UiEvent (7종)
├── renderer/
│   ├── context.rs    GpuContext (wgpu Surface/Device/Queue 래퍼)
│   ├── post_process.rs PostProcessRenderer, PostProcessConfig     ← Phase 10
│   ├── sprite.rs     SpriteRenderer (인스턴스드 렌더링)
│   ├── text.rs       TextRenderer, TextQueue, DrawText
│   ├── texture.rs    Texture
│   ├── ui.rs         UiQueue, DrawRect
│   └── shaders/
│       ├── sprite.wgsl
│       └── post_process.wgsl                                      ← Phase 10
└── save.rs           RON 세이브/불러오기 (save/load/load_or_default/exists/delete)
```

---

## 이번 세션에서 한 일 (Phase 16)

### Phase 16 — 씬 직렬화 + 프리팹 시스템

**배경**: RON 파일 한 장으로 레벨 전체를 저장·로드하고, 단일 엔티티 템플릿(프리팹)을 재사용할 수 있는 기반을 마련한다.

**추가된 파일**: `src/prefab.rs`  
**변경된 파일**: `Cargo.toml`, `src/components.rs`, `src/lib.rs`

#### 주요 타입

| 타입 | 역할 |
|------|------|
| `Tag` | 엔티티 식별용 문자열 컴포넌트 (Serialize/Deserialize 지원) |
| `EntityDef` | 엔티티 1개를 기술하는 직렬화 가능 구조체 (tag, transform, sprite 선택 필드) |
| `SceneDef` | `Vec<EntityDef>` 래퍼 — RON 파일 한 장 = 레벨 하나 |
| `Prefab` | `EntityDef`를 파일로 저장·로드·스폰하는 단일 템플릿 |

#### 공개 함수

```rust
spawn_entity_def(world, &EntityDef) -> Entity
spawn_scene_def(world, &SceneDef)   -> Vec<Entity>
SceneDef::save(&self, path)         -> Result<(), SaveError>
SceneDef::load(path)                -> Result<SceneDef, SaveError>
Prefab::save(&self, path)           -> Result<(), SaveError>
Prefab::load(path)                  -> Result<Prefab, SaveError>
Prefab::spawn(&self, world)         -> Entity
```

#### 씬 파일 형식 (RON 예시)

```ron
SceneDef(
    entities: [
        EntityDef(
            tag: Some("ground"),
            transform: Some(Transform(
                position: (0.0, -200.0),
                scale: (800.0, 32.0),
                rotation: 0.0,
                z: 0.0,
            )),
            sprite: Some(Sprite(
                texture: None,
                color: (0.3, 0.6, 0.3, 1.0),
            )),
        ),
    ],
)
```

#### Cargo.toml 변경

- `glam = { version = "0.28", features = ["serde"] }` — Vec2 serde 지원 추가

#### 설계 결정

- **정적 타입 EntityDef**: Transform + Sprite만 지원. 동적 컴포넌트 레지스트리는 Phase 17 이후 고려.  
- **save.rs 재사용**: 씬/프리팹 직렬화는 기존 `save()` / `load()` 인프라 위에 구현.  
- **Tag 컴포넌트 분리**: 씬 로드 후 "player", "enemy" 등 역할을 쿼리로 구분하기 위한 전용 컴포넌트.

---

## 이전 세션 (Phase 15)

### Phase 15 — 게임패드 + UI Slider/CheckBox

**배경**: 키보드/마우스만 지원하던 입력 레이어를 완성하고, 슬라이더·체크박스 UI 위젯을 추가해 설정 화면 구성 능력을 갖추는 것이 목표.

**추가된 파일**: `src/input/gamepad.rs`, `src/ui/slider.rs`, `src/ui/checkbox.rs`  
**변경된 파일**: `Cargo.toml`, `src/input/mod.rs`, `src/app.rs`, `src/ui/mod.rs`, `src/ui/system.rs`, `src/lib.rs`

#### 게임패드 입력 (gilrs 0.10)

| 타입 | 역할 |
|------|------|
| `GamepadState` | ECS 리소스. 최대 4개 패드 슬롯, 버튼/축 상태 추적 |
| `GamepadButton` | South/East/North/West/LeftBumper/RightBumper/… 16종 |
| `GamepadAxis` | LeftStickX/Y, RightStickX/Y, LeftTrigger, RightTrigger, DPadX/Y |

```rust
// 슬롯 0 (첫 번째 연결 패드)
if let Some(gs) = world.resource::<GamepadState>() {
    if gs.just_pressed(0, GamepadButton::South) { /* 점프 */ }
    let lx = gs.axis(0, GamepadAxis::LeftStickX);
}
```

- `App::new()` 에서 `GamepadState::default()` 자동 삽입
- gilrs 이벤트는 `about_to_wait` 에서 폴링 → `update()` 마지막에 `flush()`
- `Connected` / `Disconnected` gilrs 이벤트로 슬롯 동적 할당/해제

#### UI Slider

```rust
let e = world.spawn();
world.insert(e, UiNode::new(100.0, 300.0, 200.0, 20.0));
world.insert(e, Slider::new(0.0, 100.0, 50.0));
// UiEvent::SliderChanged(entity, new_value) 로 변경 통보
```

- 트랙 클릭 또는 썸 드래그로 값 변경
- 색상 커스터마이즈: `track_color`, `fill_color`, `thumb_color`, `thumb_hovered_color`

#### UI CheckBox

```rust
let e = world.spawn();
world.insert(e, UiNode::new(50.0, 200.0, 160.0, 24.0));
world.insert(e, CheckBox::new("사운드 켜기"));
// UiEvent::CheckBoxToggled(entity, checked) 로 토글 통보
```

#### UiEvent 확장

`SliderChanged(Entity, f32)`, `CheckBoxToggled(Entity, bool)` 추가 (기존 5종 → 7종).

---

## 이번 세션에서 한 일 (Phase 14)

### Phase 14 — 애니메이션 상태 머신

**배경**: `AnimationPlayer.play(clip_index)` 로만 클립을 전환하면 게임 로직이 직접 애니메이션 인덱스를 관리해야 했다. 캐릭터 상태(idle/run/jump/attack)가 많아질수록 조건 분기가 급증하므로, 상태 머신으로 전환 규칙을 선언적으로 분리할 필요가 있었다.

**추가된 파일**: `src/animation/state_machine.rs`  
**변경된 파일**: `src/animation/mod.rs`, `src/animation/player.rs`, `src/lib.rs`

#### 신규 타입

| 타입 | 역할 |
|------|------|
| `AnimationStateMachine` | 엔티티에 붙이는 상태 머신 컴포넌트 |
| `AnimState` | 클립 인덱스 + 전환 엣지 목록 |
| `AnimTransition` | 대상 상태 + AND 조건 목록 |
| `TransitionCond` | `BoolEq` / `FloatGt` / `FloatLt` / `Trigger` / `AnimationEnd` |
| `AnimParam` | `Bool(bool)` / `Float(f32)` / `Trigger(bool)` |
| `StateMachineSystem` | 매 프레임 전환 평가 → `AnimationPlayer.play()` 호출 |

#### 사용 패턴

```rust
// 상태 머신 생성 (초기 상태 "idle", 클립 인덱스 0)
let mut sm = AnimationStateMachine::new("idle", 0);
sm.add_state("run", 1)
  .add_state("jump", 2);

// 파라미터 등록
sm.set_bool("is_running", false);
sm.add_trigger("jump");

// 전환 등록
sm.add_transition("idle", "run",  vec![TransitionCond::BoolEq("is_running".into(), true)]);
sm.add_transition("run",  "idle", vec![TransitionCond::BoolEq("is_running".into(), false)]);
sm.add_transition("idle", "jump", vec![TransitionCond::Trigger("jump".into())]);
sm.add_transition("run",  "jump", vec![TransitionCond::Trigger("jump".into())]);
sm.add_transition("jump", "idle", vec![TransitionCond::AnimationEnd]);

world.add_component(player_entity, sm);

// 게임 로직에서 파라미터 조작
world.get_mut::<AnimationStateMachine>(player).unwrap().set_bool("is_running", true);
world.get_mut::<AnimationStateMachine>(player).unwrap().fire_trigger("jump");
```

#### 시스템 등록 순서

```rust
app.add_system(Box::new(AnimationSystem));     // 프레임 진행 + UvRect 동기화
app.add_system(Box::new(StateMachineSystem));  // 전환 조건 평가 → play() 호출
```

`StateMachineSystem`이 `AnimationSystem` **이후에** 실행되어야 `is_finished()` 판정이 같은 프레임에 반영된다.

#### 트리거 소비 규칙

트리거는 `StateMachineSystem`이 실행될 때마다 소비된다(전환 여부와 무관). 따라서 한 프레임에 `fire_trigger()`를 호출해야 하며, 전환 조건이 없는 상태에서 활성화하면 그 프레임 내에 버려진다.

#### `AnimationPlayer` 변경

`is_finished() -> bool` 메서드 추가 — non-looping 클립의 마지막 프레임이면 `true`. `AnimationEnd` 조건의 기반.

---

## 이번 세션에서 한 일 (Phase 13)

### Phase 13 — 물리 레이캐스트 + 캐릭터 컨트롤러

**배경**: 시야 판정·마우스 픽킹·총기 탄착 계산 등을 위한 레이캐스트가 없었고, 경사면·계단 처리를 포함하는 게임 특화 캐릭터 이동 기능이 필요했다.

**추가된 파일**: `src/physics/character.rs`  
**변경된 파일**: `src/physics/world.rs`, `src/physics/mod.rs`, `src/lib.rs`

#### 레이캐스트 (`PhysicsWorld`)

```rust
// 단순 레이캐스트 — 최초 충돌 콜라이더 핸들 + toi
let result: Option<(ColliderHandle, f32)> =
    physics.cast_ray(origin_physics, dir, max_toi, solid);

// 법선 포함 — RaycastHit { collider_handle, point, normal, toi }
let hit: Option<RaycastHit> =
    physics.cast_ray_with_normal(origin_physics, dir, max_toi, solid);
```

- 모든 좌표는 **물리 단위** (픽셀 ÷ pixels_per_unit).
- `step()` 이후 `query_pipeline`이 갱신된 뒤에 호출해야 최신 상태가 반영된다.

#### 키네마틱 바디

```rust
// 중력 비반응, 수동 위치 제어
let (rb, col) = physics.add_kinematic_box(pos / PPU, half_w, half_h);
let (rb, col) = physics.add_kinematic_circle(pos / PPU, radius);
```

#### 캐릭터 컨트롤러 (`CharacterController` 컴포넌트)

```rust
use engine::{CharacterController, PhysicsBody};

// 엔티티 생성
let (rb, col) = physics.add_kinematic_box(start / PPU, 0.4, 0.9);
world.add_component(player, PhysicsBody { rigid_body_handle: rb, collider_handle: col });
world.add_component(player, CharacterController::new()
    .with_max_slope_deg(45.0)
    .with_snap_to_ground(0.15));

// 커스텀 시스템 run() 내 — PhysicsSystem 이전에 등록 필수
let desired = Vec2::new(move_x * speed * dt, gravity_vel * dt);
physics.move_character(
    controller, body.rigid_body_handle, body.collider_handle,
    desired, dt, PIXELS_PER_UNIT,
);
if controller.grounded { /* 접지 = 점프 가능 */ }
```

**구조 특이사항**
- `CharacterController::inner`의 `up = -Y` — 엔진 화면 좌표(Y+는 아래)에 맞춰 설정.
  Rapier 기본값(+Y)을 그대로 쓰면 바닥/천장 판정이 뒤집힌다.
- `move_character()`는 내부적으로 `set_next_kinematic_translation()`을 호출하므로
  다음 `step()` 때 위치가 실제로 반영된다.
- `PhysicsSystem::run()` 이전에 캐릭터 이동을 처리하는 전용 시스템을 등록해야 올바른 순서로 동작한다.

**신규 테스트** (`src/physics/world.rs`): 7개
- `cast_ray_hits_static_box` — 정적 박스에 레이 충돌 확인
- `cast_ray_misses_when_no_obstacle` — 장애물 없으면 None
- `cast_ray_with_normal_returns_correct_normal` — 법선 방향 검증
- `add_kinematic_box_creates_body` — 키네마틱 바디 생성
- `add_kinematic_circle_creates_body` — 키네마틱 원형 바디 생성
- `move_character_grounded_on_floor` — 바닥 위 접지 판정
- `character_controller_builder_methods` — 빌더 메서드 파라미터 설정

**검증**: `cargo test` 61개 단위 + 11개 doc 테스트 전부 통과 (`rust-survivors` 빌드 무영향)

---

## 이번 세션에서 한 일 (Phase 12)

### Phase 12 — Transform 계층 (Parent · Children · GlobalTransform)

**배경**: 무기 부착, 복합 캐릭터 구성 등 엔티티 간 변환 종속성이 필요했으나, 기존 `Transform`은 독립 로컬 값만 저장하는 플랫 구조였다.

**추가된 파일**: `src/hierarchy.rs`

**신규 컴포넌트·타입**
- `Parent(Entity)` — 부모 엔티티를 가리키는 컴포넌트
- `Children(Vec<Entity>)` — 자식 엔티티 목록 (부모 측에 보관)
- `GlobalTransform { position, scale, rotation, z }` — 매 프레임 HierarchySystem이 계산하는 월드 공간 변환 (`Copy`)
- `HierarchySystem` — `System` 구현체. `App`이 유저 시스템 직후 자동 실행 (등록 불필요)
- `attach(world, child, parent)` — Parent + Children 동시 관리 헬퍼
- `detach(world, child)` — 부모 연결 해제 헬퍼

**렌더러 통합** (`src/renderer/sprite.rs`)
- `InstanceRaw::from_global()` 추가
- `render()` 루프: `GlobalTransform` 있으면 우선 사용, 없으면 `Transform` fallback → **완전 하위 호환**

**App 자동 실행** (`src/app.rs`)
```
유저 시스템(물리 포함) → HierarchySystem → 이벤트 flush → 렌더
```
물리가 `Transform.position`을 갱신한 직후 계층 전파가 실행되므로 항상 정확한 월드 변환이 보장된다.

**깊이 제한**: 내부 2-pass 구조로 최대 3단계 (루트 → 자식 → 손자) 지원.

**사용 패턴**
```rust
use engine::{attach, Transform};
use glam::Vec2;

// 무기를 플레이어에 부착
attach(&mut world, weapon, player);

// 로컬 오프셋 설정 — GlobalTransform은 매 프레임 자동 계산
world.get_mut::<Transform>(weapon).unwrap().position = Vec2::new(30.0, 0.0);
// → weapon의 GlobalTransform.position = player.position + (30, 0) rotated by player.rotation
```

**검증**: `cargo build` + `cargo test` (rust-2d-engine, rust-survivors 96개 테스트 전부 통과)

---

## 이번 세션에서 한 일 (Phase 9~11)

### Phase 9 — ECS Archetype 스토리지

**배경**: 기존 ECS는 `HashMap<TypeId, Vec<Option<Box<dyn Any>>>>` 구조로, 엔티티 수가 늘면 쿼리 루프마다 `None` 체크가 발생했다.

**변경**: `src/ecs/world.rs` 전면 재작성 — Archetype 기반 밀집 컬럼 스토리지.
- `Archetype` 내부 구조: `type_set: Vec<TypeId>` (정렬) + `entities: Vec<Entity>` + `columns: HashMap<TypeId, Vec<Box<dyn Any>>>`
- 같은 컴포넌트 집합을 가진 엔티티들이 동일 Archetype에 모이므로 쿼리 시 `None` 체크 불필요
- `add_component` / `remove_component` 시 `move_entity()` 헬퍼로 Archetype 간 이동 (swap_remove + 위치 맵 업데이트)
- 공개 API 완전 호환 유지: `spawn`, `despawn`, `get`, `get_mut`, `query1~4`, `query_opt2`, `entities()`, 리소스 메서드
- 신규 테스트 2개 추가: `archetype_reuse_across_entities`, `add_component_replaces_existing` (총 14개)

**아키텍처 결정**: `entities: Vec<Entity>` 보조 필드를 유지해 `entities() -> &[Entity]` 시그니처를 변경 없이 보존.

### Phase 10 — 포스트프로세싱

**추가된 파일**
- `src/renderer/post_process.rs`: `PostProcessConfig` + `PostProcessRenderer`
- `src/renderer/shaders/post_process.wgsl`: 비네팅·색수차·근사 블룸 WGSL 셰이더

**구조**
1. `PostProcessConfig` 리소스를 World에 삽입하고 `enabled: true` 설정
2. `App::render()` 가 중간 텍스처(`target_view`)에 씬 전체를 렌더링
3. 포스트프로세스 패스: 중간 텍스처 → 스왑체인 (풀스크린 삼각형, 버텍스 버퍼 불필요)

**효과 설명**
- **비네팅**: 화면 가장자리 어두움 (`vignette_strength`, `vignette_radius`)
- **색수차**: RGB 채널을 방사형으로 다른 UV에서 샘플 (`chroma_offset`)
- **근사 블룸**: 4-tap threshold 샘플링으로 밝은 영역 번짐 (`bloom_threshold`, `bloom_intensity`)

**사용 패턴**
```rust
app.world.insert_resource(PostProcessConfig {
    enabled: true,
    vignette_strength: 0.5,
    chroma_offset: 0.003,
    bloom_intensity: 0.4,
    ..Default::default()
});
```

**주의**: 리소스 없거나 `enabled: false`면 중간 텍스처 패스 완전 건너뜀 (제로 오버헤드).

### Phase 11 — 오디오 강화

**변경 파일**: `src/audio.rs` (기존 API 완전 호환 유지)

**추가된 기능**

#### 위치 오디오
```rust
// 1회성 위치 재생
am.play_at("sfx", "boom.wav", false, source_pos, listener_pos, 500.0);

// 움직이는 소리 발생원 — 매 프레임 호출
am.update_position("sfx", enemy_pos, player_pos, 500.0);
```
- `(볼륨, 팬)` = 거리 선형 감쇠 + X 방향 스테레오 팬 자동 계산

#### 오디오 버스 믹서
```rust
am.assign_bus("bgm",      "music");
am.assign_bus("sfx_jump", "sfx");
am.set_bus_volume("music", 0.5);   // 음악 전체 절반으로
am.set_bus_volume("sfx",   0.8);   // 효과음 전체 80%
```

#### 페이드
```rust
am.play_fade_in("bgm", "music.ogg", true, 2.0);  // 2초 페이드인
am.fade_out("bgm", 3.0);                          // 3초 페이드아웃 후 정지
am.fade_volume("sfx", 0.3, 1.5);                  // 1.5초 동안 0.3으로

// System::run() 내에서 매 프레임 호출 필수
world.resource_mut::<AudioManager>().map(|am| am.update(dt));
```

**테스트**: 위치 오디오 파라미터 계산 4개 (`spatial_params_*`)

---

## 이번 세션에서 한 일 (Phase 8)

### Save/Load 완성

**추가된 함수**
- `load_or_default<T: DeserializeOwned + Default>(path)` — 파일 없으면 `Default::default()` 반환, 파싱 에러는 그대로 전파
- `exists(path) -> bool` — 저장 파일 존재 여부 확인
- `delete(path) -> Result<(), SaveError>` — 저장 파일 삭제 (없으면 Ok)

**lib.rs re-export 추가**: `save`, `load`, `load_or_default`, `exists`, `delete`, `save_path`, `SaveError` 최상위 노출

**테스트 추가**: `load_or_default_returns_default_when_missing`, `load_or_default_returns_saved_value`, `exists_and_delete` (총 5개 → 전부 통과)

**사용 패턴**
```rust
use engine::{load_or_default, save, save_path, delete, exists};

#[derive(Serialize, Deserialize, Default)]
struct SaveData { score: u32, level: u32 }

let path = save_path("my-game", "save.ron");

// 게임 시작 — 없으면 기본값
let data: SaveData = load_or_default(&path)?;

// 게임 저장
save(&path, &data)?;

// 세이브 존재 확인
if exists(&path) { ... }

// 세이브 삭제
delete(&path)?;
```

---

## 이전 세션에서 한 일 (Phase 7)

### 물리 충돌 이벤트 — ECS 브리징

**배경**: `PhysicsPipeline::step()`이 contact handler를 `&()`(no-op)으로 고정해 충돌 시작/종료를 게임 로직에서 감지할 수 없었다.

**구현 방식**: Rapier `EventHandler` 트레잇 구현 대신 `NarrowPhase` 폴링 선택. `step()` 이후 `narrow_phase.contact_pairs()`를 반복해 이전 프레임 접촉 집합과 diff → `Events<CollisionEvent>` 전송. `Mutex`/`RefCell` 불필요, 기존 `has_contact()` 패턴과 일관성 유지.

**추가된 파일/변경**
- `src/physics/events.rs` (신규): `CollisionEvent { Started(Entity, Entity), Stopped(Entity, Entity) }` — `Copy + Clone`
- `src/physics/system.rs`: `active_contacts: HashSet<(ColliderHandle, ColliderHandle)>` 필드, `run()` 내 diff 블록
- `src/physics/mod.rs`: `pub mod events` + `CollisionEvent` re-export
- `src/lib.rs`: `CollisionEvent` 최상위 re-export

**사용 패턴**
```rust
app.register_event::<CollisionEvent>();         // 필수: 이벤트 버스 등록
app.add_system(Box::new(PhysicsSystem::new(physics, 50.0)));
app.add_system(Box::new(MySystem));             // PhysicsSystem 뒤 등록 → 같은 프레임 수신

// MySystem::run() 내
if let Some(events) = world.resource::<Events<CollisionEvent>>() {
    for ev in events.read() {
        match ev {
            CollisionEvent::Started(a, b) => { /* 충돌 시작 */ }
            CollisionEvent::Stopped(a, b) => { /* 충돌 종료 */ }
        }
    }
}
```

**주의**: ECS에 `PhysicsBody`가 없는 static 콜라이더(바닥 등)와의 충돌은 `col_map.get()` 실패로 조용히 스킵. 이벤트 미등록 시에도 패닉 없음(`resource_mut` → `None` guard).

---

## 이전 세션에서 한 일 (Phase 6)

### UI 시스템 강화

**TextInput** (`src/ui/text_input.rs`)
- `UiNode` + `TextInput` 엔티티로 텍스트 입력 필드 구성
- UTF-8 byte index 기반 커서 (`backspace()` 멀티바이트 안전)
- 커서 깜빡임: dt 누적, 0.5초마다 토글
- 이벤트: `TextChanged`, `TextSubmitted`, `TextFocused`, `TextBlurred`

**ScrollView** (`src/ui/scroll_view.rs`)
- `UiNode` + `ScrollView` 엔티티로 스크롤 목록 구성
- 자식 엔티티 없이 내부 `items: Vec<String>` 직접 렌더링
- 커서가 위젯 위에 있을 때 마우스 휠로 스크롤
- `clamp_scroll(view_height)` — 범위 초과 방지

**Panel + LayoutSystem** (`src/ui/panel.rs`)
- `UiNode` + `Panel` 엔티티: 자식 엔티티 자동 배치 (`Vertical` / `Horizontal`)
- `LayoutSystem`: UiSystem 이전에 실행 — 자식 `UiNode.offset`을 절대 스크린 좌표로 재계산
- 등록 순서 필수: `add_system(Box::new(LayoutSystem))` → `add_system(Box::new(UiSystem))`

**InputState 문자 버퍼** (`src/input/state.rs`)
- `text_input_chars: Vec<char>` 필드 추가
- `text_chars() -> &[char]` 공개 읽기 / `push_char`, `push_backspace`, `push_enter` (pub(crate))
- `app.rs`에서 `logical_key`로 문자 추출 → 버퍼 기록 (센티넬: `'\x08'` = Backspace, `'\n'` = Enter)

**UiEvent 확장**
- `Copy` 제거, `Clone` 유지 (String 포함 필요)
- 기존 `ButtonClicked` 보존 + `TextChanged`, `TextSubmitted`, `TextFocused`, `TextBlurred` 추가

---

## 알아야 할 아키텍처 결정

### 렌더러 분리
`AnimationPlayer`를 렌더러가 직접 참조하지 않는다. `AnimationSystem`이 `UvRect` 컴포넌트를 동기화하고, 렌더러는 `UvRect`만 읽는다. 레이어 경계 위반을 막기 위한 구조.

### DebugDrawQueue → UiQueue 변환
`DebugDrawQueue`는 순수 데이터(`DebugRect`)를 담고, `App`의 render 단계에서 `DrawRect`로 변환해 `UiQueue`에 넣는다. 시스템 레이어가 렌더러 타입에 의존하지 않도록 하는 설계.

### PhysicsWorld 캡슐화
내부 rapier2d 필드는 `pub(crate)`. 외부에서는 `rigid_body()`, `rigid_body_mut()`, `get_collider()`, `get_collider_mut()`, `add_dynamic_circle()`, `remove_body()` 접근자만 사용한다.

### ECS borrow 충돌 우회
Rust borrow checker 제약상 쿼리 중 `get_mut`을 바로 섞을 수 없다. 표준 패턴: 먼저 `.collect()`로 엔티티 목록을 뽑고, 순회하며 `get_mut` 호출.

### UI 문자 입력 버퍼 (Phase 6~)
`InputState.text_chars()` — 이번 프레임 입력 문자 슬라이스. `UiSystem`이 소비하고, `flush()`에서 초기화. `TextInput`이 포커스된 엔티티만 이 버퍼를 처리한다.

### LayoutSystem 실행 순서 (Phase 6~)
`Panel` 자식의 위치는 `LayoutSystem`이 계산한다. `UiSystem` 보다 반드시 먼저 등록해야 올바른 위치로 렌더된다.

---

## 미해결 / 다음 Phase 후보

| Phase | 기능 | 난이도 | 비고 |
|-------|------|--------|------|
| ~~Phase 13~~ | ~~물리 레이캐스트 + 캐릭터 컨트롤러~~ | — | 완료 |
| ~~Phase 14~~ | ~~애니메이션 상태 머신~~ | — | 완료 |
| ~~Phase 15~~ | ~~게임패드(gilrs) + UI Slider/CheckBox~~ | — | 완료 |
| ~~Phase 16~~ | ~~씬 직렬화 + 프리팹 시스템~~ | — | 완료 |
| Phase 17 | 에셋 파이프라인 + 핫 리로딩 | XL | Handle 기반, Breaking Change 포함 |

---

## 연관 저장소

| 저장소 | 역할 | 경로 |
|---|---|---|
| `rust-2d-engine` | 엔진 코어 (이 저장소) | `/Volumes/SSD/Projects/rust-2d-engine` |
| `rust-survivors` | 엔진을 사용하는 게임 프로젝트 | `/Volumes/SSD/Projects/rust-survivors` |

두 저장소는 **독립적으로** 개발된다. 엔진 개선은 `rust-2d-engine`에서만, 게임 로직은 `rust-survivors`에서만.

---

## 참고 문서

- `REFERENCE.md` — 공개 API 레퍼런스 (코드 예제 포함)
- `src/` 각 파일 인라인 doc comment — 세부 구현 의도 기록됨
