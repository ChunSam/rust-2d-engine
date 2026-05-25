# 핸드오프 문서 — rust-2d-engine

작성일: 2026-05-24 (Phase 36 갱신: 2026-05-25)  
엔진 버전: v0.36.0 (태그: v0.3.0, main 브랜치 기준)  
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
| Phase 16 | 씬 직렬화 + 프리팹 시스템 — Tag, EntityDef, SceneDef, Prefab, spawn_entity_def | `2bfbffa` |
| Phase 17 | 에셋 파이프라인 + 핫 리로딩 — Handle<T>, ImageAsset, AssetServer, App::load_image | `f985118` |
| Phase 18 | egui 인게임 디버그 에디터 — DebugUi, F1 토글, Engine Stats 내장 패널 | `83838a7` |
| Phase 19 | Rhai 스크립팅 — ScriptAsset, ScriptRunner, ScriptingSystem, App::load_script | `861e832` |
| Phase 20 | 애니메이션 블렌딩 — BlendWeight, play_with_crossfade, BlendTree1D, BlendTreeSystem | `d6ff7f9` |
| Phase 21 | Texture Atlas — TextureAtlas, AtlasSprite, AssetServer::load_atlas, App::load_atlas | `b63e9c9` |
| Phase 22 | Reflect 시스템 — Reflect 트레잇, ReflectValue, World::register_reflect/get_reflect, egui Inspector | `90f65e3` |
| Phase 23 | WASM 빌드 지원 — 플랫폼별 deps 분리, cfg-gate, EventLoopExtWebSys, getrandom wasm_js | `b9f4bdb` |
| Phase 24 | WASM 브라우저 실행 — WebGL2 강제, 비동기 GPU init, web-time, 캔버스 크기 수정 | `24e2108` |
| Phase 25-A | WebSocket 네트워킹 — NetworkClient(native tungstenite / WASM web-sys), NetworkEvent, NetworkSystem | `88311e9` |
| Phase 25-B | ECS 병렬 쿼리 — rayon par_query_for_each/map, par_query2_for_each/map, Send+Sync 컴포넌트 스토리지 | `4637ace` |
| Phase 25-C | 커스텀 셰이더 머티리얼 — ShaderMaterial, params uniform, 파이프라인 캐시, 스프라이트 배칭 정리 | `9a7b375` |
| Phase 25-D | 에디터 기즈모 — SelectedEntity 리소스, Inspector 엔티티 생성/삭제, 드래그 이동, DebugRect 강조 | `c19d0b6` |
| Phase 25-E | rust-survivors 연동 — Sprite 필드 대응, EnemyAiSystem par_query2_map 병렬화 (game repo) | — |
| Phase 26 | LOD/컬링 — Camera::visible_rect, CullConfig 리소스, 회전 고려 AABB 프러스텀 컬링, min_pixel_size LOD | `8db9bbe` |
| Phase 27 | 멀티플레이어 데모 — mp_server(릴레이 서버) + mp_client(게임 클라이언트) 예제 | — |
| Phase 28 | 에디터 씬 저장 — Inspector에 "💾 Save Scene" 버튼, SceneDef RON 직렬화 | — |
| Phase 29 | 씬 계층 직렬화 — EntityDef.parent, spawn_scene_def 2패스, topological_sort_entities | — |
| Phase 30 | 시스템 프로파일러 — System::name(), ProfilerData/RenderStats 리소스, Engine Stats 패널 확장 | — |
| Phase 31 | 에셋 브라우저 — ImageEntry, image_list(), Inspector "Assets" 탭 | — |

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
├── asset.rs          Handle<T>, ImageAsset, AssetServer  ← Phase 17
├── prefab.rs         Tag, EntityDef, SceneDef, Prefab, spawn_entity_def, spawn_scene_def  ← Phase 16
├── components.rs     Transform, Sprite (image_handle 추가 ← Phase 17)
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

## 이번 세션에서 한 일 (Phase 24)

### Phase 24 — WASM 브라우저 실행

**배경**: Phase 23에서 `cargo build --target wasm32-unknown-unknown`은 통과했지만 실제 브라우저 실행이 되지 않았다. `wasm-pack build` → HTTP 서버 → Chrome에서 컬러 박스들이 튀어다니는 데모를 동작시키는 것이 목표.

**검증**: `wasm-pack build --target web` 성공, Chrome에서 10개 컬러 박스 ECS + 바운스 물리 동작 확인

**수정된 파일**: `Cargo.toml`, `src/app.rs`, `src/renderer/context.rs`, `src/lib.rs`

**해결된 문제 목록 (순서대로 발생)**

| 문제 | 원인 | 수정 |
|------|------|------|
| `requestDevice` 실패 — `maxInterStageShaderComponents` 미인식 | wgpu 22 `Limits::default()`가 Chrome WebGPU가 인식 못하는 limit 포함 | `context.rs`: WASM에서 `Backends::GL` 강제(WebGL2) + `downlevel_webgl2_defaults()` |
| `std::time::Instant` 패닉 | WASM에서 `std::time::Instant` 미지원 | `Cargo.toml`에 `web-time = "1"` 추가; `app.rs`에 `#[cfg] use web_time::Instant` |
| GPU `async` init 불가 (`unreachable` 패닉) | WebGPU Promise 기반 → 단순 poll 불가 | `thread_local! PENDING_GPU` + `spawn_local` + `about_to_wait`/`RedrawRequested`에서 pick-up |
| `surface: 1x1` | `window.inner_size()`가 canvas attach 직후 1×1 반환 | `context.rs`: WASM에서 `#[game-canvas]` DOM 요소에서 직접 width/height 읽기 |
| `no default font found` 패닉 | WASM에 시스템 폰트 없음 — `cosmic-text` shape 시 패닉 | `finish_init()`: WASM에서 `font_bytes` 비면 `TextRenderer` 생성 생략 |
| `Surface size (2560×1440) > WebGL2 max (2048)` | Retina DPR=2 → winit `Resized` 이벤트가 물리 픽셀 보고 | `app.rs` `Resized` 핸들러: WASM에서 DOM canvas 크기로 대체 |

