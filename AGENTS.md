# AGENTS.md — skeleton-engine 에이전트 참조

> 버전 v1.0.0 | 패키지 `skeleton-engine` | 라이브러리 크레이트 `engine`
> wgpu 기반 Rust 2D 게임 엔진 | 상세 API: `REFERENCE.html` | 개발 이력: `docs/HANDOFF.md`

## 모듈 맵

| 찾는 것 | 파일 |
| --- | --- |
| 엔진 진입점, 메인 루프, 렌더링 오케스트레이션, `load_image` | `src/app.rs` |
| `Handle<T>`, `ImageAsset`, `ScriptAsset`, `AssetServer` | `src/asset.rs` |
| `TextureAtlas`, `AtlasSprite` | `src/atlas.rs` |
| `Reflect`, `ReflectValue` | `src/reflect.rs` |
| `ScriptAsset`, `ScriptRunner`, `ScriptingSystem` | `src/scripting.rs` |
| `DebugUi` | `src/debug_ui.rs` |
| 전체 공개 API re-export 목록 | `src/lib.rs` |
| `Entity`, `Component`, `Resource`, `Query` | `src/ecs/world.rs` |
| 이벤트 버스 `Events<E>` | `src/ecs/events.rs` |
| `System` 트레잇 | `src/ecs/system.rs` |
| 씬 전환: `Scene`, `SceneCmd`, `SceneChange` | `src/scene.rs` |
| `Transform`, `Sprite` | `src/components.rs` |
| `WindowConfig`, `GameState`, `ShouldQuit`, `DebugDrawQueue` | `src/resources.rs` |
| `Camera` | `src/camera.rs` |
| `InputState`, `InputMap` | `src/input/` |
| `GamepadState`, `GamepadButton`, `GamepadAxis` | `src/input/gamepad.rs` |
| `PhysicsWorld`, `PhysicsBody`, `PhysicsSystem`, `CollisionEvent` | `src/physics/` |
| `CharacterController`, `RaycastHit`, raycast, character movement | `src/physics/character.rs`, `src/physics/world.rs` |
| `add_kinematic_box`, `add_kinematic_circle` | `src/physics/world.rs` |
| `SpatialGrid`, `Collider`, `CollisionLayer` | `src/collision/` |
| `AnimationPlayer`, `AnimationClip`, `AnimationSystem`, `BlendWeight` | `src/animation/player.rs`, `src/animation/system.rs` |
| `AnimationStateMachine`, `StateMachineSystem`, `TransitionCond`, `AnimParam` | `src/animation/state_machine.rs` |
| `BlendTree1D`, `BlendEntry`, `BlendTreeSystem` | `src/animation/blend_tree.rs`, `src/animation/blend_system.rs` |
| UI: `UiNode`, `Button`, `Label`, `TextInput`, `ScrollView`, `Panel`, `LayoutSystem`, `UiEvent` | `src/ui/` |
| `Slider`, `CheckBox` | `src/ui/slider.rs`, `src/ui/checkbox.rs` |
| `Tag`, `EntityDef`, `SceneDef`, `Prefab`, prefab spawn 함수 | `src/prefab.rs` |
| `Timer`, `Tween`, `Easing` | `src/timer.rs`, `src/tween.rs` |
| `ParticleEmitter`, `ParticleSystem` | `src/particle.rs` |
| `Tilemap`, `TilemapAtlas`, `TilemapSystem` | `src/tilemap.rs` |
| `AudioManager` | `src/audio.rs` |
| save/load API, `SaveError` | `src/save.rs` |
| `PostProcessConfig`, `PostProcessRenderer` | `src/renderer/post_process.rs` |
| wgpu 렌더링 파이프라인 | `src/renderer/` |

## 핵심 아키텍처 패턴

### ECS 쿼리 API

```rust
// 단일 컴포넌트
for (entity, comp) in world.query::<MyComp>() { ... }

// 복수 컴포넌트: query2 / query3 / query4
for (e, a, b) in world.query2::<A, B>() { ... }

// A 필수, B 선택
for (e, a, b_opt) in world.query_opt2::<A, B>() { ... }

// System 시그니처
impl System for MySystem {
    fn run(&mut self, world: &mut World, dt: f32) { ... }
}
```

### Borrow Checker 우회 패턴

쿼리 이터레이터가 살아있는 동안 같은 `World`에서 `get_mut` 불가. 먼저 엔티티 목록을 수집한 뒤 수정한다.

```rust
let entities: Vec<Entity> = world.query::<Foo>().map(|(e, _)| e).collect();
for entity in entities {
    world.get_mut::<Foo>(entity).unwrap().update();
}
```

### 렌더링 레이어 분리

- `AnimationSystem` → `UvRect` 컴포넌트 동기화 → 렌더러는 `UvRect`만 읽음
- 렌더러가 `AnimationPlayer`를 직접 참조하면 레이어 위반
- `DebugDrawQueue`는 순수 데이터(`DebugRect`)만 담고, `App` render 단계에서 `DrawRect`로 변환
- 렌더 순서: Systems → Events flush → Input flush → Scene 명령 처리 → Render(스프라이트 → UI → 텍스트)

### UI 시스템 등록 순서

`Panel`을 사용할 때는 `LayoutSystem`을 `UiSystem` 앞에 등록한다.

```rust
app.add_system(Box::new(LayoutSystem)); // 자식 UiNode.offset 재계산
app.add_system(Box::new(UiSystem));     // 위치 읽어 렌더
```

- `UiEvent`는 `Copy` 없이 `Clone`만 구현한다. `TextChanged`/`TextSubmitted`가 `String`을 포함한다.
- `InputState::text_chars()`는 이번 프레임 문자 슬라이스다. `'\x08'` = Backspace, `'\n'` = Enter.

