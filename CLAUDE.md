# CLAUDE.md — rust-2d-engine 에이전트 참조 문서

> 버전 v0.18.0 | wgpu 기반 Rust 2D 게임 엔진 | 단일 크레이트 (`engine`)  
> 상세 API: `REFERENCE.md` | 개발 이력/아키텍처 결정: `HANDOFF.md`

---

## 모듈 맵

무엇을 찾으려면 어디를 읽어야 하는가:

| 찾는 것 | 파일 |
|---------|------|
| 엔진 진입점, 메인 루프, 렌더링 오케스트레이션, `load_image` | `src/app.rs` |
| Handle<T>, ImageAsset, AssetServer (에셋 로드·캐싱·핫 리로딩) | `src/asset.rs` |
| DebugUi (egui 오버레이, F1 토글, `ctx()`로 커스텀 패널) | `src/debug_ui.rs` |
| 전체 공개 API re-export 목록 | `src/lib.rs` |
| Entity / Component / Resource / Query | `src/ecs/world.rs` |
| 이벤트 버스 (`Events<E>`) | `src/ecs/events.rs` |
| `System` 트레잇 | `src/ecs/system.rs` |
| 씬 전환 (Scene, SceneCmd, SceneChange) | `src/scene.rs` |
| Transform, Sprite | `src/components.rs` |
| WindowConfig, GameState, ShouldQuit, DebugDrawQueue | `src/resources.rs` |
| Camera (좌표 변환, zoom) | `src/camera.rs` |
| InputState, InputMap | `src/input/` |
| GamepadState, GamepadButton, GamepadAxis | `src/input/gamepad.rs` |
| PhysicsWorld, PhysicsBody, PhysicsSystem, CollisionEvent | `src/physics/` |
| CharacterController, RaycastHit, cast_ray, cast_ray_with_normal, move_character | `src/physics/character.rs`, `src/physics/world.rs` |
| add_kinematic_box, add_kinematic_circle | `src/physics/world.rs` |
| SpatialGrid, Collider, CollisionLayer | `src/collision/` |
| AnimationPlayer, AnimationClip, AnimationSystem | `src/animation/player.rs`, `src/animation/system.rs` |
| AnimationStateMachine, StateMachineSystem, TransitionCond, AnimParam | `src/animation/state_machine.rs` |
| UI (UiNode, Button, Label, TextInput, ScrollView, Panel, LayoutSystem, UiEvent) | `src/ui/` |
| Slider (수평 슬라이더), CheckBox (토글 체크박스) | `src/ui/slider.rs`, `src/ui/checkbox.rs` |
| Tag, EntityDef, SceneDef, Prefab, spawn_entity_def, spawn_scene_def | `src/prefab.rs` |
| Timer, Tween, Easing | `src/timer.rs`, `src/tween.rs` |
| ParticleEmitter, ParticleSystem | `src/particle.rs` |
| Tilemap, TilemapAtlas, TilemapSystem | `src/tilemap.rs` |
| AudioManager (재생, 위치 오디오, 버스 믹서, 페이드) | `src/audio.rs` |
| save / load / load_or_default / exists / delete / save_path / SaveError | `src/save.rs` |
| PostProcessConfig, PostProcessRenderer | `src/renderer/post_process.rs` |
| wgpu 렌더링 파이프라인 (직접 수정 드묾) | `src/renderer/` |

---

## 핵심 아키텍처 패턴

### ECS 쿼리 API

```rust
// 단일 컴포넌트
for (entity, comp) in world.query::<MyComp>() { ... }

// 복수 컴포넌트 (query2 / query3 / query4)
for (e, a, b) in world.query2::<A, B>() { ... }

// A 필수, B 선택
for (e, a, b_opt) in world.query_opt2::<A, B>() { ... }

// System 시그니처
impl System for MySystem {
    fn run(&mut self, world: &mut World, dt: f32) { ... }
}
```

### Borrow Checker 우회 패턴 (필수)

쿼리 이터레이터가 살아있는 동안 같은 World에서 `get_mut` 불가. 표준 패턴:

```rust
// 먼저 엔티티 목록을 collect, 이후 순회하며 get_mut
let entities: Vec<Entity> = world.query::<Foo>().map(|(e, _)| e).collect();
for entity in entities {
    world.get_mut::<Foo>(entity).unwrap().update();
}
```

### 렌더링 레이어 분리

- `AnimationSystem` → `UvRect` 컴포넌트 동기화 → 렌더러는 `UvRect`만 읽음  
  (렌더러가 `AnimationPlayer`를 직접 참조하면 레이어 위반)
- `DebugDrawQueue` = 순수 데이터(`DebugRect`) → `App` render 단계에서 `DrawRect`로 변환
- 렌더 순서: Systems → Events flush → Input flush → Scene 명령 처리 → Render (스프라이트 → UI → 텍스트)

### UI 시스템 등록 순서

`Panel`을 사용할 때는 `LayoutSystem`을 `UiSystem` **앞에** 등록해야 한다:

```rust
app.add_system(Box::new(LayoutSystem));  // 자식 UiNode.offset 재계산
app.add_system(Box::new(UiSystem));      // 위치 읽어 렌더
```

`UiEvent`는 `Copy` 없이 `Clone`만 구현한다 (TextChanged/TextSubmitted가 String 포함).  
`InputState::text_chars()` — 이번 프레임 문자 슬라이스. `'\x08'`=Backspace, `'\n'`=Enter.