**핵심 설계 결정**
- WebGPU 대신 WebGL2(`Backends::GL`)를 강제: Chrome 버전별 WebGPU limit 명세 불일치 문제를 회피. `wgpu = { features = ["webgl"] }` 이미 선언되어 있어 추가 의존성 없음
- `PENDING_GPU thread_local`: `spawn_local` future에서 GPU 컨텍스트를 완성 후 저장 → 메인 이벤트 루프(`about_to_wait` + `RedrawRequested`)에서 polling. 두 곳에서 체크해 타이밍 경쟁 방지
- WASM 텍스트: `FontData` 리소스에 TTF 바이트를 직접 주입해야 텍스트 렌더링 가능. 미주입 시 텍스트 렌더러 생략(무패닉)

**WASM 런타임 동작 (업데이트)**

| 기능 | WASM |
|------|------|
| wgpu 렌더링 (WebGL2) | ✅ 동작 |
| ECS, Sprite, 애니메이션 | ✅ 동작 |
| 텍스트 렌더링 | ✅ FontData 리소스 주입 시 동작 (미주입 시 생략) |
| Physics, Audio, Gamepad | 비활성 — `#[cfg(not(wasm))]` |

---

## 이전 세션에서 한 일 (Phase 27–28)

### Phase 27 — 멀티플레이어 데모

**배경**: Phase 25-A에서 NetworkClient/NetworkSystem을 구현했지만 실제 서버-클라이언트 데모가 없었다. 두 예제 바이너리를 `examples/`에 추가해 엔진의 네트워킹 API 사용 패턴을 보여준다.

**mp_server** (`examples/mp_server.rs`):
- `TcpListener::bind("127.0.0.1:9001")`으로 WebSocket 수신
- 클라이언트별 스레드 + `mpsc::Sender<Message>` 브로드캐스트 맵
- 5 ms read timeout 루프로 발신/수신 논블로킹 처리
- 프로토콜: 클라이언트 연결 시 `{"type":"hello","id":N}` 전송, 위치 릴레이 `{"type":"pos","id":N,"x":...,"y":...}`, 퇴장 통보 `{"type":"bye","id":N}`

**mp_client** (`examples/mp_client.rs`):
- `NetworkClient::connect("ws://127.0.0.1:9001")` + `NetworkSystem` 등록
- `MultiplayerSystem`: 로컬 플레이어(흰 사각형) WASD 이동, 20 Hz 위치 송신
- 원격 플레이어: ID별 고유 색상 사각형, pos/bye 수신 시 스폰/디스폰
- HUD: 연결 상태, Player ID, 접속 인원 수 표시

**실행**:
```
cargo run --example mp_server   # 터미널 1
cargo run --example mp_client   # 터미널 2, 3, ...
```

---

### Phase 29 — 씬 계층 직렬화

**배경**: Phase 12에서 `Parent`/`Children`/`GlobalTransform`/`HierarchySystem` 계층 시스템이 완전히 구현됐지만, `EntityDef`/`SceneDef` 직렬화 포맷이 평면 리스트여서 씬 파일 저장/로드 시 계층 관계가 소실됐다.

**변경 파일**: `src/prefab.rs`, `src/app.rs`, `src/lib.rs`

**추가 기능**:
- `EntityDef`에 `parent: Option<String>` 필드 추가 (`#[serde(default, skip_serializing_if = "Option::is_none")]`으로 기존 RON 파일 하위 호환 유지)
- `spawn_scene_def()` 2패스 방식으로 교체: 1패스 엔티티 생성 + tag→Entity 맵, 2패스 `hierarchy::attach()` 호출
- `topological_sort_entities(entities: &[Entity], world: &World) -> Vec<Entity>` 자유 함수 추가 (BFS, 루트→자식 순)
- 에디터 씬 저장 시 `topological_sort_entities()`로 정렬 후 `Parent` 컴포넌트를 읽어 `EntityDef.parent` 채움
- `topological_sort_entities` re-export (`lib.rs`)
- 테스트: `scene_hierarchy_roundtrip`, `topological_sort_roots_before_children` 추가

---

### Phase 30 — 시스템 프로파일러

**배경**: 시스템별 실행 시간과 렌더러 통계(draw call 수, culled 스프라이트 수)를 에디터에서 실시간으로 확인할 수 없었다.

**변경 파일**: `src/ecs/system.rs`, `src/resources.rs`, `src/renderer/sprite.rs`, `src/app.rs`, `src/lib.rs`

**추가 기능**:
- `System` 트레잇에 `fn name(&self) -> &'static str { "" }` default 메서드 추가 (기존 impl System 하위 호환)
- `SystemProfile { name, last_us, avg_us }`, `RenderStats { draw_calls, sprites_rendered, sprites_culled }`, `ProfilerData { systems, render, frame_ms }` 리소스 추가
- `ProfilerData::record_system()` — EMA(α=1/60) 이동 평균 계산
- `App::update()` 시스템 루프를 `Instant` 계측 래퍼로 교체, 결과를 `ProfilerData`에 기록
- `sprite.rs render()` 반환 타입 `RenderStats`로 변경, culling/draw call 카운터 수집
- Engine Stats 패널에 "Systems" / "Render" collapsible 섹션 추가, `resizable(true)`로 변경
- `ProfilerData`, `RenderStats`, `SystemProfile` re-export (`lib.rs`)

---

### Phase 36 — 비헤이비어 트리

**배경**: A* 경로탐색으로 이동 경로를 구할 수 있게 됐지만, 적 AI의 의사결정 구조(추적·공격·대기 전환)를 시스템 코드에 분산해야 했다. 비헤이비어 트리는 AI 로직을 계층적·재사용 가능한 노드 그래프로 선언할 수 있게 한다.

**변경 파일**: `src/behavior.rs` (신규), `src/ecs/world.rs`, `src/lib.rs`

**추가 기능**:

*비헤이비어 트리 (src/behavior.rs)*
- `BehaviorStatus { Running, Success, Failure }` — 노드 실행 결과
- `BehaviorNode` 트레잇: `tick(&mut self, world, entity, dt)` + `reset()` (선택 구현)
- `Sequence`: 자식 순서 실행, 첫 Failure에 즉시 중단 → Failure / 전부 성공 → Success
- `Selector`: 자식 순서 실행, 첫 Success에 즉시 중단 → Success / 전부 실패 → Failure
- `Inverter`: Success ↔ Failure 반전, Running 유지
- `AlwaysSucceed`: 자식 결과 무시하고 항상 Success 반환
- `BehaviorTree` 컴포넌트: 루트 `Box<dyn BehaviorNode>` 래퍼
- `BehaviorSystem`: `take_component → tick → add_component` 패턴으로 이중 borrow 없이 실행
- 테스트 8개

*World::take_component (src/ecs/world.rs)*
- 컴포넌트를 소유권째 꺼내 반환하는 새 API
- placeholder(`Box<()>`) 교체 → `remove_component`로 아키타입 정리 → 이중 해제 없음

```rust
// 커스텀 노드 예
struct ChasePlayer;
impl BehaviorNode for ChasePlayer {
    fn tick(&mut self, world: &mut World, entity: Entity, _dt: f32) -> BehaviorStatus {
        // world에서 플레이어 위치를 읽어 entity를 이동
        BehaviorStatus::Running
    }
}

// 조합
world.add_component(enemy, BehaviorTree::new(Box::new(Selector::new(vec![
    Box::new(Sequence::new(vec![Box::new(CanSeePlayer), Box::new(ChasePlayer)])),
    Box::new(Patrol),
]))));
app.add_system(BehaviorSystem);
```

---

### Phase 35 — Inspector Undo/Redo

**배경**: Inspector에서 엔티티를 잘못 이동·삭제해도 되돌릴 방법이 없어 에디터 워크플로가 불편했다.

**변경 파일**: `src/app.rs` 만 수정.

**추가 기능**:
- `EditorCmd` enum: `MoveEntity { entity, old_pos, new_pos }` / `CreateEntity { entity }` / `DeleteEntity { tag, transform, sprite }`
- `EditorHistory { undo: Vec, redo: Vec }`: `push` / `undo` / `redo` 메서드
- **Gizmo 드래그 완료** 시 `MoveEntity` 기록 (위치 변화 없으면 기록 않음)
- **New Entity 버튼** → `CreateEntity` 기록
- **Delete 버튼** → 스냅샷(tag/transform/sprite) 캡처 후 `DeleteEntity` 기록
- **Ctrl+Z** → undo, **Ctrl+Shift+Z** → redo (egui `ctx.input()` 기반)
- 네이티브 전용 (`#[cfg(not(target_arch = "wasm32"))]`)

---

### Phase 34 — RenderLayer + 스프라이트 배칭

**배경**: 수백 개의 동일 텍스처 스프라이트가 있어도, z 정렬 후 다른 텍스처와 교차되면 draw call이 하나씩 발생했다. 레이어 분리가 없어 배경·게임오브젝트·전경의 렌더 순서를 보장할 방법도 없었다.

**변경 파일**: `src/components.rs`, `src/renderer/sprite.rs`, `src/lib.rs`

**추가 기능**:
- `RenderLayer(i32)` 컴포넌트 추가 (선택, 미지정 시 0)
  - 낮은 값이 먼저(뒤에) 그려짐. 배경=-1, 게임플레이=0, 전경/이펙트=1 권장
- 스프라이트 정렬 키: `z` 단독 → `(layer, tex_key, z)` 변경
  - 같은 `(layer, tex_key)` 는 항상 연속 → 텍스처당 draw call 1회 보장
  - 다른 layer 간 렌더 순서 항상 보장
- `AtlasSprite`도 동일하게 `RenderLayer` 읽기 적용

**트레이드오프**: 같은 layer 내 서로 다른 텍스처 간 z-ordering은 텍스처 키 사전순으로 결정된다. 정확한 교차 z-ordering이 필요하면 `RenderLayer`로 분리한다.

---

### Phase 33 — A* 경로 탐색 + ECS 쿼리 필터

**배경**: 적 AI가 장애물을 피해 이동할 수단이 없었고, ECS 쿼리에서 "특정 컴포넌트가 있는/없는 엔티티만"을 표현하는 방법이 없어 시스템 내부에서 수동으로 필터링해야 했다. 두 기능은 파일 충돌 없이 병렬 구현 가능해 서브 에이전트 2개로 동시 진행했다.

**변경 파일**: `src/pathfinding.rs` (신규), `src/ecs/world.rs`, `src/lib.rs`

**A* 경로 탐색 (src/pathfinding.rs)**

- `PathGrid { width, height, cells: Vec<bool> }` — row-major 격자
  - `new(w, h)` — 전부 통행 가능
  - `new_blocked(w, h)` — 전부 막힘
  - `set_walkable(x, y, bool)` / `is_walkable(x, y) -> bool` (범위 밖 = false)
- `find_path(grid, start: IVec2, goal: IVec2) -> Option<Vec<IVec2>>`
  - 4방향 이동, 맨해튼 휴리스틱, `BinaryHeap` min-heap (역순 `Ord`)
  - 반환 경로에 start 미포함, goal 포함
  - `start == goal` → `Some(vec![goal])`, 경로 없음 → `None`
  - 목표가 막혀 있으면 즉시 `None` (open set 탐색 없이)
- 테스트 4개: 직선·우회·막힘·동일점

**ECS 쿼리 필터 (src/ecs/world.rs)**

- `World::query_with::<A, B>()` — A와 B를 **모두** 가진 엔티티만 `(Entity, &A)` 반환
- `World::query_without::<A, B>()` — A는 있고 B가 **없는** 엔티티만 `(Entity, &A)` 반환
- 구현: 아키타입 레벨에서 `TypeId` 포함 여부(`arch.contains(tb)`)를 판단. per-entity `get::<B>()` 호출보다 효율적이며 기존 `query2` 패턴과 일치함.
- 마커 타입(`With<T>`, `Without<T>`) 미생성 — 불필요한 추상화 배제
- 테스트 3개 추가 (총 16개): With 필터, Without 필터, 혼합 4-조합 케이스