### 애니메이션 상태 머신 등록 순서

`StateMachineSystem`은 `AnimationSystem` 이후에 등록해야 `is_finished()` 판정이 같은 프레임에 반영된다.

```rust
app.add_system(Box::new(AnimationSystem));    // 프레임 진행 + UvRect 동기화
app.add_system(Box::new(StateMachineSystem)); // 전환 조건 평가 -> play() 호출
```

파라미터는 시스템 내에서 `world.get_mut::<AnimationStateMachine>(entity)`로 접근한다.

```rust
sm.set_bool("is_running", true); // BoolEq 조건용
sm.set_float("speed", 3.5);      // FloatGt / FloatLt 조건용
sm.fire_trigger("jump");         // Trigger 조건용, 매 프레임 자동 소비
```

`TransitionCond::AnimationEnd`는 non-looping 클립의 마지막 프레임 도달 시 참이다.

### PhysicsWorld 캡슐화

내부 rapier2d 필드는 `pub(crate)`이므로 외부에서 직접 접근하지 않는다. 사용 가능한 접근자:

```text
rigid_body() / rigid_body_mut()
get_collider() / get_collider_mut()
add_dynamic_circle() / add_dynamic_box() / add_static_box()
remove_body()
```

## 자주 하는 작업 패턴
| 작업 | 순서 |
| --- | --- |
| 새 컴포넌트 | `src/components.rs` 또는 해당 모듈에 struct 정의 → `src/lib.rs` re-export |
| 새 시스템 | `System` 트레잇 구현 → `app.add_system(Box::new(MySystem))` 또는 `Scene::on_enter`에서 등록 |
| 새 리소스 | `src/resources.rs`에 struct 정의 → `app.world.insert_resource(MyResource { ... })` 등록 → 필요 시 re-export |

### 새 이벤트 추가

```rust
// 1. 타입 정의: Clone + 'static 필요
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

// SceneCmd::Push(Box::new(MyScene)) - 스택에 추가
// SceneCmd::Pop                     - 이전 씬으로
```

## 에이전트 작업 유의사항

상세 운영 규칙은 `docs/AGENT_WORKFLOW.md`를 따른다. `AGENTS.md`는 빠른 참조 문서이므로 **항상 200줄 이내**로 유지한다. 중요 내용 때문에 200줄을 초과할 경우 `docs/*.md` 문서를 새로 만들고, 이 파일에는 요약과 링크만 둔다.

### 기본 흐름

- 탐색 → 범위 판단 → 필요 시 계획 → 구현 → 검증 → 요약 보고 순서로 진행한다.
- 파일 전체를 읽기 전 `rg`로 심볼/키워드 위치를 먼저 확인하고, 기본 읽기 순서는 `src/lib.rs` → 모듈 맵 → 대상 파일이다.
- 단일 파일·요건 명확한 수정은 메인 세션에서 직접 처리한다.
- 3개 이상 파일 탐색, 여러 서브시스템 변경, 긴 대화 후 구현, 병렬 검토가 유리한 작업은 서브 에이전트를 적극 사용한다.
- 공개 API/사용법/예제 영향이 있으면 관련 문서 갱신 필요성을 확인한다.
- 기본 검증은 엔진 저장소 기준으로 수행한다. `rust-survivors` 확인은 사용자 요청 또는 명시적 필요가 있을 때만 수행한다.
- stage/commit/push는 사용자 요청 시에만 수행한다.
- 공개 API 삭제/이름변경, 의존성·버전 변경, 대량 리팩터, 파일 삭제, destructive Git은 사전 확인한다.
- 서브 에이전트 프롬프트에는 파일 경로, 적용 패턴, 기대 동작, 변경 금지 범위를 포함한다.

## 연관 프로젝트
- `skeleton-engine`: `/Users/jkl/Projects/skeleton-engine` — 엔진 코어, 이 저장소
- `rust-survivors`: `/Users/jkl/Projects/rust-survivors` — `engine` 크레이트명으로 이 엔진을 사용하는 게임 프로젝트
- 엔진 공개 API의 파괴적 변경 시 game 측 영향 확인 필요

## Markdown 문서 구조
에이전트가 자동 인식해야 하는 지침 파일은 루트에 둔다. 일반 문서는 `docs/`에 모아 관리한다.

| 위치 | 용도 |
| --- | --- |
| `AGENTS.md` | Codex/에이전트 공통 빠른 참조: 모듈 맵, 핵심 패턴, 작업 체크리스트 |
| `CLAUDE.md` | Claude 계열 에이전트용 빠른 참조 |
| `README.md`, `REFERENCE.html`, `docs/HANDOFF.md` | 소개/사용법, 공개 API, Phase별 개발 이력 |
| `docs/CHANGELOG.md`, `docs/ROADMAP.md` | 릴리스 변경 이력, 향후 개발 계획 |
| `docs/AGENT_WORKFLOW.md` | 에이전트 상세 작업 규칙 |
| `docs/REMAINING_WORK.md`, `docs/PARALLEL_TASKS.md` | 로컬 전용 과거 작업 계획(ignored) |
| `docs/ENGINE_REVIEW_FIX_PROMPT.md` | 로컬 전용 엔진 리뷰 수정 프롬프트(ignored) |
| `docs/rust_game_engine_plan.md` | 로컬 전용 초기/상세 개발 계획 문서(ignored) |

> **확장 전략**: 새 서브시스템 문서는 `docs/SUBSYSTEM.md`처럼 `docs/` 아래에 추가하고, 이 파일 모듈 맵에는 한 줄 참조만 추가한다.