### 애니메이션 상태 머신 등록 순서

`StateMachineSystem`은 `AnimationSystem` **이후에** 등록해야 `is_finished()` 판정이 같은 프레임에 반영된다:

```rust
app.add_system(Box::new(AnimationSystem));     // 프레임 진행 + UvRect 동기화
app.add_system(Box::new(StateMachineSystem));  // 전환 조건 평가 → play() 호출
```

파라미터 조작은 시스템 내에서 `world.get_mut::<AnimationStateMachine>(entity)` 로 접근한다:

```rust
sm.set_bool("is_running", true);   // BoolEq 조건용
sm.set_float("speed", 3.5);        // FloatGt / FloatLt 조건용
sm.fire_trigger("jump");           // Trigger 조건용 (매 프레임 자동 소비)
```

`TransitionCond::AnimationEnd`는 non-looping 클립의 마지막 프레임 도달 시 참이 된다.

### PhysicsWorld 캡슐화

내부 rapier2d 필드는 `pub(crate)`. 외부에서 직접 접근 금지. 사용 가능한 접근자:

```
rigid_body() / rigid_body_mut()
get_collider() / get_collider_mut()
add_dynamic_circle() / add_dynamic_box() / add_static_box()
remove_body()
```

---

## 자주 하는 작업 패턴

### 새 컴포넌트 추가

1. `src/components.rs` 또는 해당 모듈 파일에 struct 정의
2. `src/lib.rs`에 re-export 추가

### 새 시스템 추가

1. `System` 트레잇 구현
2. `app.add_system(Box::new(MySystem))` 또는 `Scene::on_enter`에서 등록

### 새 리소스 추가

1. `src/resources.rs`에 struct 정의
2. `app.world.insert_resource(MyResource { ... })` 로 등록
3. 필요 시 `src/lib.rs` re-export 추가

### 새 이벤트 추가

```rust
// 1. 타입 정의 (Clone + 'static 필요)
#[derive(Clone)]
struct MyEvent { pub data: f32 }

// 2. App 설정 시 등록
app.register_event::<MyEvent>();

// 3. 시스템 내 사용
world.resource_mut::<Events<MyEvent>>().unwrap().send(MyEvent { data: 1.0 });
for ev in world.resource::<Events<MyEvent>>().unwrap().read() { ... }
```

### 씬 전환

```rust
world.resource_mut::<SceneChange>().unwrap().0 =
    Some(SceneCmd::Replace(Box::new(MyScene)));
// SceneCmd::Push(Box::new(MyScene)) — 스택에 추가
// SceneCmd::Pop                      — 이전 씬으로
```

---

## 에이전트 작업 유의사항

### 컨텍스트 관리

세션이 길어질수록 누적 컨텍스트가 응답 품질을 낮춘다. 작업 유형에 따라 방식을 분리한다:

| 상황 | 권장 방식 |
|------|-----------|
| 단일 파일 수정 (요건 명확) | 메인 세션에서 직접 수정 |
| 여러 파일에 걸친 기능 구현 | Task 서브 에이전트로 분리 |
| 탐색이 3개 파일 이상 필요 | Explore 서브 에이전트 |
| 대화가 길어진 후 코드 작성 | Task 서브 에이전트 (컨텍스트 오염 방지) |

### 탐색 효율화

- 파일 전체 읽기 전 `grep`으로 심볼/키워드 먼저 위치 확인
- 경로가 이미 알려진 경우 Read 직접 사용 (Explore 서브 에이전트 불필요)
- 읽기 순서: `src/lib.rs` → 모듈 맵 → 대상 파일로 좁혀 읽기

### 서브 에이전트 프롬프트 원칙

서브 에이전트는 현재 대화 컨텍스트를 모른 채 시작한다. 프롬프트에 반드시 포함:

1. **수정할 파일 경로** (절대 경로)
2. **적용할 패턴** — borrow 우회, 레이어 분리 등 이 파일의 핵심 패턴 섹션 요약을 전달
3. **기대 결과물** — 어떤 동작이 달라져야 하는가

---

## 연관 프로젝트

| 저장소 | 경로 | 역할 |
|--------|------|------|
| rust-2d-engine | `/Volumes/SSD/Projects/rust-2d-engine` | 엔진 코어 (이 저장소) |
| rust-survivors | `/Volumes/SSD/Projects/rust-survivors` | 엔진을 사용하는 게임 프로젝트 |

`rust-survivors`는 `engine = { git = "https://github.com/ChunSam/rust-2d-engine" }` 의존.  
엔진 공개 API의 파괴적 변경 시 game 측 영향 확인 필요.

---

## 문서 지도

| 문서 | 용도 |
|------|------|
| `CLAUDE.md` (이 파일) | 에이전트 빠른 참조 — 모듈 맵, 핵심 패턴, 작업 체크리스트 |
| `REFERENCE.md` | 전체 공개 API + 코드 예제 (상세) |
| `HANDOFF.md` | Phase별 개발 이력, 아키텍처 결정 배경 |

> **확장 전략**: 새 서브시스템이 생기면 `docs/SUBSYSTEM.md`를 별도 작성하고, 이 파일 모듈 맵에 한 줄 참조만 추가해 200줄 이내를 유지한다.