**사용 예**:
```rust
// Sprite가 있는 Transform만 처리
for (e, t) in world.query_with::<Transform, Sprite>() { ... }

// Enemy 없는 엔티티만 (NPC 처리 등)
for (e, t) in world.query_without::<Transform, Enemy>() { ... }

// A* 경로 탐색
let mut grid = PathGrid::new(20, 15);
grid.set_walkable(5, 3, false); // 장애물
if let Some(path) = find_path(&grid, IVec2::new(0, 0), IVec2::new(19, 14)) {
    // path: [IVec2, ...] goal 포함, start 미포함
}
```

---

### Phase 32 — 런타임 안정성

**배경**: 이미지 로드 실패 시 마젠타 폴백이 있었지만 어떤 핸들이 실패했는지 알 수 없었다. `SceneDef` RON 포맷에 버전 정보가 없어 향후 구조 변경 시 구 파일을 감지할 수단이 없었다. Inspector에 Save는 있지만 Load가 없어 에디터 워크플로가 불완전했다.

**변경 파일**: `src/asset.rs`, `src/prefab.rs`, `src/app.rs`, `src/lib.rs`

**추가 기능**:

*AssetLoadState (src/asset.rs)*
- `AssetLoadState { Loaded, Failed(String) }` enum 추가
- `AssetServer`에 `image_load_states: HashMap<AssetId, AssetLoadState>` 필드 추가
- `load_image()` 내부에서 `decode_image_with_state()` 호출 — 성공/실패 상태를 함께 기록
- 핫 리로딩(`poll_reloads`)도 리로드 후 상태 갱신
- `AssetServer::load_state(&Handle<ImageAsset>) -> AssetLoadState` 공개 API
- `AssetServer::failed_images() -> Vec<AssetId>` — 실패 핸들 목록 (디버그용)
- `AssetLoadState` re-export (`lib.rs`)

*SceneDef 스키마 버전 (src/prefab.rs)*
- `SCENE_DEF_VERSION: u32 = 1` 상수 추가
- `SceneDef`에 `#[serde(default)] pub version: u32` 필드 추가 — 구 파일(version 없음)은 0으로 역직렬화되어 하위 호환 유지
- `Default` 구현을 수동으로 변경: `version: SCENE_DEF_VERSION`으로 초기화
- `SceneDef::load()` — 역직렬화 후 버전 불일치 시 `log::warn` 출력, 로드는 계속
- `SceneDef::save()` — 항상 `version: SCENE_DEF_VERSION`으로 덮어써서 저장
- `SCENE_DEF_VERSION` re-export (`lib.rs`)

*Inspector Load Scene (src/app.rs)*
- `App`에 `editor_load_status: Option<String>` 필드 추가
- Inspector 씬 저장 행에 `📂 Load Scene` 버튼 추가 (Save 버튼 왼쪽)
- 클릭 시: RON 로드 → `Transform`을 가진 기존 엔티티 전부 despawn → `spawn_scene_def` 호출
- 성공/실패 메시지를 `editor_load_status`에 저장해 패널에 표시
- `inspector_selected` 초기화 (로드 후 선택 상태 리셋)

**아키텍처 결정**:
- Load Scene 시 `Transform` 보유 엔티티만 제거한다. 물리 바디, 카메라 등 시스템 엔티티는 건드리지 않아 충돌을 피한다.
- `AssetLoadState::Failed` 내부에 오류 문자열을 포함해 `log::error` 없이도 원인 추적이 가능하다.

---

### Phase 31 — 에셋 브라우저

**배경**: 현재 로드된 이미지 에셋 목록을 에디터에서 확인할 수 없었고, `AssetServer`의 내부 `path_to_id` 맵이 private이어서 외부에서 조회 불가였다.

**변경 파일**: `src/asset.rs`, `src/app.rs`, `src/lib.rs`

**추가 기능**:
- `ImageEntry { path, id, width, height }` 구조체 추가
- `AssetServer::image_list() -> Vec<ImageEntry>` — 현재 로드된 이미지 목록 반환
- `AssetServer::get_image_by_id(id: AssetId) -> Option<&ImageAsset>` 추가
- `App` 구조체에 `inspector_tab: u8` (0=Entities, 1=Assets) 필드 추가
- Inspector 패널 상단에 탭 버튼 추가, "Assets" 탭에서 파일명·해상도 그리드 표시
- `ImageEntry` re-export (`lib.rs`)

---

### Phase 28 — 에디터 씬 저장

**배경**: Phase 25-D에서 기즈모로 엔티티를 배치할 수 있게 됐지만, 배치 결과를 파일로 저장하는 수단이 없었다.

**변경 파일**: `src/app.rs` 만 수정.

**추가 기능**:
- `App` 구조체에 `editor_save_path: String`, `editor_save_status: Option<String>` 필드 추가
- Inspector 패널 하단에 "Path:" 텍스트 입력 + `💾 Save Scene` 버튼 추가 (`#[cfg(not(target_arch = "wasm32"))]` 게이트)
- 버튼 클릭 시 현재 월드의 모든 엔티티를 순회해 `Tag`/`Transform`/`Sprite`가 있는 엔티티를 `EntityDef`로 수집 → `SceneDef::save()` 호출
- 결과 메시지 (예: `✓ 5 entities → saved_scene.ron`) 패널 하단에 표시
- `reload_scene()` 시 저장 상태 메시지 초기화

---

## 이전 세션에서 한 일 (Phase 23)

### Phase 23 — WASM 빌드 지원

**배경**: `rapier2d`, `rodio`, `gilrs`, `notify`는 OS 스레드/파일/HID API를 사용해 `wasm32-unknown-unknown` 타겟에서 컴파일되지 않는다. 이들을 플랫폼별 의존성으로 분리하고, 관련 코드를 `cfg`로 게이팅해 WASM 빌드를 통과시킨다.

**검증**: `cargo build --target wasm32-unknown-unknown` — 경고 없음, 오류 없음

**수정된 파일**: `Cargo.toml`, `.cargo/config.toml` (신규), `src/lib.rs`, `src/app.rs`, `src/asset.rs`, `src/save.rs`, `src/input/gamepad.rs`

**추가된 파일**: `examples/wasm/index.html`, `examples/wasm/build.sh`, `.cargo/config.toml`

**Cargo.toml 변경**

