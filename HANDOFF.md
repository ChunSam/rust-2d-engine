# 핸드오프 문서 — rust-2d-engine

작성일: 2026-05-24  
엔진 버전: v0.11.0 (태그: v0.3.0, main 브랜치 기준)  
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
| Phase 9 | ECS Archetype 스토리지 — TypeId HashMap+Vec → Archetype 밀집 컬럼 스토리지 | (이번 세션) |
| Phase 10 | 포스트프로세싱 — 비네팅, 색수차, 근사 블룸 (PostProcessConfig 리소스) | (이번 세션) |
| Phase 11 | 오디오 강화 — 위치 오디오, 버스 믹서, 페이드인/아웃 | (이번 세션) |

---

## 현재 구조

```
src/
├── app.rs            엔진 진입점 (winit ApplicationHandler)
├── ecs/
│   ├── world.rs      Entity/Component/Resource 저장소, query1~4, query_opt2
│   ├── events.rs     Events<E> 프레임 경계 이벤트 버스
│   └── system.rs     System 트레잇
├── scene.rs          Scene 트레잇, SceneCmd, SceneChange
├── components.rs     Transform, Sprite
├── resources.rs      WindowConfig, ViewportSize, GameState, ShouldQuit, ...
├── camera.rs         Camera (position, zoom, screen_to_world)
├── input/
│   ├── state.rs      InputState (키보드, 마우스, 스크롤, 문자 입력 버퍼)
│   └── map.rs        InputMap (키 리바인딩)
├── physics/
│   ├── world.rs      PhysicsWorld (Rapier2D 래퍼)
│   ├── body.rs       PhysicsBody 컴포넌트
│   ├── events.rs     CollisionEvent (Started/Stopped)              ← Phase 7
│   └── system.rs     PhysicsSystem
├── collision/
│   ├── grid.rs       SpatialGrid, CollisionGridSystem
│   ├── query.rs      Collider, CollisionLayer
│   └── debug.rs      CollisionDebugSystem, DebugConfig
├── audio.rs          AudioManager (재생/정지/볼륨/팬/톤)
├── animation/
│   ├── player.rs     AnimationPlayer, AnimationClip, UvRect
│   └── system.rs     AnimationSystem
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
│   └── system.rs     UiSystem, UiEvent (5종)
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

Phase 9 이후 계획은 미정. 사용자와 협의 필요. 아래는 가능한 방향:

- **오디오 강화**: 스트리밍 재생, 3D 위치 오디오, 오디오 버스 믹서
- **ECS 성능**: Archetype 기반 스토리지로 교체 (현재는 TypeId HashMap + Vec)
- **셰이더/포스트프로세싱**: wgpu 파이프라인 확장 (블룸, 색수차 등)

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