| 분류 | 이전 | 이후 |
|------|------|------|
| `wgpu` | `"22"` | `{ version = "22", features = ["webgl"] }` |
| `rapier2d`, `rodio`, `gilrs`, `notify`, `dirs` | `[dependencies]` | `[target.'cfg(not(wasm))'.dependencies]` |
| `wasm-bindgen`, `wasm-bindgen-futures`, `web-sys`, `console_error_panic_hook` | 없음 | `[target.'cfg(wasm)'.dependencies]` |

**getrandom 충돌 해결** (`getrandom 0.2` + `0.3` 동시 사용)
- `getrandom 0.2` — `rand 0.8`용, `js` feature
- `getrandom 0.3` — wgpu 등 전이 의존성, `wasm_js` feature (알리아스 `getrandom3`)
- `.cargo/config.toml` — `--cfg getrandom_backend="wasm_js"` RUSTFLAGS 설정

**cfg-gate 목록**

| 파일 | 변경 내용 |
|------|-----------|
| `src/lib.rs` | `pub mod physics`, `pub mod audio` + 관련 re-export 조건부 컴파일 |
| `src/lib.rs` | `#[wasm_bindgen(start)]` — `console_error_panic_hook` 초기화 |
| `src/app.rs` | `gilrs: Option<gilrs::Gilrs>` 필드 + `poll_gilrs()` + `gilrs::Gilrs::new()` |
| `src/app.rs` | `run()` — WASM: `EventLoopExtWebSys::spawn_app(self)` |
| `src/app.rs` | `resumed()` — WASM: 수동 단일-poll executor (webgl 동기 완료 활용) |
| `src/asset.rs` | `use notify::...` + `_watcher: Option<RecommendedWatcher>` + 감시 설정 |
| `src/save.rs` | `save_path()` — WASM: `dirs` 없이 상대 경로 반환 |
| `src/input/gamepad.rs` | `id_map`, `Slot::new`, `process_event`, `slot_mut`, `map_button`, `map_axis` |

**WASM 런타임 동작**

| 기능 | WASM |
|------|------|
| wgpu 렌더링 (WebGL2) | 동작 |
| ECS, UI, 애니메이션, 타일맵 | 동작 |
| Physics (rapier2d) | 비활성 — `#[cfg(not(wasm))]` |
| Audio (rodio) | 비활성 — `#[cfg(not(wasm))]` |
| Gamepad (gilrs) | 비활성 — `#[cfg(not(wasm))]` |
| 파일시스템 에셋 로드 | 런타임 오류 (std::fs 미지원) |
| 핫 리로딩 | 비활성 — notify 없음 |

**브라우저 실행 방법**
```bash
# 의존성: cargo install wasm-pack
cd /path/to/rust-2d-engine
bash examples/wasm/build.sh
python3 -m http.server 8080 --directory examples/wasm
# 브라우저에서 http://localhost:8080 열기
```

**핵심 설계 결정**
- `physics`와 `audio` 모듈 전체를 lib.rs에서 `#[cfg(not(wasm))]`으로 게이팅 → 해당 파일들 자체는 수정 불필요
- WASM용 GPU 초기화: wgpu webgl 백엔드는 adapter 요청이 첫 poll에서 즉시 완료(동기) → `pollster::block_on` 없이 단순 수동 poll로 동작
- `GamepadState` 구조체는 WASM에서도 존재하지만 gilrs 타입(`GamepadId`) 의존 필드/메서드 제거 → 빈 상태로 컴파일 가능

---

## 이전 세션에서 한 일 (Phase 22)

### Phase 22 — Reflect 시스템

**배경**: egui 인스펙터에서 컴포넌트 속성을 이름으로 읽고 쓸 수 있는 런타임 필드 접근 API가 필요했다. proc-macro 없이 핵심 컴포넌트에 수동 구현해 복잡도를 낮췄다.

**추가된 파일**: `src/reflect.rs`

**수정된 파일**: `src/components.rs`, `src/prefab.rs`, `src/ecs/world.rs`, `src/app.rs`, `src/lib.rs`

**새 타입**
- `ReflectValue` (`src/reflect.rs`) — `F32 | Vec2 | Bool | String | Color([f32;4])` 열거형
- `Reflect` 트레잇 (`src/reflect.rs`) — `fields()`, `set_field()`, `type_name()` 인터페이스
- `ReflectEntry` (`src/ecs/world.rs`) — `Copy` 가능한 함수 포인터 쌍 (`get`, `get_mut`)

**컴포넌트 구현**
- `Transform` — x, y, rotation, scale_x, scale_y, z (모두 F32)
- `Sprite` — color (Color), texture (String)
- `Tag` — tag (String)

**World 확장** (`src/ecs/world.rs`)
- `reflect_registry: HashMap<TypeId, ReflectEntry>` 필드 추가
- `register_reflect::<T>()` — TypeId → 함수 포인터 등록
- `get_reflect(entity, TypeId)` → `Option<&dyn Reflect>`
- `get_reflect_mut(entity, TypeId)` → `Option<&mut dyn Reflect>`
- `reflected_components(entity)` → `Vec<TypeId>` (등록된 컴포넌트 중 보유 목록)
- `is_alive(entity)` → `bool`

**egui Inspector 패널** (`src/app.rs`)
- F1 Debug UI 내 `Inspector` 창 추가 (기본 위치: [10, 130])
- 좌측: 엔티티 목록 (Tag 있으면 Tag명, 없으면 "Entity N" 표시, 클릭으로 선택)
- 우측: 선택된 엔티티의 컴포넌트별 collapsing 패널 + Grid 레이아웃 필드 편집기
  - F32 → `DragValue` (슬라이더 속도 0.5)
  - Color → `color_edit_button_rgba_unmultiplied`
  - String → `text_edit_singleline`
- 편집은 "stage-and-apply" 패턴: 읽기(불변) → egui 수정 → 쓰기(가변) — borrow 충돌 없음

**자동 등록**: `App::new()` + `App::reload_scene()`에서 Transform, Sprite, Tag 자동 `register_reflect`

**핵심 설계 결정**
- `ReflectEntry`가 `Copy`인 이유: 함수 포인터를 담아 `let entry = *map.get()?` 로 복사 후 `&mut self.archetypes` borrow 가능
- object-safe 유지: `Reflect` 트레잇에 제네릭·Self 없음 → `dyn Reflect` 사용 가능
- `Vec2`, `Bool` ReflectValue 열거형에 포함 — 사용자 컴포넌트 확장을 위해 미리 준비

**사용 패턴**
```rust
// 수동 등록 (App::new()에서 자동 등록되지 않는 사용자 컴포넌트)
world.register_reflect::<MyComp>();

// 읽기
if let Some(refl) = world.get_reflect(entity, TypeId::of::<Transform>()) {
    for (name, val) in refl.fields() { println!("{name}: {val:?}"); }
}

// 쓰기
if let Some(refl) = world.get_reflect_mut(entity, TypeId::of::<Transform>()) {
    refl.set_field("x", ReflectValue::F32(100.0));
}

// egui Inspector — F1 키로 토글, 별도 코드 불필요 (App 내장)
```

---

## 이전 세션에서 한 일 (Phase 21)

### Phase 21 — Texture Atlas 시스템

**배경**: 렌더러는 이미 GPU 인스턴싱을 사용하지만, 텍스처별 드로우콜이 발생한다. 여러 스프라이트를 하나의 아틀라스 텍스처로 묶으면 드로우콜을 최소화할 수 있다.

**추가된 파일**: `src/atlas.rs`

**수정된 파일**: `src/asset.rs`, `src/renderer/sprite.rs`, `src/app.rs`, `src/lib.rs`

**새 타입**
- `TextureAtlas` (`src/atlas.rs`) — 이미지 핸들 + cols/rows 그리드 정보. `uv_rect(index)` → `UvRect` 계산
- `AtlasSprite` (`src/atlas.rs`) — `Handle<TextureAtlas>` + index + color. Transform과 함께 사용

**AssetServer 확장** (`src/asset.rs`)
- `atlases: HashMap<AssetId, TextureAtlas>` + `atlas_path_to_id` 추가
- `load_atlas(path, cols, rows) → Handle<TextureAtlas>` — 같은 경로 재호출 시 캐시 반환
- `get_atlas(handle) → Option<&TextureAtlas>` — 렌더러 내부에서 UV 계산에 사용

**App 확장** (`src/app.rs`)
- `load_atlas(path, cols, rows) → Handle<TextureAtlas>` — `pending_textures`에 추가해 GPU 텍스처도 로드

**렌더러 확장** (`src/renderer/sprite.rs`)
- `AtlasSprite` 쿼리 → `AssetServer::get_atlas()` → `uv_rect()` → 기존 `sprites` Vec에 추가
- 기존 Sprite와 동일한 z-sort + texture-group 드로우콜 흐름 그대로 사용 (하위 호환 유지)

**사용 패턴**
```rust
// 4×4 그리드 아틀라스 로드
let atlas = app.load_atlas("assets/characters.png", 4, 4);

// 엔티티 생성
let e = world.spawn();
world.add_component(e, Transform::default());
world.add_component(e, AtlasSprite::new(atlas.clone(), 5)); // index 5번 타일

// index 변경 (애니메이션)
world.get_mut::<AtlasSprite>(e).unwrap().index = 6;
```

**핵심 설계 결정**
- 같은 아틀라스 텍스처를 사용하는 `AtlasSprite` 엔티티들은 z-sort 후 연속 배치 시 **1개 드로우콜**
- 아틀라스 이미지 경로가 텍스처 캐시 키이므로 기존 `Sprite(texture: path)` 경로와 공유 가능
- `atlases` map은 `path → AtlasId` 단방향 캐시 — 같은 경로로 다른 cols/rows 호출 시 첫 번째 설정 사용

---

## 이전 세션에서 한 일 (Phase 20)

### Phase 20 — 애니메이션 블렌딩

**배경**: `AnimationPlayer`는 클립 간 즉시 전환만 지원했다. 크로스페이드와 파라미터 기반 클립 선택을 추가해 부드러운 애니메이션 전환을 가능하게 한다.

**추가된 파일**: `src/animation/blend_tree.rs`, `src/animation/blend_system.rs`  
**변경된 파일**: `src/animation/player.rs`, `src/animation/system.rs`, `src/animation/mod.rs`, `src/lib.rs`

#### 주요 타입

| 타입 | 역할 |
|------|------|
| `BlendWeight` | 크로스페이드 진행도(0.0→1.0) 컴포넌트. `AnimationSystem`이 매 프레임 갱신 |
| `BlendTree1D` | float 파라미터 → 자동 클립 선택 + 크로스페이드 컴포넌트 |
| `BlendEntry` | BlendTree1D의 항목 (threshold, clip_index) |
| `BlendTreeSystem` | BlendTree1D를 읽어 AnimationPlayer에 클립 전환을 지시하는 시스템 |

#### 크로스페이드 API

```rust
// 즉시 전환 (기존)
player.play(clip_index);

// 0.2초 크로스페이드 전환 (신규)
player.play_with_crossfade(clip_index, 0.2);

// 전환 진행도 읽기 (0.0 = from 클립, 1.0 = to 클립 / 전환 없으면 1.0)
let w = player.blend_weight();

// 전환 중 여부
let crossfading = player.is_crossfading();

// BlendWeight 컴포넌트로도 읽을 수 있다 (AnimationSystem이 자동 갱신)
if let Some(bw) = world.get_mut::<BlendWeight>(entity) {
    sprite.alpha = bw.0;  // 알파 보간 예시
}
```

#### 1D 블렌드 트리 API

```rust
// 트리 구성 (threshold 오름차순)
let tree = BlendTree1D::new(
    vec![
        BlendEntry { threshold: 0.0, clip_index: 0 },  // idle
        BlendEntry { threshold: 0.3, clip_index: 1 },  // walk
        BlendEntry { threshold: 1.2, clip_index: 2 },  // run
    ],
    0.15,  // 클립 전환 시 크로스페이드 0.15초
);
world.add_component(entity, tree);

// 매 프레임 파라미터 갱신 (예: speed)
world.get_mut::<BlendTree1D>(entity).unwrap().set_param(speed);
```

#### 등록 순서

```rust
app.add_system(Box::new(BlendTreeSystem));   // 클립 선택
app.add_system(Box::new(AnimationSystem));   // 프레임 진행 + BlendWeight 갱신
app.add_system(Box::new(StateMachineSystem)); // 상태 머신 (기존)
```

#### 크로스페이드 동작 원리

| 진행도 | 출력 UV | 설명 |
|--------|---------|------|
| 0.0 ~ 0.5 미만 | from_clip 현재 프레임 | 이전 클립 계속 표시 |
| 0.5 이상 ~ 1.0 | to_clip 현재 프레임 | 새 클립으로 전환 |
| 완료(elapsed ≥ duration) | to_clip 프레임 | crossfade 해제, 정상 재생 |

두 클립 모두 진행도와 무관하게 계속 진행되므로, UV 전환 시점에 to_clip이 자연스럽게 앞서 재생된 상태다.

---

## 이전 세션에서 한 일 (Phase 19)

### Phase 19 — Rhai 스크립팅

**배경**: 게임 로직을 Rust 재컴파일 없이 `.rhai` 스크립트로 작성할 수 있게 한다. 각 엔티티에 `ScriptRunner`를 붙이면 매 프레임 `on_update(dt)`가 실행되고, Transform이 자동 동기화된다.

**추가된 파일**: `src/scripting.rs`  
**변경된 파일**: `Cargo.toml`, `src/asset.rs`, `src/app.rs`, `src/lib.rs`

#### 주요 타입

| 타입 | 역할 |
|------|------|
| `ScriptAsset` | CPU-side Rhai AST + 소스 문자열 (AssetServer 관리) |
| `ScriptRunner` | 엔티티 컴포넌트; 스크립트 핸들 + Scope 보유 |
| `ScriptingSystem` | 매 프레임 `on_update(dt)` 실행 + Transform 동기화 |

#### 공개 API

```rust
// 스크립트 로드
let handle = app.load_script("assets/enemy_ai.rhai");

// 엔티티에 부착
world.add_component(entity, ScriptRunner::new(handle));

// 시스템 등록
app.add_system(Box::new(ScriptingSystem::new()));
```

**스크립트 예시 (`enemy_ai.rhai`)**:
```rhai
fn on_start() {
    log("AI 초기화");
}

fn on_update(dt) {
    x += 100.0 * dt;   // 오른쪽 이동
    rot += 2.0 * dt;   // 회전
}
```

#### 스코프 변수 (읽기/쓰기)

| 변수 | 타입 | 설명 |
|------|------|------|
| `x`, `y` | `f64` | Transform.position |
| `rot` | `f64` | Transform.rotation (라디안) |
| `sx`, `sy` | `f64` | Transform.scale |

#### 등록된 함수

| 함수 | 설명 |
|------|------|
| `log(msg)` | 디버그 출력 (`[Script] msg`) |

#### 설계 결정

- `ScriptingSystem`이 `Engine`을 직접 소유 — `ScriptEngine` 리소스 없이 간단하게 유지
- `on_start` / `on_update` 없어도 오류 없이 무시 (`EvalAltResult::ErrorFunctionNotFound` 처리)
- 핫 리로딩: `poll_reloads`가 `.rhai` 파일 변경 감지 시 AST 재컴파일. `runner.reset()` 호출 시 `on_start` 재실행
- `max_operations = 1_000_000` 제한으로 스크립트 무한 루프 방지
- `rhai = { features = ["sync"] }` — Engine을 `Send+Sync`로 만들어 향후 멀티스레드 확장 지원

#### Cargo.toml 변경

```toml
rhai = { version = "1", features = ["sync"] }
```

---

## 이전 세션에서 한 일 (Phase 18)

### Phase 18 — egui 인게임 디버그 에디터

**배경**: 개발 중 엔티티/컴포넌트 상태를 실시간으로 확인할 수 없었다. egui를 통합해 인게임 오버레이 패널을 `System` 안에서 자유롭게 추가할 수 있게 한다.

**추가된 파일**: `src/debug_ui.rs`  
**변경된 파일**: `Cargo.toml`, `src/app.rs`, `src/lib.rs`, `src/ecs/world.rs`, `src/asset.rs`

#### 주요 타입

| 타입 | 역할 |
|------|------|
| `DebugUi` | ECS Resource; egui Context 보유, enabled 토글 |

#### 공개 API

```rust
// System 안에서 자유롭게 egui 윈도우 추가
let debug = world.resource::<DebugUi>().unwrap();
if debug.is_enabled() {
    egui::Window::new("My Panel").show(debug.ctx(), |ui| {
        ui.label("Hello!");
    });
}

// F1 키 → 자동 토글 (별도 코드 불필요)
// 내장 패널: "Engine Stats" — FPS / ms / 엔티티 수 / 에셋 수
```

#### 렌더 아키텍처

- 씬 → (포스트프로세스) → **egui 오버레이** → present
- egui는 별도 `CommandEncoder`로 렌더해 씬 인코더와 lifetime 분리
- egui-wgpu 0.29의 `PaintCallbackFn`이 `&mut RenderPass<'static>`을 요구하는 설계 제약 때문에 `egui_render_pass()` 헬퍼 함수에서 `unsafe transmute` 사용 (paint callback 미등록 상태에서 안전)

#### Cargo.toml 변경

```toml
egui = "0.29"
egui-wgpu = "0.29"   # wgpu 22 호환
egui-winit = { version = "0.29", default-features = false }  # clipboard 제외 (macOS objc2 충돌)
```

#### 설계 결정

- `egui-winit`의 clipboard 기능 비활성화: macOS에서 `objc2-app-kit 0.3.2`와 버전 충돌 발생
- F1 토글은 egui_state 이벤트 처리 전에 InputState와 별도로 처리
- `DebugUi::ctx()`는 `begin_pass`/`end_pass` 사이에서만 유효; 엔진이 update() 에서 자동 관리

---

## 이번 세션에서 한 일 (Phase 17)

### Phase 17 — 에셋 파이프라인 + 핫 리로딩

**배경**: 텍스처를 문자열 경로 대신 타입 안전한 `Handle<T>`로 참조하고, 런타임 중 파일이 변경되면 자동으로 GPU 텍스처를 재업로드한다.

**추가된 파일**: `src/asset.rs`  
**변경된 파일**: `Cargo.toml`, `src/components.rs`, `src/lib.rs`, `src/app.rs`, `src/renderer/sprite.rs`, `src/particle.rs`

#### 주요 타입

| 타입 | 역할 |
|------|------|
| `AssetId` | `u64` 전역 단조 증가 ID |
| `Handle<T>` | 타입 지정 에셋 참조 (Clone O(1), id + Arc<str> 경로 보유) |
| `ImageAsset` | CPU-side RGBA8 이미지 데이터 (Arc<Vec<u8>> + 크기) |
| `AssetServer` | 에셋 로드·캐싱·파일 감시·핫 리로딩 리소스 |

#### 공개 API

```rust
// App 레벨 편의 메서드
let handle: Handle<ImageAsset> = app.load_image("assets/player.png");

// 직접 AssetServer 사용
let as_ = world.resource_mut::<AssetServer>().unwrap();
let handle = as_.load_image("assets/bg.png");
let image: Option<&ImageAsset> = as_.get_image(&handle);

// Sprite에 핸들 지정 (texture 경로보다 우선 적용)
Sprite::with_handle(handle)
```

#### Sprite Breaking Change

`Sprite` 구조체에 `image_handle: Option<Handle<ImageAsset>>` 필드 추가.

- `#[serde(skip)]` — RON 직렬화 무영향 (기존 씬 파일 그대로 사용 가능)
- 리터럴 `Sprite { texture: None, color: ... }` 초기화 코드는 `image_handle: None` 추가 필요
- `Sprite::colored()`, `Sprite::textured()`, `Sprite::with_handle()` 생성자는 모두 안전

#### 핫 리로딩 동작

1. `App::new()` 시 `AssetServer::new()` 생성 → World 리소스로 삽입
2. `notify::recommended_watcher`가 백그라운드 스레드에서 파일 변경 감시
3. `App::update()` 매 프레임: `AssetServer::poll_reloads()` → 변경 경로 수신
4. `SpriteRenderer::reload_texture(path)` 호출 → GPU 텍스처 갱신

#### Cargo.toml 변경

- `notify = "6"` — 크로스 플랫폼 파일 감시 (macOS FSEvents, Linux inotify, Windows ReadDirectoryChanges)

#### 설계 결정

- **Handle에 경로 내재**: `Handle<T>`이 `Arc<str>` 경로를 보유해 렌더러가 AssetServer 없이 GPU 텍스처를 조회 가능.
- **기존 `texture` 경로와 공존**: `image_handle`이 있으면 우선 적용, 없으면 `texture` 문자열 경로를 그대로 사용 — 기존 코드 마이그레이션 불필요.
- **파일 감시 실패 시 graceful degradation**: `notify` 초기화 실패(샌드박스 등)해도 로드·캐싱은 정상 동작, 핫 리로딩만 비활성.

---

## 이전 세션 (Phase 16)

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
| ~~Phase 17~~ | ~~에셋 파이프라인 + 핫 리로딩~~ | — | 완료 |
| ~~Phase 18~~ | ~~egui 인게임 디버그 에디터~~ | — | 완료 |
| ~~Phase 19~~ | ~~Rhai 스크립팅 — ScriptAsset/ScriptRunner/ScriptingSystem~~ | — | 완료 |
| ~~Phase 20~~ | ~~애니메이션 블렌딩 — BlendWeight/play_with_crossfade/BlendTree1D~~ | — | 완료 |
| ~~Phase 21~~ | ~~Texture Atlas — TextureAtlas/AtlasSprite/load_atlas~~ | — | 완료 |
| ~~Phase 22~~ | ~~Reflect 시스템 — Reflect 트레잇, ReflectValue, World::register_reflect/get_reflect~~ | — | 완료 |
| ~~Phase 23~~ | ~~WASM 빌드 지원 — cfg-gate 4개 의존성, fs 추상화, 진입점 분기~~ | — | 완료 |
| ~~Phase 24~~ | ~~WASM 브라우저 실행 — WebGL2 강제, 비동기 GPU init, web-time~~ | — | 완료 |
| ~~Phase 25~~ | ~~네트워킹 / ECS 병렬 / 셰이더 머티리얼 / 에디터 기즈모 / 연동~~ | — | 완료 |
| ~~Phase 26~~ | ~~LOD / 컬링 — Camera::visible_rect, CullConfig, AABB 프러스텀 컬링, min_pixel_size LOD~~ | — | 완료 |
| ~~Phase 27~~ | ~~멀티플레이어 데모 — NetworkClient 기반 서버-클라 롤플레잉 예제~~ | — | 완료 |
| ~~Phase 28~~ | ~~에디터 씬 저장 — 기즈모로 배치한 엔티티를 SceneDef RON으로 직렬화~~ | — | 완료 |
| ~~Phase 29~~ | ~~씬 계층 직렬화 — EntityDef.parent, 2패스 스폰, topological_sort_entities~~ | — | 완료 |
| ~~Phase 30~~ | ~~시스템 프로파일러 — System::name(), ProfilerData, RenderStats, Engine Stats 확장~~ | — | 완료 |
| ~~Phase 31~~ | ~~에셋 브라우저 — ImageEntry, image_list(), Inspector Assets 탭~~ | — | 완료 |
| ~~Phase 32~~ | ~~런타임 안정성 — AssetLoadState, SceneDef.version, Load Scene 버튼~~ | — | 완료 |
| ~~Phase 33~~ | ~~A* 경로 탐색 (PathGrid/find_path) + ECS 쿼리 필터 (query_with/query_without)~~ | — | 완료 |
| ~~Phase 34~~ | ~~RenderLayer 컴포넌트 + 스프라이트 배칭 (layer·tex·z 정렬)~~ | — | 완료 |
| ~~Phase 35~~ | ~~Inspector Undo/Redo (Ctrl+Z/Shift+Z) — 이동·생성·삭제~~ | — | 완료 |
| ~~Phase 36~~ | ~~비헤이비어 트리 — BehaviorTree/BehaviorSystem, Sequence/Selector/Inverter~~ | — | 완료 |

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
