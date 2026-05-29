# Handoff document — skeleton-engine

Written: 2026-05-24 (Phase 45~53 update: 2026-05-26 / Phase 46~59 complete: 2026-05-26 / 7 code-review fixes: 2026-05-26 / doc cleanup: 2026-05-29)
Engine version: **v1.0.0** (tag: v1.0.0, based on main branch)
Package: **skeleton-engine** (library crate: `engine`)
Author: ChunSam

---

## Project overview

A wgpu-based Rust 2D game engine. On top of an ECS architecture it provides physics (Rapier2D), audio, particles, tilemaps, UI, a scene system, and more. A separate game project (`rust-survivors`) uses this engine as a dependency.

- **Repository**: `https://github.com/ChunSam/skeleton-engine`
- **Local path**: `/Users/jkl/Projects/skeleton-engine`
- **Branch**: `main`
- **Engine source size**: ~5,400 LOC (entire src/)

---

## Completed work — by Phase

| Phase | Summary | Commit |
|---|---|---|
| Phase 1 | Audio pan, particle system, collision debug visualization, tilemap, input rebinding | `93d54c4` |
| Phase 2 | Added ECS `query4` | `93d54c4` |
| Phase 3 | `PhysicsWorld` encapsulation (accessor methods, pub(crate) internals) | `fa9013c` |
| Phase 4 | `query_opt2`, `Events<E>` event system, UI Widget System | `767a1d2` |
| Phase 5 | Scene system (Scene/SceneCmd/SceneChange), Timer, Tween (6 Easing variants) | `2147291` |
| Phase 6 | UI system enhancements — TextInput, ScrollView, Panel+LayoutSystem | `e98b893` |
| Phase 7 | CollisionEvent — Rapier NarrowPhase polling → bridged to `Events<CollisionEvent>` | `b4a931d` |
| Phase 8 | Save/Load completion — `load_or_default`, `exists`, `delete`, lib.rs re-export | `01f983b` |
| Phase 9 | ECS Archetype storage — TypeId HashMap+Vec → Archetype dense column storage | `a8b49cc` |
| Phase 10 | Post-processing — vignette, chromatic aberration, approximate bloom (PostProcessConfig resource) | `a8b49cc` |
| Phase 11 | Audio enhancements — spatial audio, bus mixer, fade in/out | `a8b49cc` |
| Phase 12 | Transform hierarchy — Parent/Children/GlobalTransform, HierarchySystem, attach/detach | `3862f8d` |
| Phase 13 | Physics raycast + character controller — RaycastHit, add_kinematic_*, move_character | `eee451d` |
| Phase 14 | Animation state machine — AnimationStateMachine, StateMachineSystem, TransitionCond, AnimParam | `93eb65f` |
| Phase 15 | Gamepad (gilrs) + UI Slider/CheckBox — GamepadState, Slider, CheckBox, UiEvent extension | `30d1b9e` |
| Phase 16 | Scene serialization + prefab system — Tag, EntityDef, SceneDef, Prefab, spawn_entity_def | `2bfbffa` |
| Phase 17 | Asset pipeline + hot reloading — Handle<T>, ImageAsset, AssetServer, App::load_image | `f985118` |
| Phase 18 | egui in-game debug editor — DebugUi, F1 toggle, built-in Engine Stats panel | `83838a7` |
| Phase 19 | Rhai scripting — ScriptAsset, ScriptRunner, ScriptingSystem, App::load_script | `861e832` |
| Phase 20 | Animation blending — BlendWeight, play_with_crossfade, BlendTree1D, BlendTreeSystem | `d6ff7f9` |
| Phase 21 | Texture Atlas — TextureAtlas, AtlasSprite, AssetServer::load_atlas, App::load_atlas | `b63e9c9` |
| Phase 22 | Reflect system — Reflect trait, ReflectValue, World::register_reflect/get_reflect, egui Inspector | `90f65e3` |
| Phase 23 | WASM build support — platform-specific dep split, cfg-gate, EventLoopExtWebSys, getrandom wasm_js | `b9f4bdb` |
| Phase 24 | WASM browser run — force WebGL2, async GPU init, web-time, canvas size fix | `24e2108` |
| Phase 25-A | WebSocket networking — NetworkClient (native tungstenite / WASM web-sys), NetworkEvent, NetworkSystem | `88311e9` |
| Phase 25-B | ECS parallel queries — rayon par_query_for_each/map, par_query2_for_each/map, Send+Sync component storage | `4637ace` |
| Phase 25-C | Custom shader materials — ShaderMaterial, params uniform, pipeline cache, sprite batching cleanup | `9a7b375` |
| Phase 25-D | Editor gizmos — SelectedEntity resource, Inspector entity create/delete, drag move, DebugRect highlight | `c19d0b6` |
| Phase 25-E | rust-survivors integration — adapt to Sprite fields, EnemyAiSystem par_query2_map parallelization (game repo) | — |
| Phase 26 | LOD/culling — Camera::visible_rect, CullConfig resource, rotation-aware AABB frustum culling, min_pixel_size LOD | `8db9bbe` |
| Phase 27 | Multiplayer demo — mp_server (relay server) + mp_client (game client) examples | — |
| Phase 28 | Editor scene save — Inspector "💾 Save Scene" button, SceneDef RON serialization | — |
| Phase 29 | Scene hierarchy serialization — EntityDef.parent, two-pass spawn_scene_def, topological_sort_entities | — |
| Phase 30 | System profiler — System::name(), ProfilerData/RenderStats resources, Engine Stats panel extension | — |
| Phase 31 | Asset browser — ImageEntry, image_list(), Inspector "Assets" tab | — |
| Phase 32 | Runtime stability — AssetLoadState, SceneDef.version, Inspector 📂 Load Scene button | — |
| Phase 33 | A* pathfinding — PathGrid, find_path + ECS query filters query_with/query_without | — |
| Phase 34 | RenderLayer + sprite batching — (layer, tex_key, z) sort, single draw per same texture | — |
| Phase 35 | Inspector Undo/Redo — EditorCmd/EditorHistory, Ctrl+Z / Ctrl+Shift+Z | — |
| Phase 36 | Behavior tree — BehaviorNode trait, Sequence/Selector/Inverter/AlwaysSucceed, BehaviorSystem | — |
| Phase 37a | Blackboard + Steering Behaviors — BlackboardValue, Seek/Flee/Arrive/Wander, SteeringSystem | — |
| Phase 37d | CommandBuffer — Commands::spawn/despawn/insert/remove + World::apply_commands | — |
| Phase 38a | Scene graph panel — Inspector "Scene" tab TreeView, Tag name editing, hierarchy visualization | — |
| Phase 38d | Rhai scripting API extension — spawn_entity, despawn_entity, bb_set/get_*, seek/flee/stop_steering | — |
| Phase 39b | Inspector component add/remove UI — factory pattern, register_component, ComboBox, ✕ button | — |
| Phase 39d | REFERENCE.html v0.38.0 — added Steering/Blackboard/Commands/SceneGraph/Rhai sections | — |
| Phase 40c | Gizmo Grid Snap — snap_enabled/snap_size, snap_to_grid helper, Inspector checkbox + DragValue | — |
| Phase 40d | REFERENCE.html v0.39.0 — documented component add/remove UI, register_component API | — |
| Phase 41b | ECS change detection — query_added<T>/query_changed<T>, clear_change_tracking, HashSet-based | `5cf3233` |
| Phase 41a | 2D lighting — PointLight component, AmbientLight resource, LightingRenderer (WGSL, up to 16 lights) | `2230e31` |
| Phase 41d | REFERENCE.html v0.41.0 — added 2D lighting / ECS change detection sections | — |
| Phase 42a | 2D normal-map lighting — Sprite.normal_texture, PointLight.light_height, normal buffer, Lambert diffuse WGSL | `bdcd5c8` |
| Phase 42b | Camera effects — shake(strength,duration), follow_entity/lerp_factor, zoom_to(target,speed) | `b83ff6b` |
| Phase 42d | Object pool — Pool::new/acquire/release/clear, Pooled marker component | `b83ff6b` |
| Phase 43b | Entity cloning — World::clone_entity, register_clone<T>, Inspector Duplicate button | `e3717c9` |
| Phase 43c | Debug Draw API — DebugDraw::rect/line/circle/cross, auto-cleared every frame | `a6accb5` |
| Phase 43d | Timeline/cutscene — Timeline, Track<T>, Keyframe, Lerp trait, TimelineSystem | `6fc4007` |
| Phase 43a | Scene transition — FadeTransition::fade_in/out, FadeRenderer (WGSL alpha blend) | `c4a05f6` |
| Phase 44b | Physics joints — add_distance_joint/revolute_joint/prismatic_joint, ImpulseJointHandle | `96b35d1` |
| Phase 44c | Audio effects — AudioEffect (low_pass_hz/pitch/attack_secs), set_effect/clear_effect | `7ea4763` |
| Phase 45 | System execution order — SystemLabel/before/after (topological sort + cycle detection), SystemSet on/off | `9f2273d` |
| Phase 48 | Physics layers/sensors — CollisionGroups (bitmask), sensor trigger zone, TriggerEvent | `b352ecd` |
| Phase 49 | Text completion — multiline/alignment (TextAlign), rich text, IME composition input (preedit) | `d648371` |
| Phase 50 | Localization — LocaleResource, t API, TextDirection (RTL), LocaleBundle/Data | `9321cac` |
| Phase 53 | Save security — chacha20poly1305 AEAD encryption + tamper detection (SaveError::Corrupted) | `34aa368` |
| Phase 46 | Render textures — RenderTarget, OffscreenCamera, SpriteRenderer rt_cache, offscreen pass | `96aa5d7` |
| Phase 47 | Touch input/mobile — TouchState (multi-touch/swipe/pinch), VirtualJoystick, WASM mouse emulation | `96aa5d7` |
| Phase 51 | Async asset loading — AssetLoadState::Loading, load_image_async, LoadProgress, WASM fetch | `96aa5d7` |
| Phase 52 | Panic recovery — catch_unwind system wrapper, PanickedSystems, crash log writing | `96aa5d7` |
| Phase 54 | Editor completion — PrefabInstance tracking, break_prefab_instance, multi-select Ctrl+C/V, group move | `96aa5d7` |
| Phase 56 | Advanced rendering — GpuParticleEmitter/GpuParticleRenderer (compute shader), PostProcess color grading | `687ec89` |
| Phase 55 | Release packaging — release/release-wasm profiles, scripts/build_wasm.sh | `706a263` |
| Phase 57c | CI — .github/workflows/ci.yml (native+WASM build, clippy, rustdoc) | `706a263` |
| Phase 57a/b | Rustdoc — fixed broken intra_doc_link/invalid_html_tags, RUSTDOCFLAGS="-D warnings" passing | `706a263` |
| Phase 59 | API Freeze — Cargo.toml v1.0.0, added keywords/categories/rust-version | `706a263` |
| Code review | Fixed 7 runtime/structural risks — Timeline NaN, TextureError fallback, World contamination, OffscreenCamera layer_mask, ScriptingSystem register_fn one-time, Network backpressure, egui unsafe documentation | `4084cee` |

> Phase 46~59 all complete. 7 code-review fixes + actual wss:// TLS read timeout fix complete.
> Latest verification: `cargo fmt --check` passing / `cargo test` unit 207 + doctest 31 passing / `cargo clippy --all-targets --locked -- -D warnings` passing / `cargo package --locked --allow-dirty --list` passing / `cargo publish --dry-run --locked --allow-dirty` passing. Since the current changes are pre-commit, packaging verification used `--allow-dirty`. **skeleton-engine v1.0.0 is ready for release.**

## post-v1.0 stability improvement notes

These are compatibility-preserving improvements that reduce the "may silently behave differently than intended" risks found in analysis. Default runtime behavior stays identical to v1.0, and strict failure handling is provided as opt-in APIs.

- `ScheduleErrorPolicy` / `SystemPanicPolicy` let you opt into stricter handling of schedule cycles and system panics. The defaults keep the existing compatible behavior.
- `examples/runtime_policies.rs` shows the configuration form of `PanicOnCycle`, `AbortAfterLog`, and the default panic-recovery policy in a short command-line example.
- `World::mark_changed<T>()` / `World::get_mut_tracked<T>()` were added so that direct field mutations can be explicitly reflected in `query_changed<T>()`.
- The native `AssetServer` normalizes existing file paths to canonical paths, reducing duplicate caching of relative/absolute paths and hot-reload matching problems. WASM paths are not normalized, to preserve URL semantics.
- `SpriteRenderer` caches file textures under both the original requested path and the canonical handle path. After `App::load_image()` with a relative path, `Sprite::textured_with_handle(...)`, `DrawImage::textured_with_handle(...)`, and `AtlasSprite` do not fall back to white even when prioritizing the handle path. Guidance for removing the game-side temporary workaround is documented in `docs/RUST_SURVIVORS_TEXTURE_CACHE_KEY_PROMPT.md`.
- `PhysicsSystem` documents the `pixels_per_unit` unit convention and, in release builds, defends against abnormal values by clamping to a minimal positive value. In debug builds, inputs of 0 or below are caught with `debug_assert`.
- Rhai scripting is for trusted local game code, not a hostile sandbox. The negative return value of `spawn_entity()` is not a stable handle for manipulating the actual entity within the same script.
- It is documented in rustdoc that `Entity(pub u32)` has no generation number, so an ID may be reused after despawn. The structural change is left as a v2 candidate.
- The v2 `Entity` generation-number design is documented as a finalized plan in `docs/ENTITY_GENERATION_V2_PLAN.md`. The core direction is `Entity { index, generation }`, stale handle no-op, and removing `entity.0`.
- Verification: `cargo fmt`, `cargo run --example runtime_policies`, `cargo test --all-targets` (library 218 tests + `mp_server` 3 tests), `cargo clippy --all-targets -- -D warnings` passing. After the texture cache key fix, a separate `cargo test` also passes with unit 225 + doctest 31 passing / 19 ignored.

---

## Current structure

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

## Work this session (7 code-review fixes — v1.0.0 quality hardening)

> At the time, the 7 runtime/structural risk items from ENGINE_REVIEW_FIX_PROMPT.md were fixed in priority order.
> No changes to existing public APIs.
> Verification record at the time: `cargo test` lib 196 tests + doctest 31 passing / `cargo clippy --all-targets -- -D warnings` passing. The current verification baseline follows the 207 unit + 31 doctest results in the summary above.

### 1. Removed Timeline NaN sampling panic

**Changed file**: `src/timeline.rs`

**Problem**: When calling `Track::sample(f32::NAN)`, all `rposition` comparisons are false → `unwrap()` panic.

**Fix**: Added an early return `if t.is_nan() { return None; }`.

**Added tests**: `track_sample_nan_returns_none`, `track_nan_keyframe_does_not_panic_normal_sample`

---

### 2. Texture loading panic → fallback conversion

**Changed files**: `src/renderer/texture.rs`, `src/lib.rs`

**Problem**: `from_path` panics on missing file / decode failure.

**Fix**:
- Added `TextureError { Io(std::io::Error), Decode(image::ImageError) }`
- Added `try_from_path(...)  -> Result<Self, TextureError>`
- `from_path` → on `try_from_path` failure, magenta 1×1 fallback + `log::warn!`
- Added `decode_image_bytes(bytes) -> Result<(Vec<u8>, u32, u32), TextureError>` (for GPU-free testing)
- Added `pub use renderer::texture::TextureError` to `src/lib.rs`
- 3 added tests (IO error, decode error, valid PNG)

---

### 3. Fixed OffscreenCamera World state contamination

**Changed files**: `src/ecs/world.rs`, `src/app.rs`

**Problem**: Even with no Camera before the offscreen render, it was saved with `unwrap_or_default()` → a Camera was always inserted after rendering.

**Fix**:
- Added a `World::remove_resource<T>()  -> Option<T>` method
- app.rs: save `Option<Camera>` → on restore, if `None`, call `remove_resource::<Camera>()`
- 2 added tests: `world_remove_resource_removes_and_returns`, `world_remove_resource_missing_returns_none`

---

### 4. OffscreenCamera self-capture prevention (layer_mask)

**Changed files**: `src/components.rs`, `src/renderer/sprite.rs`, `src/app.rs`, `examples/minimap.rs`, `examples/split_screen.rs`

**Problem**: The offscreen pass could include the very sprite that displays its result in the render set.

**Fix**:
- Added `OffscreenCamera.layer_mask: u32` field (0 = allow all, backward compatible)
- Added `layer_mask: u32` parameter to `SpriteRenderer::render(...)`
- Sprite collection loop: bit-filter when `layer_mask != 0`
- Offscreen pass passes `cam.layer_mask`, main pass passes `0`
- `examples/minimap.rs`: `layer_mask: 1 << 0` (game world only, excluding UI sprites)

---

### 5. Removed duplicate ScriptingSystem register_fn calls

**Changed file**: `src/scripting.rs`

**Problem**: The 11 `register_fn` calls were repeated every frame for every N entities → accumulation in the Rhai internal registry.

**Fix**: Introduced the `thread_local! { static SCRIPT_CTX: RefCell<Option<ScriptCtx>> }` pattern.
- Register all functions **only once** in `with_limits()`; access the context via `SCRIPT_CTX.with(|c| { ... })`
- `run()` loop: create per-entity buffer → `set_script_ctx(ctx)` → execute → `clear_script_ctx()`
- Kept all existing API names (`spawn_entity`, `bb_set_bool`, etc.)
- 3 added tests: spawn command, bb round-trip, no buffer contamination between two entities

---

### 6. Network backpressure + wss:// TLS read timeout fix

**Changed file**: `src/network.rs`

**Problem**: Unbounded send channel → unlimited memory growth on slow connections. Whether `send_text`/`send_bytes` drops was opaque. On `wss://` TLS connections, `socket.read()` could block, delaying send/close handling.

**Fix**:
- Added `NetworkConfig.max_pending_messages: usize` field (default 256)
- Replaced `mpsc::channel` → `mpsc::sync_channel(config.max_pending_messages)`
- `NetworkClient.msg_tx`: `Sender` → `SyncSender`
- `send_bytes`/`send_text`: use `try_send`, when the queue is full `log::warn!` + drop (signature preserved)
- Added `try_send_bytes(&self, data: &[u8]) -> bool`, `try_send_text(&self, text) -> bool`
- **Actual TLS read timeout fix**: In the `MaybeTlsStream::Rustls(tls)` variant, call `set_read_timeout(5ms)` directly on `tls.sock` (`rustls::StreamOwned.sock: pub TcpStream`). `wss://` connections can now check the send channel at 5 ms intervals, just like plain TCP.
- 1 added test: `network_bounded_channel_drops_on_full`

---

### 7. Isolating and documenting the egui unsafe helper

**Changed file**: `src/app.rs` (`egui_render_pass` function)

**Problem**: Uses 2 `transmute`s — can't be immediately replaced with a safe API (egui-wgpu 0.29 requirement).

**Fix**: Documentation only, no code change. Converted `fn egui_render_pass` into a documented function:
- Documented the 3 invariants in a `/// # Safety` section
- Added a removal checklist for when egui-wgpu is upgraded

---

## Work this session (Phase 46~59 — v1.0.0 completion)

> At the time, REMAINING_WORK.md Track A serial chain (46→47→51→52→54→56) + Track B (55, 57c) + Solo (57a/b, 59) were completed in a single session.
> 183 tests passing at the time. The current v1.0.0 gate is the pre-commit verification baseline: `cargo test` unit 207 + doctest 31, `cargo clippy --all-targets --locked -- -D warnings`, `cargo publish --dry-run --locked --allow-dirty` passing.

### Phase 46 — Render textures (Offscreen Render Targets)

**Changed files**: `src/renderer/render_target.rs` (new), `src/renderer/mod.rs`, `src/components.rs`, `src/app.rs`, `src/lib.rs`

- `RenderTarget { texture, view, sampler, bind_group: Arc<BindGroup>, width, height }` — RENDER_ATTACHMENT|TEXTURE_BINDING
- `OffscreenCamera { target: String, camera: Camera }` component
- `App::create_render_target(name, w, h)` + offscreen render pass (runs before the main pass)
- `SpriteRenderer::register_render_target(key, bg)` + `rt_cache` — use a render texture as a sprite source
- `examples/minimap.rs`, `examples/split_screen.rs`

### Phase 47 — Touch input + mobile

**Changed files**: `src/input/touch.rs` (new), `src/input/mod.rs`, `src/ui/joystick.rs` (new), `src/ui/mod.rs`, `src/app.rs`, `src/lib.rs`

- `TouchState`: multi-touch HashMap, began/moved/ended frame buffers, pinch/swipe detection
- `VirtualJoystick`: TouchState → normalized Vec2 output, `update_raw()`, `output_with_deadzone(f32)`
- `WindowEvent::Touch` handling + WASM/PC mouse emulation
- `examples/touch_demo.rs`

### Phase 51 — Async asset loading

**Changed files**: `src/asset.rs`, `src/app.rs`, `src/resources.rs`, `src/lib.rs`, `Cargo.toml`

- Added `AssetLoadState::Loading`, `AsyncImageResult` + mpsc channel (native) / thread_local VecDeque (WASM)
- `App::load_image_async(path)` — spawn_blocking (native) / wasm_bindgen_futures::spawn_local
- `LoadProgress { total, loaded }` resource, `poll_async_completions()` processed every frame

### Phase 52 — Panic recovery

**Changed files**: `src/app.rs`, `src/resources.rs`, `src/lib.rs`

- Isolate system panics with a `catch_unwind(AssertUnwindSafe(|| system.run(...)))` wrapper
- Keep the index of the panicking system in a `HashSet<usize>`, skipping it in subsequent frames
- `PanickedSystems { disabled: Vec<String> }` resource, `write_crash_log()` (native only)

### Phase 54 — Editor completion

**Changed files**: `src/app.rs`, `src/prefab.rs`, `src/lib.rs`

- `PrefabInstance { source_path: String }` component, `Prefab::spawn_with_tracking()`
- `break_prefab_instance(world, entity)` — removes PrefabInstance
- Inspector Ctrl+C (copy)/Ctrl+V (paste), multi-entity selection/group move

### Phase 56 — Advanced rendering

**Changed files**: `src/gpu_particle.rs` (new), `src/renderer/gpu_particle.rs` (new), `src/renderer/shaders/gpu_particle_*.wgsl` (new), `src/renderer/post_process.rs`, `src/renderer/shaders/post_process.wgsl`, `src/app.rs`, `src/lib.rs`

**56a GPU particles**:
- `GpuParticleEmitter` component, ring-buffer (4096 slots) emission logic
- `GpuParticleRenderer`: compute shader (WGSL) physics simulation + render pipeline (6 vertices per particle)
- App render loop step 2.8 lazy-init integration, dt passed into render() via `last_dt` field
- `examples/gpu_particles.rs`

**56b post-process color grading**:
- `PostProcessConfig`: added `brightness/contrast/saturation` parameters
- `post_process.wgsl`: WGSL color-grading code (brightness → contrast → saturation order)

### Phase 55/57c/57a-b/59 — Release/CI/Rustdoc/API Freeze

- `Cargo.toml`: release/release-wasm profiles (LTO/strip), v1.0.0 metadata
- `scripts/build_wasm.sh`: wasm-bindgen automation + index.html generation
- `.github/workflows/ci.yml`: native (test/clippy/fmt/release) + WASM + rustdoc checks
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` 0 warnings

### Phase 45 — Explicit system execution order (Track A)

**Changed files**: `src/ecs/schedule.rs` (new), `src/ecs/mod.rs`, `src/app.rs`, `src/lib.rs`

- `SystemLabel` (= `&'static str`), `SystemConfig` builder (`.label/.before/.after/.in_set`), `SystemMeta`
- `compute_order()` — Kahn topological sort, insertion-order tiebreaker (deterministic), `ScheduleError::Cycle` on a cycle
- `App::add_system_labeled(system, config)` — registration with order specification (existing `add_system` unchanged)
- `App::set_enabled(set, bool)` — batch on/off of a SystemSet group
- `update()` loop: when dirty, recompute `exec_order` → iterate + skip disabled sets
- After scene Push/Pop/Replace, sync meta with `reconcile_meta()` (`Scene::on_enter` signature unchanged)

### Phase 48 — Physics layers/masks + sensors (Track B)

**Changed files**: `src/physics/world.rs`, `src/physics/events.rs`, `src/physics/system.rs`, `src/physics/mod.rs`, `src/lib.rs`

- `CollisionGroups` (wrapping rapier `InteractionGroups`, bitmask) — selective per-layer collision
- Sensors (trigger zone) — `sensor(true)` collider + `intersection_pairs()` polling
- `TriggerEvent` (Entered/Exited) — the user must `register_event::<TriggerEvent>()`

### Phase 49 — Text completion + IME (Track A)

**Changed files**: `src/renderer/text.rs`, `src/ui/text_input.rs`, `src/input/state.rs`, `src/app.rs`, `src/lib.rs`

- Multiline + alignment (`TextAlign::Left/Center/Right`), rich text
- IME composition input — winit `WindowEvent::Ime` handling, preedit preview rendering while composing

### Phase 50 — Localization i18n (Track B)

**Changed files**: `src/locale.rs` (new), `src/lib.rs`

- `LocaleResource` — locale → key → string, switching API, `t(key)` lookup + fallback
- `LocaleBundle`/`LocaleData` (RON parsing), `TextDirection` (RTL metadata)
- WASM compatible: file reading is the caller's responsibility, the API only parses strings/bytes

### Phase 53 — Save data security (Track B)

**Changed files**: `src/save.rs`, `Cargo.toml`, `src/lib.rs`

- Encrypt RON-serialized data with `chacha20poly1305` (ChaCha20-Poly1305 AEAD)
- Tamper detection via the AEAD authentication tag → `SaveError::Corrupted` (no separate checksum needed)
- Keeps the existing save/load API signatures backward compatible + provides SaveKey/save_with_key/load_with_key for per-app keys

---

## Previous session (Phase 44)

### Phase 44b — Physics joints

**Background**: Many mechanisms in platformer/puzzle genres — chains, hinged doors, sliding platforms — are impossible without joints. Rapier2D's `ImpulseJointSet` was already in `PhysicsWorld`, but there was no public API.

**Changed files**: `src/physics/world.rs`, `src/physics/mod.rs`, `src/lib.rs`

**Added features**:
- `add_distance_joint(body1, body2, anchor1, anchor2, rest_length) -> ImpulseJointHandle` — spring-based distance joint (stiffness=1000, damping=10). Since Rapier2D 0.22 has no `DistanceJointBuilder`, implemented with `SpringJointBuilder`
- `add_revolute_joint(body1, body2, anchor1, anchor2) -> ImpulseJointHandle` — hinge (pivot) joint
- `add_prismatic_joint(body1, body2, anchor1, anchor2, axis) -> ImpulseJointHandle` — slider joint, `axis` is normalized internally to a `UnitVector`
- `remove_joint(handle: ImpulseJointHandle)` — remove a joint
- `ImpulseJointHandle` — re-exported from `src/physics/mod.rs` as `rapier2d::prelude::ImpulseJointHandle`, exposed via `src/lib.rs`
- 3 unit tests (distance create/remove, revolute create, prismatic create)

---

### Phase 44c — Audio effects

**Background**: The existing AudioManager supported only volume/pan/fade. There were no gameplay-feedback effects like low-pass (sounds beyond a monster's wall), pitch manipulation (sense of speed/slowdown), or fade-in (attack).

**Changed files**: `src/audio.rs`, `src/lib.rs`

**Added features**:
- `AudioEffect` struct: `low_pass_hz: Option<u32>`, `pitch: f32`, `attack_secs: f32`, `release_secs: f32`. `Default::pitch = 1.0`
- `AudioManager::effects: HashMap<String, AudioEffect>` field
- `set_effect(channel, effect)` — set an effect (applied on the next play_* call)
- `clear_effect(channel)` — reset the effect
- `effect(channel) -> Option<&AudioEffect>` — query the current effect
- Modified `play_internal`: box as `Box<dyn Source<Item=i16> + Send + 'static>` and chain pitch (`.speed()`) → low_pass (`.low_pass()`) → attack (`.fade_in()`) in order
- 2 unit tests (default_pitch, set_and_clear)

---

## Work this session (Phase 43)

### Phase 43b — Entity cloning

**Background**: There was no way to duplicate an entity in the Inspector, and no API to copy an entity programmatically from system code either.

**Changed files**: `src/ecs/world.rs`, `src/app.rs`

**Added features**:
- `World::register_clone<T>()` — register a clone closure per TypeId
- `World::clone_entity(src) -> Entity` — copy all registered types (remove→call→reinsert pattern to avoid borrow conflicts)
- `World::has_component_typeid(entity, TypeId) -> bool` — check component existence by TypeId
- Default registered types: `Transform`, `Sprite`, `RenderLayer`, `Tag`, `AnimationPlayer`, `Timer`
- Inspector "Duplicate" button — applies a (16,16) offset and selects the new entity
- 3 unit tests

---

### Phase 43c — Debug Draw API

**Background**: To draw debug shapes (collision boxes, paths, radii) from a game system, you had to create ECS entities directly or use the existing `DebugDrawQueue` (Rect only) directly.

**Changed files**: `src/resources.rs`, `src/app.rs`, `src/lib.rs`

**Added features**:
- `DebugShape` enum: `Rect`, `Line`, `Circle`, `Cross`
- `DebugDraw` resource: `rect/line/line_thick/circle/cross/clear/shapes` methods
- Rendering: converted to `DrawRect`/`UiQueue` (lines = dot-chain, circles = 24-gon)
- App auto-registration + automatic clear via `std::mem::take` after rendering
- 4 unit tests

---

### Phase 43d — Timeline/cutscene

**Background**: There was no standard API to control entity movement/color changes over time. Each had to be implemented with a custom timer system.

**Changed files**: `src/timeline.rs` (new), `src/lib.rs`

**Added features**:
- `Lerp` trait: implemented for `f32`, `Vec2`, `[f32;4]`
- `Keyframe<T>` — `time`, `value`, `easing: Easing` (reusing the existing tween.rs)
- `Track<T: Clone+Lerp>` — keyframe sorting, `sample(t)` binary-search interpolation, `add/duration/is_empty`
- `Timeline` ECS component — `position/rotation/scale/color/alpha` tracks, `playing/looping`, `play/pause/restart/is_finished`
- `TimelineSystem` — applies to Transform/Sprite without borrow using the `take_component` pattern
- 11 unit tests

---

### Phase 43a — Scene transition

**Background**: Scene transitions swapped instantly with no visual continuity. A fade in/out overlay provides a smooth transition.

**Changed files**: `src/renderer/fade.rs` (new), `src/renderer/mod.rs`, `src/resources.rs`, `src/app.rs`, `src/lib.rs`

**Added features**:
- `FadeTransition` resource: `fade_out(duration)`, `fade_in(duration)`, `.with_color(r,g,b)`, `update(dt)`, `finished` field
- `FadeRenderer` (native only): full-screen solid quad, `alpha blend`, `LoadOp::Load` (overlay)
- WGSL shader: `FadeUniforms { color: vec3, alpha: f32 }` 16-byte uniform
- App integration: lazy initialization, runs last after all render passes, runs the pass only when `alpha > 0.001`
- 4 unit tests

---

## Work this session (Phase 42b + 42d)

### Phase 42b — Camera effects

**Background**: The camera supported only a static position/zoom, leaving no way to implement gameplay feedback like hit effects (shake), player tracking (follow), or scene-transition staging (zoom tween).

**Changed files**: `src/camera.rs`, `src/app.rs`

**Added features**:

*New Camera struct fields*
- `shake_strength/shake_duration/shake_timer` — shake state
- `follow_entity: Option<Entity>` — tracking target
- `lerp_factor: f32` (default 5.0) — tracking lerp strength (per second)
- `zoom_target/zoom_tween_speed` — zoom tween state

*New methods*
- `Camera::shake(strength, duration)` — schedule a screen shake
- `Camera::zoom_to(target_zoom, speed)` — smooth zoom tween
- `Camera::shake_offset() -> Vec2` — current shake pixel offset (sin/cos sum of two frequencies)
- `Camera::update(dt, follow_pos: Option<Vec2>)` — advance effects (called by App every frame)

*view_proj change*: shake is automatically reflected via `self.position + self.shake_offset()`

*App::update() change*: read the `follow_entity` position first, then call `camera.update(dt, follow_pos)` (to avoid borrow conflicts)

Added 6 unit tests (shake decay, zoom tween full/partial, follow lerp, no-follow noop, active shake offset)

---

### Phase 42d — Object pool

**Background**: When mass spawn/despawn entities like bullets/particles repeat spawn/despawn every frame, archetype reallocation overhead occurs. A Pool reuses entities to cut the cost.

**Changed files**: `src/pool.rs` (new), `src/lib.rs`

**Added features**:

*Pool struct*
- FIFO reuse queue based on `VecDeque<Entity>`
- `acquire(world, setup)` — take from the pool or spawn. Remove the `Pooled` marker, then run the setup closure
- `release(entity, world)` — add the `Pooled` marker and return to the queue. Despawn when over capacity
- `clear(world)` — empty the entire pool
- Entities despawned externally are automatically skipped via an `is_alive` check (idempotency guaranteed)

*Pooled marker component*: inactive entities can be excluded with `query_without::<SomeComp, Pooled>()`

Added 5 unit tests (spawn, reacquire, overflow, clear, skip-dead)

---

## Work this session (Phase 42)

### Phase 42a — 2D normal-map lighting

**Background**: Phase 41a's `PointLight` system applied only distance attenuation (atten²), so everything brightened uniformly from all directions. Adding a normal map and Lambert diffuse implements directional lighting where surface relief is expressed according to the light's direction.

**Changed files**: `src/components.rs`, `src/renderer/lighting.rs`, `src/app.rs`, `src/particle.rs` (Sprite struct literal fix)

**Added features**:

*`Sprite` extension*
- `normal_texture: Option<String>` — normal-map file path (RON serialization supported)
- `normal_handle: Option<Handle<ImageAsset>>` — runtime handle (`#[serde(skip)]`)

*`PointLight` extension*
- `light_height: f32` (default `0.15`) — virtual Z height of the light source. Used as the Z component of the L vector. Lower → side lighting → emphasizes normal-map relief; higher → frontal lighting

*`GpuLightData` change*
- `_pad: f32` → `light_height: f32` (kept at 32 bytes, `LightingUniforms` kept at 544 bytes)

*Normal buffer (LightingRenderer)*
- `normal_texture: wgpu::Texture` + `normal_view: wgpu::TextureView` — `Rgba8Unorm`, `RENDER_ATTACHMENT | TEXTURE_BINDING`
- `clear_normal_buffer(encoder)` — initialize to flat normals with `LoadOp::Clear([0.5, 0.5, 1.0, 1.0])` (no draw call)
- `resize()` also re-creates normal_texture
- bind group layout binding 3 = normal_tex (fragment)
- `run_pass()` 4 bind group entries

*WGSL shader (Lambert diffuse)*
```wgsl
let N = normalize(n_sample.xyz * 2.0 - vec3(1.0));
let L = normalize(vec3(diff_uv.x, -diff_uv.y * aspect_ratio, l.light_height));
let diffuse = max(0.0, dot(N, L));
total = total + l.color * l.intensity * diffuse * atten * atten;
```

**Current behavior**: The normal buffer is always initialized to flat normals → per-sprite normal-texture rendering is planned for a future SpriteRenderer extension. Even with `light_height` alone there is a side/frontal lighting transition effect.

```rust
// 노멀 맵 스프라이트 설정 예
let mut sprite = Sprite::textured("assets/stone.png");
sprite.normal_texture = Some("assets/stone_normal.png".to_string());

// 측면 방향성 조명
world.add_component(light, PointLight {
    color: [1.0, 0.7, 0.3],
    radius: 400.0,
    intensity: 2.5,
    light_height: 0.08,  // 낮게 → 노멀 맵 요철 강조
});
```

---

## Work this session (Phase 41)

### Phase 41b — ECS change detection

**Background**: Tracking entities whose components changed required adding a manual dirty-flag field to each component. `query_added`/`query_changed` allow querying only the components added or replaced this frame.

**Changed files**: `src/ecs/world.rs`, `src/app.rs`

**Added features**:
- Added `added_this_tick: HashSet<(Entity, TypeId)>` and `changed_this_tick: HashSet<(Entity, TypeId)>` fields to the `World` struct
- Modified `add_component`: record in `added_this_tick` on first add, in `changed_this_tick` on replacement
- Modified `despawn` / `remove_component`: remove the entity/type pair from the tracking sets (idempotency guaranteed)
- `World::clear_change_tracking()` — auto-called at the very top of `App::update()` every frame
- `World::query_added::<T>()` / `World::query_changed::<T>()` — filtered query based on the tracking sets
- 4 unit tests

```rust
// 이번 프레임에 새로 추가된 Enemy만 초기화
let new_enemies: Vec<_> = world.query_added::<Enemy>().map(|(e, _)| e).collect();
for e in new_enemies {
    world.add_component(e, Blackboard::default());
}

// 위치가 바뀐 오브젝트만 공간 캐시 갱신
for (e, t) in world.query_changed::<Transform>() {
    cache.update(e, t.position);
}
```

---

### Phase 41a — 2D lighting

**Background**: The engine had no lighting effects, so every scene was uniformly bright. Simply registering the `AmbientLight` resource activates the lighting post-pass, and a `PointLight` + `Transform` combination lets you place point lights.

**Changed files**: `src/renderer/lighting.rs` (new), `src/renderer/mod.rs`, `src/components.rs`, `src/resources.rs`, `src/app.rs`, `src/lib.rs`

**Added features**:

*Components / resources*
- `PointLight { color: [f32;3], radius: f32, intensity: f32 }` — added to an entity together with Transform
- `AmbientLight { color: [f32;3], intensity: f32 }` (default: white 10%) — activates lighting immediately on registration

*LightingRenderer (src/renderer/lighting.rs)*
- `GpuLightData` (32 bytes), `LightingUniforms` (544 bytes) — bytemuck `Pod+Zeroable`
- Handles up to 16 `PointLight`s, `atten*atten` quadratic attenuation function
- Inline WGSL shader: sample scene_tex → sum ambient + point lights → clamp with min(total, 1.0)
- World→NDC conversion: `ndc_x = pos.x / (vp_w / 2)`, `radius_ndc = radius / (vp_w / 2)`

*App integration (src/app.rs)*
- `lighting_renderer: Option<LightingRenderer>` field (native only)
- Auto-enable/disable when the `AmbientLight` resource exists
- Lighting-dedicated intermediate scene texture (`scene_texture_for_lighting`) — when lighting+post are both active, post output is the lighting input
- `gpu_struct_sizes` unit test (verifies GpuLightData=32, LightingUniforms=544)

```rust
// 라이팅 씬 설정 예
world.insert_resource(AmbientLight { color: [1.0, 1.0, 1.0], intensity: 0.05 });

let torch = world.spawn();
world.add_component(torch, Transform { position: Vec2::new(0.0, 0.0), ..Default::default() });
world.add_component(torch, PointLight { color: [1.0, 0.7, 0.3], radius: 250.0, intensity: 1.5 });
```

> **Platform**: native only (`#[cfg(not(target_arch = "wasm32"))]`).

---

## Work this session (Phase 24)

### Phase 24 — WASM browser run

**Background**: In Phase 23, `cargo build --target wasm32-unknown-unknown` passed, but it didn't actually run in a browser. The goal was to get a `wasm-pack build` → HTTP server → Chrome demo of color boxes bouncing around to work.

**Verification**: `wasm-pack build --target web` succeeded; confirmed in Chrome that 10 color boxes run ECS + bounce physics

**Changed files**: `Cargo.toml`, `src/app.rs`, `src/renderer/context.rs`, `src/lib.rs`

**List of problems solved (in order of occurrence)**

| Problem | Cause | Fix |
|------|------|------|
| `requestDevice` failure — `maxInterStageShaderComponents` unrecognized | wgpu 22 `Limits::default()` includes a limit Chrome WebGPU doesn't recognize | `context.rs`: force `Backends::GL` (WebGL2) on WASM + `downlevel_webgl2_defaults()` |
| `std::time::Instant` panic | `std::time::Instant` unsupported on WASM | Added `web-time = "1"` to `Cargo.toml`; `#[cfg] use web_time::Instant` in `app.rs` |
| GPU `async` init impossible (`unreachable` panic) | WebGPU is Promise-based → can't simply poll | `thread_local! PENDING_GPU` + `spawn_local` + pick-up in `about_to_wait`/`RedrawRequested` |
| `surface: 1x1` | `window.inner_size()` returns 1×1 right after canvas attach | `context.rs`: on WASM read width/height directly from the `#[game-canvas]` DOM element |
| `no default font found` panic | No system font on WASM — panics during `cosmic-text` shaping | `finish_init()`: skip `TextRenderer` creation on WASM when `font_bytes` is empty |
| `Surface size (2560×1440) > WebGL2 max (2048)` | Retina DPR=2 → winit `Resized` event reports physical pixels | `app.rs` `Resized` handler: substitute the DOM canvas size on WASM |

**Key design decisions**
- Force WebGL2 (`Backends::GL`) instead of WebGPU: avoids the WebGPU limit-spec mismatch across Chrome versions. `wgpu = { features = ["webgl"] }` is already declared, so no extra dependency
- `PENDING_GPU thread_local`: complete the GPU context in the `spawn_local` future, then store it → poll from the main event loop (`about_to_wait` + `RedrawRequested`). Checking in both places prevents a timing race
- WASM text: text rendering requires injecting TTF bytes directly into the `FontData` resource. Without injection, the text renderer is skipped (no panic)

**WASM runtime behavior (updated)**

| Feature | WASM |
|------|------|
| wgpu rendering (WebGL2) | ✅ works |
| ECS, Sprite, animation | ✅ works |
| Text rendering | ✅ works when the FontData resource is injected (skipped if not) |
| Physics, Audio, Gamepad | disabled — `#[cfg(not(wasm))]` |

---

## Previous session (Phase 27–28)

### Phase 27 — Multiplayer demo

**Background**: Phase 25-A implemented NetworkClient/NetworkSystem, but there was no actual server-client demo. Two example binaries were added to `examples/` to show the usage patterns of the engine's networking API.

**mp_server** (`examples/mp_server.rs`):
- WebSocket accept via `TcpListener::bind("127.0.0.1:9001")`
- Per-client thread + `mpsc::Sender<Message>` broadcast map
- Non-blocking send/receive via a 5 ms read-timeout loop
- Protocol: on client connect send `{"type":"hello","id":N}`, relay position `{"type":"pos","id":N,"x":...,"y":...}`, leave notice `{"type":"bye","id":N}`

**mp_client** (`examples/mp_client.rs`):
- `NetworkClient::connect("ws://127.0.0.1:9001")` + register `NetworkSystem`
- `MultiplayerSystem`: local player (white square) WASD movement, 20 Hz position send
- Remote players: per-ID unique-color square, spawn/despawn on pos/bye receive
- HUD: shows connection status, Player ID, number of connected players

**Run**:
```
cargo run --example mp_server   # 터미널 1
cargo run --example mp_client   # 터미널 2, 3, ...
```

---

### Phase 29 — Scene hierarchy serialization

**Background**: In Phase 12 the `Parent`/`Children`/`GlobalTransform`/`HierarchySystem` hierarchy system was fully implemented, but the `EntityDef`/`SceneDef` serialization format was a flat list, so hierarchy relationships were lost when saving/loading scene files.

**Changed files**: `src/prefab.rs`, `src/app.rs`, `src/lib.rs`

**Added features**:
- Added a `parent: Option<String>` field to `EntityDef` (`#[serde(default, skip_serializing_if = "Option::is_none")]` keeps existing RON files backward compatible)
- Replaced `spawn_scene_def()` with a two-pass approach: pass 1 creates entities + tag→Entity map, pass 2 calls `hierarchy::attach()`
- Added the free function `topological_sort_entities(entities: &[Entity], world: &World) -> Vec<Entity>` (BFS, root→child order)
- When the editor saves a scene, sort with `topological_sort_entities()`, then read the `Parent` component to fill `EntityDef.parent`
- Re-export `topological_sort_entities` (`lib.rs`)
- Tests: added `scene_hierarchy_roundtrip`, `topological_sort_roots_before_children`

---

### Phase 30 — System profiler

**Background**: Per-system execution time and renderer stats (draw call count, culled sprite count) couldn't be checked in real time from the editor.

**Changed files**: `src/ecs/system.rs`, `src/resources.rs`, `src/renderer/sprite.rs`, `src/app.rs`, `src/lib.rs`

**Added features**:
- Added a `fn name(&self) -> &'static str { "" }` default method to the `System` trait (backward compatible with existing `impl System`)
- Added `SystemProfile { name, last_us, avg_us }`, `RenderStats { draw_calls, sprites_rendered, sprites_culled }`, `ProfilerData { systems, render, frame_ms }` resources
- `ProfilerData::record_system()` — EMA (α=1/60) moving average calculation
- Replaced the `App::update()` system loop with an `Instant` instrumentation wrapper, recording results into `ProfilerData`
- Changed `sprite.rs render()` return type to `RenderStats`, collecting culling/draw-call counters
- Added "Systems" / "Render" collapsible sections to the Engine Stats panel, changed to `resizable(true)`
- Re-export `ProfilerData`, `RenderStats`, `SystemProfile` (`lib.rs`)

---

### Phase 40c — Gizmo Grid Snap

**Background**: When moving entities by gizmo drag, they were placed only in pixel units, making alignment inconvenient for tilemap/grid-based level design.

**Changed file**: `src/app.rs`

**Added features**:
- `snap_to_grid(pos: Vec2, snap_size: f32) -> Vec2` helper function (native only)
- `App` struct fields: `snap_enabled: bool` (default `false`), `snap_size: f32` (default `16.0`)
- Added a "Snap" checkbox + grid size `DragValue` (1~128 px, suffix " px") at the top of the Inspector Entities tab
- After computing the gizmo drag position, apply `snap_to_grid` if `snap_enabled`

---

### Phase 39b — Inspector component add/remove UI

**Background**: You could edit components in the Inspector, but there was no way to attach a new component to the selected entity or to detach an unneeded one.

**Changed files**: `src/app.rs`, `src/ecs/world.rs`

**Added features**:

*Component list + remove (src/app.rs)*
- Show the selected entity's component list at the bottom of the Entities tab
- An "✕" button to the right of each component removes it immediately (except Transform — a required component)
- borrow workaround: store the click result in `to_remove: Option<String>`, perform the actual remove after the egui closure ends

*Add Component (src/app.rs)*
- Added a `component_factories: HashMap<String, Box<dyn Fn(&mut World, Entity) + Send + Sync>>` field
- Default registered components: `Sprite`, `RenderLayer`, `ParticleEmitter`, `Blackboard`, `Timer`, etc.
- Select from the registered component list with `egui::ComboBox` → add `Default` value via the "+ Add" button
- `App::register_component(name, factory)` public API allows registering user-defined components too

*register_reflect_named (src/ecs/world.rs)*
- Added `World::register_reflect_named::<T>(name)` — specify the name to be shown in the Inspector list

```rust
// 커스텀 컴포넌트 등록 예
app.register_component("Enemy", |world, entity| {
    world.add_component(entity, Enemy { hp: 100, speed: 80.0 });
});
```

---

### Phase 38a — Scene graph panel

**Background**: The Inspector showed the entity list only as a simple flat list, making the hierarchy hard to grasp, and entity names couldn't be edited.

**Changed file**: `src/app.rs`

**Added features**:
- Added a **"Scene"** tab (tab index 2) to the Inspector tab bar (native only)
- Hierarchy TreeView: list root entities without a `Parent` first, show indentation via DFS stack traversal based on `Children`
- Each node: show the name if it has a `Tag`, otherwise `"Entity {id}"`. Show a `▶` marker if it has children
- On click, update `SelectedEntity` (`selectable_label`)
- Inline Tag name editing at the bottom of the Scene tab (`text_edit_singleline`), "Add Name" button if no Tag
- Also added a selected-entity name editing field to the Entities tab (tab 0)

---

### Phase 38d — Rhai scripting API extension

**Background**: Rhai scripts had no way to create/delete entities, read/write AI state (Blackboard), or set steering behaviors.

**Changed files**: `src/scripting.rs`, `src/behavior.rs`

**Added features**:

*Commands API*
- `spawn_entity() -> i64` — schedule a spawn during script execution, processed with `world.spawn()` after execution. Returns a temporary handle (-1,-2,...)
- `despawn_entity(id: i64)` — add a positive ID to the despawn queue, processed after execution

*Blackboard API*
- Before execution, copy `Blackboard` values into an `Arc<Mutex<HashMap>>` snapshot → read from the script
- `bb_get_bool(key) / bb_get_float(key) / bb_get_int(key)` — read the snapshot, default if absent
- `bb_set_bool / bb_set_float / bb_set_int` — collect into a change buffer → reflect into `Blackboard` after execution (auto-added if absent)
- Added a `Blackboard::entries()` iterator (for snapshot collection)

*Steering API*
- `seek_target(tx, ty, speed)` — add/replace `Seek` + `SteeringVelocity`
- `flee_from(tx, ty, speed, radius)` — add/replace `Flee` + `SteeringVelocity`
- `stop_steering()` — `SteeringVelocity.velocity = Vec2::ZERO`

**Borrow workaround**: World can't be captured directly into a Rhai closure → snapshot before execution + collect into `Arc<Mutex<buffer>>` → reflect into World after execution

```rhai
// 스크립트 예시
let id = spawn_entity();
despawn_entity(old_id);
bb_set_bool("chasing", true);
let speed = bb_get_float("move_speed");
seek_target(player_x, player_y, speed);
```

---

### Phase 37a — Blackboard + Steering Behaviors

**Background**: The behavior tree (Phase 36) nodes had no way to share state with each other, and there was no standard steering logic for enemy AI to move toward the player.

**Changed files**: `src/behavior.rs`, `src/steering.rs` (new), `src/lib.rs`

**Added features**:

*Blackboard (src/behavior.rs)*
- `BlackboardValue { Bool / Float / Int / Vec2 / String }` enum
- `Blackboard` standalone ECS component — a key-value store based on `HashMap<String, BlackboardValue>`
- `set_bool / set_float / set_int / set_vec2 / set_string` / `get_bool / get_float / get_int / get_vec2 / get_string` methods
- Designed as a component separate from BehaviorTree → with the `take_component` pattern it's accessible via `world.get_mut::<Blackboard>(entity)` even while BehaviorTree is taken out
- 6 unit tests

*Steering Behaviors (src/steering.rs)*
- `SteeringVelocity { velocity: Vec2, max_speed: f32 }` — result-storing component
- `Seek { target: Vec2, max_speed: f32 }` — straight-line movement toward the target
- `Flee { target: Vec2, max_speed: f32, flee_radius: f32 }` — flee when approached within the radius
- `Arrive { target: Vec2, max_speed: f32, slow_radius: f32, stop_radius: f32 }` — decelerated settling
- `Wander { max_speed: f32, change_interval: f32 }` — deterministic pseudo-random wandering (no rand crate)
- `SteeringSystem`: compute `SteeringVelocity` in Seek → Flee → Arrive → Wander order, then apply the `Transform.position` movement
- 4 unit tests

```rust
// 스티어링 예
world.add_component(enemy, Seek { target: player_pos, max_speed: 120.0 });
world.add_component(enemy, SteeringVelocity::default());
app.add_system(SteeringSystem);

// Blackboard 예 (BehaviorNode 내부에서)
if let Some(bb) = world.get_mut::<Blackboard>(entity) {
    bb.set_bool("player_in_range", dist < 200.0);
}
```

---

### Phase 37d — CommandBuffer

**Background**: Creating/deleting entities or adding/removing components during system execution caused a double-borrow conflict with the query iterator. The `Commands` buffer accumulates commands lazily and applies them in a batch at end of frame.

**Changed files**: `src/ecs/commands.rs` (new), `src/ecs/mod.rs`, `src/ecs/world.rs`, `src/components.rs`

**Added features**:

*Commands (src/ecs/commands.rs)*
- Deferred-execution buffer based on `Vec<Box<dyn FnOnce(&mut World) + Send>>`
- `Commands::new()` / `Commands::default()`
- `spawn(f: impl FnOnce(&mut World, Entity) + Send)` — spawn + add components in a single closure
- `despawn(entity)` — schedule deletion (noop if already gone)
- `insert::<T>(entity, comp)` — schedule component addition
- `remove::<T>(entity)` — schedule component removal
- `Commands::apply(self, world)` / `World::apply_commands(cmds)` — order-preserving batch application
- 8 unit tests (spawn/despawn/insert/remove/ordering/multiple/noop × 2)

```rust
struct SpawnSystem;
impl System for SpawnSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let mut cmds = Commands::new();

        // 쿼리 루프 안에서 안전하게 스폰 예약
        for (e, _) in world.query::<Enemy>().map(|(e,c)|(e,c)).collect::<Vec<_>>() {
            cmds.despawn(e);
            cmds.spawn(|world, ne| {
                world.add_component(ne, Respawned);
            });
        }

        world.apply_commands(cmds); // 루프 끝난 뒤 일괄 적용
    }
}
```

---

### Phase 36 — Behavior tree

**Background**: A* pathfinding made it possible to compute movement paths, but the decision-making structure of enemy AI (chase/attack/idle transitions) had to be scattered across system code. A behavior tree lets you declare AI logic as a hierarchical, reusable node graph.

**Changed files**: `src/behavior.rs` (new), `src/ecs/world.rs`, `src/lib.rs`

**Added features**:

*Behavior tree (src/behavior.rs)*
- `BehaviorStatus { Running, Success, Failure }` — node execution result
- `BehaviorNode` trait: `tick(&mut self, world, entity, dt)` + `reset()` (optional implementation)
- `Sequence`: run children in order, abort immediately on the first Failure → Failure / all succeed → Success
- `Selector`: run children in order, abort immediately on the first Success → Success / all fail → Failure
- `Inverter`: invert Success ↔ Failure, keep Running
- `AlwaysSucceed`: ignore the child result and always return Success
- `BehaviorTree` component: a wrapper around the root `Box<dyn BehaviorNode>`
- `BehaviorSystem`: runs without double borrow using the `take_component → tick → add_component` pattern
- 8 tests

*World::take_component (src/ecs/world.rs)*
- A new API that takes a component out by ownership and returns it
- Replace with a placeholder (`Box<()>`) → clean up the archetype with `remove_component` → no double free

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

**Background**: There was no way to undo accidentally moving/deleting an entity in the Inspector, making the editor workflow inconvenient.

**Changed file**: only `src/app.rs`.

**Added features**:
- `EditorCmd` enum: `MoveEntity { entity, old_pos, new_pos }` / `CreateEntity { entity }` / `DeleteEntity { tag, transform, sprite }`
- `EditorHistory { undo: Vec, redo: Vec }`: `push` / `undo` / `redo` methods
- On **gizmo drag completion**, record `MoveEntity` (not recorded if there's no position change)
- **New Entity button** → record `CreateEntity`
- **Delete button** → capture a snapshot (tag/transform/sprite), then record `DeleteEntity`
- **Ctrl+Z** → undo, **Ctrl+Shift+Z** → redo (based on egui `ctx.input()`)
- Native only (`#[cfg(not(target_arch = "wasm32"))]`)

---

### Phase 34 — RenderLayer + sprite batching

**Background**: Even with hundreds of same-texture sprites, if they interleaved with other textures after z-sorting, draw calls were issued one by one. With no layer separation, there was also no way to guarantee the render order of background, game objects, and foreground.

**Changed files**: `src/components.rs`, `src/renderer/sprite.rs`, `src/lib.rs`

**Added features**:
- Added a `RenderLayer(i32)` component (optional, 0 if unspecified)
  - Lower values are drawn first (behind). Recommended: background=-1, gameplay=0, foreground/effects=1
- Sprite sort key: from `z` alone → `(layer, tex_key, z)`
  - The same `(layer, tex_key)` is always contiguous → guarantees 1 draw call per texture
  - The render order between different layers is always guaranteed
- `AtlasSprite` also reads `RenderLayer` the same way

**Trade-off**: Within the same layer, z-ordering between different textures is decided by the lexicographic order of texture keys. If exact interleaved z-ordering is needed, separate with `RenderLayer`.

---

### Phase 33 — A* pathfinding + ECS query filters

**Background**: Enemy AI had no means to move around obstacles, and ECS queries had no way to express "only entities that have/don't have a specific component," requiring manual filtering inside systems. The two features could be implemented in parallel without file conflicts, so two sub-agents worked on them simultaneously.

**Changed files**: `src/pathfinding.rs` (new), `src/ecs/world.rs`, `src/lib.rs`

**A* pathfinding (src/pathfinding.rs)**

- `PathGrid { width, height, cells: Vec<bool> }` — row-major grid
  - `new(w, h)` — all walkable
  - `new_blocked(w, h)` — all blocked
  - `set_walkable(x, y, bool)` / `is_walkable(x, y) -> bool` (out of range = false)
- `find_path(grid, start: IVec2, goal: IVec2) -> Option<Vec<IVec2>>`
  - 4-directional movement, Manhattan heuristic, `BinaryHeap` min-heap (reversed `Ord`)
  - The returned path excludes start, includes goal
  - `start == goal` → `Some(vec![goal])`, no path → `None`
  - If the goal is blocked, immediately `None` (without open-set search)
- 4 tests: straight, detour, blocked, same point

**ECS query filters (src/ecs/world.rs)**

- `World::query_with::<A, B>()` — returns `(Entity, &A)` for only entities that have **both** A and B
- `World::query_without::<A, B>()` — returns `(Entity, &A)` for only entities that have A but **not** B
- Implementation: judge `TypeId` inclusion (`arch.contains(tb)`) at the archetype level. More efficient than per-entity `get::<B>()` calls and consistent with the existing `query2` pattern.
- No marker types (`With<T>`, `Without<T>`) created — avoiding unnecessary abstraction
- Added 3 tests (16 total): With filter, Without filter, mixed 4-combination case

**Usage example**:
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

### Phase 32 — Runtime stability

**Background**: There was a magenta fallback on image load failure, but no way to know which handle failed. The `SceneDef` RON format had no version info, so there was no way to detect old files when the structure changed later. The Inspector had Save but no Load, leaving the editor workflow incomplete.

**Changed files**: `src/asset.rs`, `src/prefab.rs`, `src/app.rs`, `src/lib.rs`

**Added features**:

*AssetLoadState (src/asset.rs)*
- Added an `AssetLoadState { Loaded, Failed(String) }` enum
- Added an `image_load_states: HashMap<AssetId, AssetLoadState>` field to `AssetServer`
- Inside `load_image()`, call `decode_image_with_state()` — record the success/failure state together
- Hot reloading (`poll_reloads`) also updates the state after reload
- `AssetServer::load_state(&Handle<ImageAsset>) -> AssetLoadState` public API
- `AssetServer::failed_images() -> Vec<AssetId>` — list of failed handles (for debugging)
- Re-export `AssetLoadState` (`lib.rs`)

*SceneDef schema version (src/prefab.rs)*
- Added a `SCENE_DEF_VERSION: u32 = 1` constant
- Added a `#[serde(default)] pub version: u32` field to `SceneDef` — old files (no version) deserialize to 0, keeping backward compatibility
- Changed the `Default` implementation manually: initialize with `version: SCENE_DEF_VERSION`
- `SceneDef::load()` — after deserialization, on a version mismatch print `log::warn`; loading continues
- `SceneDef::save()` — always overwrite and save with `version: SCENE_DEF_VERSION`
- Re-export `SCENE_DEF_VERSION` (`lib.rs`)

*Inspector Load Scene (src/app.rs)*
- Added an `editor_load_status: Option<String>` field to `App`
- Added a `📂 Load Scene` button to the Inspector scene-save row (left of the Save button)
- On click: RON load → despawn all existing entities that have a `Transform` → call `spawn_scene_def`
- Store the success/failure message in `editor_load_status` and show it in the panel
- Reset `inspector_selected` (reset selection state after load)

**Architecture decisions**:
- On Load Scene, remove only entities that have a `Transform`. System entities such as physics bodies and the camera are left untouched to avoid conflicts.
- Including the error string inside `AssetLoadState::Failed` allows tracing the cause without `log::error`.

---

### Phase 31 — Asset browser

**Background**: There was no way to check the list of currently loaded image assets from the editor, and `AssetServer`'s internal `path_to_id` map was private, so it couldn't be queried externally.

**Changed files**: `src/asset.rs`, `src/app.rs`, `src/lib.rs`

**Added features**:
- Added an `ImageEntry { path, id, width, height }` struct
- `AssetServer::image_list() -> Vec<ImageEntry>` — return the list of currently loaded images
- Added `AssetServer::get_image_by_id(id: AssetId) -> Option<&ImageAsset>`
- Added an `inspector_tab: u8` (0=Entities, 1=Assets) field to the `App` struct
- Added tab buttons at the top of the Inspector panel; the "Assets" tab shows a grid of filenames/resolutions
- Re-export `ImageEntry` (`lib.rs`)

---

### Phase 28 — Editor scene save

**Background**: Phase 25-D made it possible to place entities with gizmos, but there was no way to save the placement result to a file.

**Changed file**: only `src/app.rs`.

**Added features**:
- Added `editor_save_path: String`, `editor_save_status: Option<String>` fields to the `App` struct
- Added a "Path:" text input + `💾 Save Scene` button at the bottom of the Inspector panel (gated with `#[cfg(not(target_arch = "wasm32"))]`)
- On button click, iterate all entities in the current world, collect entities that have `Tag`/`Transform`/`Sprite` into `EntityDef`s → call `SceneDef::save()`
- Show a result message (e.g. `✓ 5 entities → saved_scene.ron`) at the bottom of the panel
- Reset the save-status message on `reload_scene()`

---

## Previous session (Phase 23)

### Phase 23 — WASM build support

**Background**: `rapier2d`, `rodio`, `gilrs`, and `notify` use OS thread/file/HID APIs and don't compile for the `wasm32-unknown-unknown` target. Splitting them into platform-specific dependencies and gating the related code with `cfg` makes the WASM build pass.

**Verification**: `cargo build --target wasm32-unknown-unknown` — no warnings, no errors

**Changed files**: `Cargo.toml`, `.cargo/config.toml` (new), `src/lib.rs`, `src/app.rs`, `src/asset.rs`, `src/save.rs`, `src/input/gamepad.rs`

**Added files**: `examples/wasm/index.html`, `examples/wasm/build.sh`, `.cargo/config.toml`

**Cargo.toml changes**

| Category | Before | After |
|------|------|------|
| `wgpu` | `"22"` | `{ version = "22", features = ["webgl"] }` |
| `rapier2d`, `rodio`, `gilrs`, `notify`, `dirs` | `[dependencies]` | `[target.'cfg(not(wasm))'.dependencies]` |
| `wasm-bindgen`, `wasm-bindgen-futures`, `web-sys`, `console_error_panic_hook` | none | `[target.'cfg(wasm)'.dependencies]` |

**getrandom conflict resolution** (`getrandom 0.2` + `0.3` used simultaneously)
- `getrandom 0.2` — for `rand 0.8`, `js` feature
- `getrandom 0.3` — transitive dependency of wgpu etc., `wasm_js` feature (alias `getrandom3`)
- `.cargo/config.toml` — set `--cfg getrandom_backend="wasm_js"` RUSTFLAGS

**cfg-gate list**

| File | Change |
|------|-----------|
| `src/lib.rs` | conditional compilation of `pub mod physics`, `pub mod audio` + related re-exports |
| `src/lib.rs` | `#[wasm_bindgen(start)]` — `console_error_panic_hook` initialization |
| `src/app.rs` | `gilrs: Option<gilrs::Gilrs>` field + `poll_gilrs()` + `gilrs::Gilrs::new()` |
| `src/app.rs` | `run()` — WASM: `EventLoopExtWebSys::spawn_app(self)` |
| `src/app.rs` | `resumed()` — WASM: manual single-poll executor (using webgl synchronous completion) |
| `src/asset.rs` | `use notify::...` + `_watcher: Option<RecommendedWatcher>` + watch setup |
| `src/save.rs` | `save_path()` — WASM: return a relative path without `dirs` |
| `src/input/gamepad.rs` | `id_map`, `Slot::new`, `process_event`, `slot_mut`, `map_button`, `map_axis` |

**WASM runtime behavior**

| Feature | WASM |
|------|------|
| wgpu rendering (WebGL2) | works |
| ECS, UI, animation, tilemap | works |
| Physics (rapier2d) | disabled — `#[cfg(not(wasm))]` |
| Audio (rodio) | disabled — `#[cfg(not(wasm))]` |
| Gamepad (gilrs) | disabled — `#[cfg(not(wasm))]` |
| Filesystem asset loading | runtime error (std::fs unsupported) |
| Hot reloading | disabled — no notify |

**How to run in a browser**
```bash
# 의존성: cargo install wasm-pack
cd /path/to/skeleton-engine
bash examples/wasm/build.sh
python3 -m http.server 8080 --directory examples/wasm
# 브라우저에서 http://localhost:8080 열기
```

**Key design decisions**
- Gate the entire `physics` and `audio` modules with `#[cfg(not(wasm))]` in lib.rs → those files themselves need no modification
- WASM GPU init: the wgpu webgl backend completes adapter requests immediately (synchronously) on the first poll → works with simple manual polling without `pollster::block_on`
- The `GamepadState` struct exists on WASM too, but the fields/methods depending on gilrs types (`GamepadId`) are removed → it compiles as an empty state

---

## Previous session (Phase 22)

### Phase 22 — Reflect system

**Background**: A runtime field-access API was needed so the egui inspector could read and write component properties by name. Without a proc-macro, it was implemented manually on core components to keep the complexity low.

**Added file**: `src/reflect.rs`

**Changed files**: `src/components.rs`, `src/prefab.rs`, `src/ecs/world.rs`, `src/app.rs`, `src/lib.rs`

**New types**
- `ReflectValue` (`src/reflect.rs`) — `F32 | Vec2 | Bool | String | Color([f32;4])` enum
- `Reflect` trait (`src/reflect.rs`) — `fields()`, `set_field()`, `type_name()` interface
- `ReflectEntry` (`src/ecs/world.rs`) — a `Copy`-able pair of function pointers (`get`, `get_mut`)

**Component implementations**
- `Transform` — x, y, rotation, scale_x, scale_y, z (all F32)
- `Sprite` — color (Color), texture (String)
- `Tag` — tag (String)

**World extension** (`src/ecs/world.rs`)
- Added a `reflect_registry: HashMap<TypeId, ReflectEntry>` field
- `register_reflect::<T>()` — register function pointers per TypeId
- `get_reflect(entity, TypeId)` → `Option<&dyn Reflect>`
- `get_reflect_mut(entity, TypeId)` → `Option<&mut dyn Reflect>`
- `reflected_components(entity)` → `Vec<TypeId>` (the held subset among registered components)
- `is_alive(entity)` → `bool`

**egui Inspector panel** (`src/app.rs`)
- Added an `Inspector` window inside the F1 Debug UI (default position: [10, 130])
- Left: entity list (show Tag name if it has a Tag, otherwise "Entity N", select by click)
- Right: per-component collapsing panels for the selected entity + Grid-layout field editor
  - F32 → `DragValue` (slider speed 0.5)
  - Color → `color_edit_button_rgba_unmultiplied`
  - String → `text_edit_singleline`
- Editing uses the "stage-and-apply" pattern: read (immutable) → egui edit → write (mutable) — no borrow conflict

**Auto-registration**: `App::new()` + `App::reload_scene()` auto `register_reflect` for Transform, Sprite, Tag

**Key design decisions**
- Why `ReflectEntry` is `Copy`: it holds function pointers, so after `let entry = *map.get()?` (a copy), `&mut self.archetypes` can be borrowed
- Keeps object-safety: the `Reflect` trait has no generics/Self → `dyn Reflect` is usable
- `Vec2`, `Bool` are included in the ReflectValue enum — prepared in advance for user component extension

**Usage pattern**
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

## Previous session (Phase 21)

### Phase 21 — Texture Atlas system

**Background**: The renderer already uses GPU instancing, but a draw call is issued per texture. Bundling multiple sprites into a single atlas texture minimizes draw calls.

**Added file**: `src/atlas.rs`

**Changed files**: `src/asset.rs`, `src/renderer/sprite.rs`, `src/app.rs`, `src/lib.rs`

**New types**
- `TextureAtlas` (`src/atlas.rs`) — image handle + cols/rows grid info. `uv_rect(index)` → computes a `UvRect`
- `AtlasSprite` (`src/atlas.rs`) — `Handle<TextureAtlas>` + index + color. Used together with Transform

**AssetServer extension** (`src/asset.rs`)
- Added `atlases: HashMap<AssetId, TextureAtlas>` + `atlas_path_to_id`
- `load_atlas(path, cols, rows) → Handle<TextureAtlas>` — returns the cache on re-call with the same path
- `get_atlas(handle) → Option<&TextureAtlas>` — used by the renderer internally for UV computation

**App extension** (`src/app.rs`)
- `load_atlas(path, cols, rows) → Handle<TextureAtlas>` — also loads the GPU texture by adding to `pending_textures`

**Renderer extension** (`src/renderer/sprite.rs`)
- `AtlasSprite` query → `AssetServer::get_atlas()` → `uv_rect()` → added to the existing `sprites` Vec
- Uses the same z-sort + texture-group draw-call flow as the existing Sprite (backward compatible)
- If an `AtlasSprite` entity also has a `UvRect` component, that value is used instead of the grid UV. Non-uniform crop, vertical flip, and packed-atlas correction are handled by this override.

**Usage pattern**
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

**Key design decisions**
- `AtlasSprite` entities using the same atlas texture become **1 draw call** when placed contiguously after z-sort
- The atlas image path is the texture cache key, so it can be shared with the existing `Sprite(texture: path)` path
- The `atlases` map is a one-way `path → AtlasId` cache — calling with different cols/rows for the same path uses the first setting
- Rather than adding new fields to the public `AtlasSprite` struct, a `UvRect` component override is used to keep compatibility with existing literal initialization.

**Follow-up API extensions**
- `UvRect::new(...)`, `UvRect::from_pixels(...)`, `flipped_x()`, `flipped_y()` — custom crop/UV orientation helper
- `Sprite::textured_with_handle(path, Option<Handle<ImageAsset>>)` — runtime handle first, with a path fallback for tests/small worlds
- `DrawImage`, `UiImageQueue` — screen-space textured UI primitive. Coordinates are in logical viewport pixels like `DrawRect`, and it renders after the sprite pass / before the text pass.

---

## Previous session (Phase 20)

### Phase 20 — Animation blending

**Background**: `AnimationPlayer` supported only instant switching between clips. Adding crossfade and parameter-based clip selection enables smooth animation transitions.

**Added files**: `src/animation/blend_tree.rs`, `src/animation/blend_system.rs`
**Changed files**: `src/animation/player.rs`, `src/animation/system.rs`, `src/animation/mod.rs`, `src/lib.rs`

#### Main types

| Type | Role |
|------|------|
| `BlendWeight` | Crossfade progress (0.0→1.0) component. `AnimationSystem` updates it every frame |
| `BlendTree1D` | float parameter → automatic clip selection + crossfade component |
| `BlendEntry` | An item of BlendTree1D (threshold, clip_index) |
| `BlendTreeSystem` | A system that reads BlendTree1D and directs clip switching on AnimationPlayer |

#### Crossfade API

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

#### 1D blend tree API

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

#### Registration order

```rust
app.add_system(Box::new(BlendTreeSystem));   // 클립 선택
app.add_system(Box::new(AnimationSystem));   // 프레임 진행 + BlendWeight 갱신
app.add_system(Box::new(StateMachineSystem)); // 상태 머신 (기존)
```

#### How crossfade works

| Progress | Output UV | Description |
|--------|---------|------|
| 0.0 ~ below 0.5 | from_clip current frame | keep showing the previous clip |
| 0.5 and above ~ 1.0 | to_clip current frame | switch to the new clip |
| done (elapsed ≥ duration) | to_clip frame | release crossfade, normal playback |

Both clips keep advancing regardless of progress, so at the moment of UV switching the to_clip is naturally already playing ahead.

---

## Previous session (Phase 19)

### Phase 19 — Rhai scripting

**Background**: This lets game logic be written in `.rhai` scripts without recompiling Rust. Attaching a `ScriptRunner` to each entity runs `on_update(dt)` every frame and automatically syncs the Transform.

**Added file**: `src/scripting.rs`
**Changed files**: `Cargo.toml`, `src/asset.rs`, `src/app.rs`, `src/lib.rs`

#### Main types

| Type | Role |
|------|------|
| `ScriptAsset` | CPU-side Rhai AST + source string (managed by AssetServer) |
| `ScriptRunner` | An entity component; holds the script handle + Scope |
| `ScriptingSystem` | Runs `on_update(dt)` every frame + Transform sync |

#### Public API

```rust
// 스크립트 로드
let handle = app.load_script("assets/enemy_ai.rhai");

// 엔티티에 부착
world.add_component(entity, ScriptRunner::new(handle));

// 시스템 등록
app.add_system(Box::new(ScriptingSystem::new()));
```

**Script example (`enemy_ai.rhai`)**:
```rhai
fn on_start() {
    log("AI 초기화");
}

fn on_update(dt) {
    x += 100.0 * dt;   // 오른쪽 이동
    rot += 2.0 * dt;   // 회전
}
```

#### Scope variables (read/write)

| Variable | Type | Description |
|------|------|------|
| `x`, `y` | `f64` | Transform.position |
| `rot` | `f64` | Transform.rotation (radians) |
| `sx`, `sy` | `f64` | Transform.scale |

#### Registered functions

| Function | Description |
|------|------|
| `log(msg)` | debug output (`[Script] msg`) |

#### Design decisions

- `ScriptingSystem` owns the `Engine` directly — kept simple without a `ScriptEngine` resource
- Missing `on_start` / `on_update` are ignored without error (handling `EvalAltResult::ErrorFunctionNotFound`)
- Hot reloading: when `poll_reloads` detects a `.rhai` file change, the AST is recompiled. Calling `runner.reset()` re-runs `on_start`
- A `max_operations = 1_000_000` limit prevents infinite loops in scripts
- `rhai = { features = ["sync"] }` — makes the Engine `Send+Sync` to support future multithreaded extension

#### Cargo.toml change

```toml
rhai = { version = "1", features = ["sync"] }
```

---

## Previous session (Phase 18)

### Phase 18 — egui in-game debug editor

**Background**: During development, entity/component state couldn't be checked in real time. Integrating egui lets you freely add in-game overlay panels from within a `System`.

**Added file**: `src/debug_ui.rs`
**Changed files**: `Cargo.toml`, `src/app.rs`, `src/lib.rs`, `src/ecs/world.rs`, `src/asset.rs`

#### Main types

| Type | Role |
|------|------|
| `DebugUi` | ECS Resource; holds the egui Context, enabled toggle |

#### Public API

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

#### Render architecture

- scene → (post-process) → **egui overlay** → present
- egui is rendered with a separate `CommandEncoder` to keep lifetimes separate from the scene encoder
- Because egui-wgpu 0.29's `PaintCallbackFn` requires a `&mut RenderPass<'static>` by design, the `egui_render_pass()` helper function uses `unsafe transmute` (safe when no paint callback is registered)

#### Cargo.toml changes

```toml
egui = "0.29"
egui-wgpu = "0.29"   # wgpu 22 호환
egui-winit = { version = "0.29", default-features = false }  # clipboard 제외 (macOS objc2 충돌)
```

#### Design decisions

- Disable `egui-winit`'s clipboard feature: it causes a version conflict with `objc2-app-kit 0.3.2` on macOS
- The F1 toggle is handled separately from InputState, before egui_state event processing
- `DebugUi::ctx()` is valid only between `begin_pass`/`end_pass`; the engine manages it automatically in update()

---

## Work this session (Phase 17)

### Phase 17 — Asset pipeline + hot reloading

**Background**: Reference textures via a type-safe `Handle<T>` instead of a string path, and automatically re-upload the GPU texture when the file changes at runtime.

**Added file**: `src/asset.rs`
**Changed files**: `Cargo.toml`, `src/components.rs`, `src/lib.rs`, `src/app.rs`, `src/renderer/sprite.rs`, `src/particle.rs`

#### Main types

| Type | Role |
|------|------|
| `AssetId` | `u64` global monotonically increasing ID |
| `Handle<T>` | typed asset reference (Clone O(1), holds id + Arc<str> path) |
| `ImageAsset` | CPU-side RGBA8 image data (Arc<Vec<u8>> + size) |
| `AssetServer` | asset load/cache/file-watch/hot-reload resource |

#### Public API

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

Added an `image_handle: Option<Handle<ImageAsset>>` field to the `Sprite` struct.

- `#[serde(skip)]` — no effect on RON serialization (existing scene files still usable)
- Literal `Sprite { texture: None, color: ... }` initialization code needs to add `image_handle: None`
- The `Sprite::colored()`, `Sprite::textured()`, `Sprite::with_handle()` constructors are all safe

#### Hot-reload behavior

1. On `App::new()`, create `AssetServer::new()` → insert as a World resource
2. `notify::recommended_watcher` watches for file changes on a background thread
3. `App::update()` every frame: `AssetServer::poll_reloads()` → receive changed paths
4. Call `SpriteRenderer::reload_texture(path)` → update the GPU texture

#### Cargo.toml change

- `notify = "6"` — cross-platform file watching (macOS FSEvents, Linux inotify, Windows ReadDirectoryChanges)

#### Design decisions

- **Path embedded in Handle**: `Handle<T>` holds an `Arc<str>` path so the renderer can look up the GPU texture without the AssetServer.
- **Coexists with the existing `texture` path**: if `image_handle` exists it takes precedence, otherwise the `texture` string path is used as-is — no migration of existing code needed.
- **Graceful degradation on watch failure**: even if `notify` init fails (sandbox, etc.), load/cache works normally; only hot reloading is disabled.

---

## Previous session (Phase 16)

### Phase 16 — Scene serialization + prefab system

**Background**: This lays the groundwork to save/load an entire level with a single RON file and reuse a single-entity template (prefab).

**Added file**: `src/prefab.rs`
**Changed files**: `Cargo.toml`, `src/components.rs`, `src/lib.rs`

#### Main types

| Type | Role |
|------|------|
| `Tag` | string component for entity identification (Serialize/Deserialize supported) |
| `EntityDef` | a serializable struct describing one entity (tag, transform, sprite optional fields) |
| `SceneDef` | a `Vec<EntityDef>` wrapper — one RON file = one level |
| `Prefab` | a single template that saves/loads/spawns an `EntityDef` to/from a file |

#### Public functions

```rust
spawn_entity_def(world, &EntityDef) -> Entity
spawn_scene_def(world, &SceneDef)   -> Vec<Entity>
SceneDef::save(&self, path)         -> Result<(), SaveError>
SceneDef::load(path)                -> Result<SceneDef, SaveError>
Prefab::save(&self, path)           -> Result<(), SaveError>
Prefab::load(path)                  -> Result<Prefab, SaveError>
Prefab::spawn(&self, world)         -> Entity
```

#### Scene file format (RON example)

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

#### Cargo.toml change

- `glam = { version = "0.28", features = ["serde"] }` — added Vec2 serde support

#### Design decisions

- **Statically typed EntityDef**: supports only Transform + Sprite. A dynamic component registry is considered after Phase 17.
- **Reuse save.rs**: scene/prefab serialization is built on the existing `save()` / `load()` infrastructure.
- **Separate Tag component**: a dedicated component to distinguish roles like "player", "enemy" by query after scene load.

---

## Previous session (Phase 15)

### Phase 15 — Gamepad + UI Slider/CheckBox

**Background**: The goal was to complete the input layer, which previously supported only keyboard/mouse, and add slider/checkbox UI widgets to gain the ability to build settings screens.

**Added files**: `src/input/gamepad.rs`, `src/ui/slider.rs`, `src/ui/checkbox.rs`
**Changed files**: `Cargo.toml`, `src/input/mod.rs`, `src/app.rs`, `src/ui/mod.rs`, `src/ui/system.rs`, `src/lib.rs`

#### Gamepad input (gilrs 0.10)

| Type | Role |
|------|------|
| `GamepadState` | ECS resource. Up to 4 pad slots, tracks button/axis state |
| `GamepadButton` | South/East/North/West/LeftBumper/RightBumper/… 16 variants |
| `GamepadAxis` | LeftStickX/Y, RightStickX/Y, LeftTrigger, RightTrigger, DPadX/Y |

```rust
// 슬롯 0 (첫 번째 연결 패드)
if let Some(gs) = world.resource::<GamepadState>() {
    if gs.just_pressed(0, GamepadButton::South) { /* 점프 */ }
    let lx = gs.axis(0, GamepadAxis::LeftStickX);
}
```

- `App::new()` auto-inserts `GamepadState::default()`
- gilrs events are polled in `about_to_wait` → `flush()` at the end of `update()`
- Dynamic slot allocation/release via `Connected` / `Disconnected` gilrs events

#### UI Slider

```rust
let e = world.spawn();
world.insert(e, UiNode::new(100.0, 300.0, 200.0, 20.0));
world.insert(e, Slider::new(0.0, 100.0, 50.0));
// UiEvent::SliderChanged(entity, new_value) 로 변경 통보
```

- Change the value by clicking the track or dragging the thumb
- Color customization: `track_color`, `fill_color`, `thumb_color`, `thumb_hovered_color`

#### UI CheckBox

```rust
let e = world.spawn();
world.insert(e, UiNode::new(50.0, 200.0, 160.0, 24.0));
world.insert(e, CheckBox::new("사운드 켜기"));
// UiEvent::CheckBoxToggled(entity, checked) 로 토글 통보
```

#### UiEvent extension

Added `SliderChanged(Entity, f32)`, `CheckBoxToggled(Entity, bool)` (5 → 7 variants).

---

## Work this session (Phase 14)

### Phase 14 — Animation state machine

**Background**: Switching clips only via `AnimationPlayer.play(clip_index)` forced game logic to manage animation indices directly. As character states (idle/run/jump/attack) multiply, conditional branching explodes, so it was necessary to separate transition rules declaratively with a state machine.

**Added file**: `src/animation/state_machine.rs`
**Changed files**: `src/animation/mod.rs`, `src/animation/player.rs`, `src/lib.rs`

#### New types

| Type | Role |
|------|------|
| `AnimationStateMachine` | a state-machine component attached to an entity |
| `AnimState` | clip index + list of transition edges |
| `AnimTransition` | target state + list of AND conditions |
| `TransitionCond` | `BoolEq` / `FloatGt` / `FloatLt` / `Trigger` / `AnimationEnd` |
| `AnimParam` | `Bool(bool)` / `Float(f32)` / `Trigger(bool)` |
| `StateMachineSystem` | evaluates transitions every frame → calls `AnimationPlayer.play()` |

#### Usage pattern

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

#### System registration order

```rust
app.add_system(Box::new(AnimationSystem));     // 프레임 진행 + UvRect 동기화
app.add_system(Box::new(StateMachineSystem));  // 전환 조건 평가 → play() 호출
```

`StateMachineSystem` must run **after** `AnimationSystem` so that the `is_finished()` decision is reflected in the same frame.

#### Trigger consumption rule

A trigger is consumed every time `StateMachineSystem` runs (regardless of whether a transition happens). So `fire_trigger()` must be called within one frame, and activating it in a state with no transition condition discards it within that frame.

#### `AnimationPlayer` change

Added an `is_finished() -> bool` method — `true` when it's the last frame of a non-looping clip. The basis of the `AnimationEnd` condition.

---

## Work this session (Phase 13)

### Phase 13 — Physics raycast + character controller

**Background**: There was no raycast for line-of-sight checks, mouse picking, gun impact calculation, etc., and game-specific character movement including slope/stair handling was needed.

**Added file**: `src/physics/character.rs`
**Changed files**: `src/physics/world.rs`, `src/physics/mod.rs`, `src/lib.rs`

#### Raycast (`PhysicsWorld`)

```rust
// 단순 레이캐스트 — 최초 충돌 콜라이더 핸들 + toi
let result: Option<(ColliderHandle, f32)> =
    physics.cast_ray(origin_physics, dir, max_toi, solid);

// 법선 포함 — RaycastHit { collider_handle, point, normal, toi }
let hit: Option<RaycastHit> =
    physics.cast_ray_with_normal(origin_physics, dir, max_toi, solid);
```

- All coordinates are in **physics units** (pixels ÷ pixels_per_unit).
- Must be called after `step()`, once `query_pipeline` has been updated, so the latest state is reflected.

#### Kinematic bodies

```rust
// 중력 비반응, 수동 위치 제어
let (rb, col) = physics.add_kinematic_box(pos / PPU, half_w, half_h);
let (rb, col) = physics.add_kinematic_circle(pos / PPU, radius);
```

#### Character controller (`CharacterController` component)

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

**Structural notes**
- `CharacterController::inner`'s `up = -Y` — set to match the engine's screen coordinates (Y+ is down).
  Using Rapier's default (+Y) as-is would flip the floor/ceiling decision.
- `move_character()` internally calls `set_next_kinematic_translation()`, so
  the position is actually reflected on the next `step()`.
- A dedicated system handling character movement must be registered before `PhysicsSystem::run()` to work in the correct order.

**New tests** (`src/physics/world.rs`): 7
- `cast_ray_hits_static_box` — confirms a ray hits a static box
- `cast_ray_misses_when_no_obstacle` — None when there's no obstacle
- `cast_ray_with_normal_returns_correct_normal` — verifies the normal direction
- `add_kinematic_box_creates_body` — creates a kinematic body
- `add_kinematic_circle_creates_body` — creates a kinematic circle body
- `move_character_grounded_on_floor` — grounded decision on the floor
- `character_controller_builder_methods` — builder-method parameter setting

**Verification**: `cargo test` — all 61 unit + 11 doc tests pass (no effect on `rust-survivors` build)

---

## Work this session (Phase 12)

### Phase 12 — Transform hierarchy (Parent · Children · GlobalTransform)

**Background**: Transform dependencies between entities — weapon attachment, composite character setups, etc. — were needed, but the existing `Transform` was a flat structure storing only an independent local value.

**Added file**: `src/hierarchy.rs`

**New components/types**
- `Parent(Entity)` — a component pointing to the parent entity
- `Children(Vec<Entity>)` — list of child entities (held on the parent side)
- `GlobalTransform { position, scale, rotation, z }` — world-space transform computed by HierarchySystem every frame (`Copy`)
- `HierarchySystem` — a `System` implementation. `App` runs it automatically right after user systems (no registration needed)
- `attach(world, child, parent)` — helper that manages Parent + Children together
- `detach(world, child)` — helper that detaches the parent link

**Renderer integration** (`src/renderer/sprite.rs`)
- Added `InstanceRaw::from_global()`
- `render()` loop: use `GlobalTransform` first if present, fall back to `Transform` if not → **fully backward compatible**

**App auto-run** (`src/app.rs`)
```
유저 시스템(물리 포함) → HierarchySystem → 이벤트 flush → 렌더
```
Because hierarchy propagation runs right after physics updates `Transform.position`, an accurate world transform is always guaranteed.

**Depth limit**: the internal 2-pass structure supports up to 3 levels (root → child → grandchild).

**Usage pattern**
```rust
use engine::{attach, Transform};
use glam::Vec2;

// 무기를 플레이어에 부착
attach(&mut world, weapon, player);

// 로컬 오프셋 설정 — GlobalTransform은 매 프레임 자동 계산
world.get_mut::<Transform>(weapon).unwrap().position = Vec2::new(30.0, 0.0);
// → weapon의 GlobalTransform.position = player.position + (30, 0) rotated by player.rotation
```

**Verification record at the time**: `cargo build` + `cargo test` (skeleton-engine, rust-survivors 96 tests passing)

---

## Work this session (Phase 9~11)

### Phase 9 — ECS Archetype storage

**Background**: The existing ECS used a `HashMap<TypeId, Vec<Option<Box<dyn Any>>>>` structure, so as entity count grew, a `None` check occurred in every query loop.

**Change**: full rewrite of `src/ecs/world.rs` — Archetype-based dense column storage.
- Internal `Archetype` structure: `type_set: Vec<TypeId>` (sorted) + `entities: Vec<Entity>` + `columns: HashMap<TypeId, Vec<Box<dyn Any>>>`
- Entities with the same component set gather in the same Archetype, so no `None` check is needed during queries
- On `add_component` / `remove_component`, move between Archetypes via the `move_entity()` helper (swap_remove + position-map update)
- Full public API compatibility kept: `spawn`, `despawn`, `get`, `get_mut`, `query1~4`, `query_opt2`, `entities()`, resource methods
- Added 2 new tests: `archetype_reuse_across_entities`, `add_component_replaces_existing` (14 total)

**Architecture decision**: keep the `entities: Vec<Entity>` auxiliary field to preserve the `entities() -> &[Entity]` signature unchanged.

### Phase 10 — Post-processing

**Added files**
- `src/renderer/post_process.rs`: `PostProcessConfig` + `PostProcessRenderer`
- `src/renderer/shaders/post_process.wgsl`: vignette/chromatic-aberration/approximate-bloom WGSL shader

**Structure**
1. Insert the `PostProcessConfig` resource into World and set `enabled: true`
2. `App::render()` renders the entire scene to an intermediate texture (`target_view`)
3. Post-process pass: intermediate texture → swapchain (full-screen triangle, no vertex buffer needed)

**Effect descriptions**
- **Vignette**: dark screen edges (`vignette_strength`, `vignette_radius`)
- **Chromatic aberration**: sample RGB channels at radially different UVs (`chroma_offset`)
- **Approximate bloom**: 4-tap threshold sampling to bleed bright areas (`bloom_threshold`, `bloom_intensity`)

**Usage pattern**
```rust
app.world.insert_resource(PostProcessConfig {
    enabled: true,
    vignette_strength: 0.5,
    chroma_offset: 0.003,
    bloom_intensity: 0.4,
    ..Default::default()
});
```

**Note**: if the resource is absent or `enabled: false`, the intermediate-texture pass is fully skipped (zero overhead).

### Phase 11 — Audio enhancements

**Changed file**: `src/audio.rs` (fully compatible with the existing API)

**Added features**

#### Spatial audio
```rust
// 1회성 위치 재생
am.play_at("sfx", "boom.wav", false, source_pos, listener_pos, 500.0);

// 움직이는 소리 발생원 — 매 프레임 호출
am.update_position("sfx", enemy_pos, player_pos, 500.0);
```
- `(volume, pan)` = distance linear attenuation + automatic X-direction stereo pan

#### Audio bus mixer
```rust
am.assign_bus("bgm",      "music");
am.assign_bus("sfx_jump", "sfx");
am.set_bus_volume("music", 0.5);   // 음악 전체 절반으로
am.set_bus_volume("sfx",   0.8);   // 효과음 전체 80%
```

#### Fade
```rust
am.play_fade_in("bgm", "music.ogg", true, 2.0);  // 2초 페이드인
am.fade_out("bgm", 3.0);                          // 3초 페이드아웃 후 정지
am.fade_volume("sfx", 0.3, 1.5);                  // 1.5초 동안 0.3으로

// System::run() 내에서 매 프레임 호출 필수
world.resource_mut::<AudioManager>().map(|am| am.update(dt));
```

#### Channel playback state
```rust
match am.playback_state("bgm") {
    AudioChannelState::Missing => { /* never played, failed, or stopped */ }
    AudioChannelState::Playing => { /* still queued */ }
    AudioChannelState::Finished => { /* non-looping sink drained naturally */ }
}

if am.is_finished("bgm") == Some(true) {
    am.play("bgm", "next_track.mp3", false);
}
```
- `stop(channel)` removes the sink, so the state becomes `Missing`.
- Natural completion keeps the sink queryable as `Finished` until the channel is stopped or reused.
- Native builds enable MP3, OGG/Vorbis, and WAV decoding through `rodio`.

**Tests**: spatial-audio parameter computations, audio-effect defaults, playback-state helper mapping, and guarded live `play_tone` drain behavior when an audio device exists.

---

## Work this session (Phase 8)

### Save/Load completion

**Added functions**
- `load_or_default<T: DeserializeOwned + Default>(path)` — returns `Default::default()` if the file is missing, propagates parse errors as-is
- `exists(path) -> bool` — check whether the save file exists
- `delete(path) -> Result<(), SaveError>` — delete the save file (Ok if absent)

**lib.rs re-export additions**: exposed `save`, `load`, `load_or_default`, `exists`, `delete`, `save_path`, `SaveError` at the top level

**Test additions**: `load_or_default_returns_default_when_missing`, `load_or_default_returns_saved_value`, `exists_and_delete` (5 total → all passing)

**Usage pattern**
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

## Previous session (Phase 7)

### Physics collision events — ECS bridging

**Background**: `PhysicsPipeline::step()` fixed the contact handler to `&()` (no-op), so collision start/stop couldn't be detected from game logic.

**Implementation approach**: chose `NarrowPhase` polling rather than implementing the Rapier `EventHandler` trait. After `step()`, iterate `narrow_phase.contact_pairs()` and diff against the previous frame's contact set → send `Events<CollisionEvent>`. No `Mutex`/`RefCell` needed, consistent with the existing `has_contact()` pattern.

**Added files/changes**
- `src/physics/events.rs` (new): `CollisionEvent { Started(Entity, Entity), Stopped(Entity, Entity) }` — `Copy + Clone`
- `src/physics/system.rs`: `active_contacts: HashSet<(ColliderHandle, ColliderHandle)>` field, diff block inside `run()`
- `src/physics/mod.rs`: `pub mod events` + `CollisionEvent` re-export
- `src/lib.rs`: top-level `CollisionEvent` re-export

**Usage pattern**
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

**Note**: collisions with static colliders (floors, etc.) that have no `PhysicsBody` in the ECS are silently skipped due to `col_map.get()` failure. No panic even when the event isn't registered (`resource_mut` → `None` guard).

---

## Previous session (Phase 6)

### UI system enhancements

**TextInput** (`src/ui/text_input.rs`)
- Compose a text input field with a `UiNode` + `TextInput` entity
- UTF-8 byte-index-based cursor (`backspace()` is multi-byte safe)
- Cursor blink: dt accumulation, toggle every 0.5 s
- Events: `TextChanged`, `TextSubmitted`, `TextFocused`, `TextBlurred`

**ScrollView** (`src/ui/scroll_view.rs`)
- Compose a scroll list with a `UiNode` + `ScrollView` entity
- Render the internal `items: Vec<String>` directly without child entities
- Scroll with the mouse wheel when the cursor is over the widget
- `clamp_scroll(view_height)` — prevent out-of-range

**Panel + LayoutSystem** (`src/ui/panel.rs`)
- `UiNode` + `Panel` entity: automatically arrange child entities (`Vertical` / `Horizontal`)
- `LayoutSystem`: runs before UiSystem — recompute child `UiNode.offset` into absolute screen coordinates
- Required registration order: `add_system(Box::new(LayoutSystem))` → `add_system(Box::new(UiSystem))`

**InputState character buffer** (`src/input/state.rs`)
- Added a `text_input_chars: Vec<char>` field
- `text_chars() -> &[char]` public read / `push_char`, `push_backspace`, `push_enter` (pub(crate))
- In `app.rs`, extract characters from `logical_key` → record into the buffer (sentinels: `'\x08'` = Backspace, `'\n'` = Enter)

**UiEvent extension**
- Removed `Copy`, kept `Clone` (needed to hold String)
- Preserved the existing `ButtonClicked` + added `TextChanged`, `TextSubmitted`, `TextFocused`, `TextBlurred`

---

## Architecture decisions worth knowing

### Renderer separation
The renderer doesn't reference `AnimationPlayer` directly. `AnimationSystem` syncs the `UvRect` component, and the renderer reads only `UvRect`. A structure to prevent layer-boundary violations.

### DebugDrawQueue → UiQueue conversion
`DebugDrawQueue` holds pure data (`DebugRect`), and `App`'s render stage converts it to `DrawRect` and puts it in `UiQueue`. A design so the system layer doesn't depend on renderer types.

### PhysicsWorld encapsulation
The internal rapier2d fields are `pub(crate)`. From outside, use only the accessors `rigid_body()`, `rigid_body_mut()`, `get_collider()`, `get_collider_mut()`, `add_dynamic_circle()`, `remove_body()`.

### ECS borrow-conflict workaround
Due to the Rust borrow checker, you can't mix `get_mut` directly during a query. Standard pattern: first `.collect()` the entity list, then iterate calling `get_mut`.

### UI character input buffer (Phase 6~)
`InputState.text_chars()` — a slice of the characters input this frame. `UiSystem` consumes it, and it's cleared in `flush()`. Only the entity with a focused `TextInput` processes this buffer.

### LayoutSystem execution order (Phase 6~)
The positions of `Panel` children are computed by `LayoutSystem`. It must be registered before `UiSystem` to render at the correct positions.

---

## Completed Phase candidate records

| Phase | Feature | Difficulty | Notes |
|-------|------|--------|------|
| ~~Phase 13~~ | ~~Physics raycast + character controller~~ | — | done |
| ~~Phase 14~~ | ~~Animation state machine~~ | — | done |
| ~~Phase 15~~ | ~~Gamepad (gilrs) + UI Slider/CheckBox~~ | — | done |
| ~~Phase 16~~ | ~~Scene serialization + prefab system~~ | — | done |
| ~~Phase 17~~ | ~~Asset pipeline + hot reloading~~ | — | done |
| ~~Phase 18~~ | ~~egui in-game debug editor~~ | — | done |
| ~~Phase 19~~ | ~~Rhai scripting — ScriptAsset/ScriptRunner/ScriptingSystem~~ | — | done |
| ~~Phase 20~~ | ~~Animation blending — BlendWeight/play_with_crossfade/BlendTree1D~~ | — | done |
| ~~Phase 21~~ | ~~Texture Atlas — TextureAtlas/AtlasSprite/load_atlas~~ | — | done |
| ~~Phase 22~~ | ~~Reflect system — Reflect trait, ReflectValue, World::register_reflect/get_reflect~~ | — | done |
| ~~Phase 23~~ | ~~WASM build support — cfg-gate 4 dependencies, fs abstraction, entry-point branching~~ | — | done |
| ~~Phase 24~~ | ~~WASM browser run — force WebGL2, async GPU init, web-time~~ | — | done |
| ~~Phase 25~~ | ~~networking / ECS parallel / shader materials / editor gizmos / integration~~ | — | done |
| ~~Phase 26~~ | ~~LOD / culling — Camera::visible_rect, CullConfig, AABB frustum culling, min_pixel_size LOD~~ | — | done |
| ~~Phase 27~~ | ~~multiplayer demo — NetworkClient-based server-client roleplay example~~ | — | done |
| ~~Phase 28~~ | ~~editor scene save — serialize gizmo-placed entities to SceneDef RON~~ | — | done |
| ~~Phase 29~~ | ~~scene hierarchy serialization — EntityDef.parent, two-pass spawn, topological_sort_entities~~ | — | done |
| ~~Phase 30~~ | ~~system profiler — System::name(), ProfilerData, RenderStats, Engine Stats extension~~ | — | done |
| ~~Phase 31~~ | ~~asset browser — ImageEntry, image_list(), Inspector Assets tab~~ | — | done |
| ~~Phase 32~~ | ~~runtime stability — AssetLoadState, SceneDef.version, Load Scene button~~ | — | done |
| ~~Phase 33~~ | ~~A* pathfinding (PathGrid/find_path) + ECS query filters (query_with/query_without)~~ | — | done |
| ~~Phase 34~~ | ~~RenderLayer component + sprite batching (layer·tex·z sort)~~ | — | done |
| ~~Phase 35~~ | ~~Inspector Undo/Redo (Ctrl+Z/Shift+Z) — move/create/delete~~ | — | done |
| ~~Phase 36~~ | ~~behavior tree — BehaviorTree/BehaviorSystem, Sequence/Selector/Inverter~~ | — | done |
| ~~Phase 37a~~ | ~~Blackboard (standalone ECS component) + Steering Behaviors (Seek/Flee/Arrive/Wander)~~ | — | done |
| ~~Phase 37d~~ | ~~CommandBuffer — Commands::spawn/despawn/insert/remove + World::apply_commands~~ | — | done |
| ~~Phase 38a~~ | ~~scene graph panel — editor TreeView + Tag name editing~~ | — | done |
| ~~Phase 38d~~ | ~~Rhai scripting API extension — spawn/despawn/Blackboard/Steering~~ | — | done |
| ~~Phase 39b~~ | ~~Inspector component add/remove UI — factory pattern + ComboBox + ✕ button~~ | — | done |
| ~~Phase 39d~~ | ~~REFERENCE.html v0.38.0 — Steering/Blackboard/Commands/SceneGraph/Rhai documentation~~ | — | done |
| ~~Phase 40c~~ | ~~Gizmo Grid Snap — checkbox + grid-size DragValue + snap_to_grid helper~~ | — | done |
| ~~Phase 40d~~ | ~~REFERENCE.html v0.39.0 — component add/remove UI, register_component documentation~~ | — | done |
| ~~7 code-review items~~ | ~~Timeline NaN / TextureError / remove_resource / layer_mask / register_fn one-time / network backpressure / egui unsafe documentation~~ | — | done (`4084cee`) |
| ~~WASM build regression + unsafe cleanup~~ | ~~recover wasm32 build broken by missing `push_event_bounded` import in network.rs + remove unnecessary `unsafe impl Send/Sync` from BehaviorTree~~ | — | done (`af6fc59`) |
| ~~vision reset~~ | ~~established `docs/VISION.md` (forkable general-purpose 2D skeleton, breadth-first + example-verification loop), `docs/NEXT_WORK.md` candidate list~~ | — | done |
| ~~2D skeletal animation~~ | ~~cutout model: `src/skeletal.rs` + `SkeletonBuilder` + `examples/skeletal_puppet.rs`. Improved HierarchySystem with arbitrary-depth propagation~~ | — | done |

> **Current status**: entering the post-v1.0 breadth-expansion stage. Vision (`docs/VISION.md`) = a forkable general-purpose 2D skeleton, features verified with small playable examples. Next candidates are in `docs/NEXT_WORK.md`. Confirmed native·wasm32 builds, 264 tests, 0 clippy warnings.

---

## Related repositories

| Repository | Role | Path |
|---|---|---|
| `skeleton-engine` | engine core (this repository) | `/Users/jkl/Projects/skeleton-engine` |
| `rust-survivors` | a game project using the engine | `/Users/jkl/Projects/rust-survivors` |

The two repositories are developed **independently**. Engine improvements only in `skeleton-engine`, game logic only in `rust-survivors`.

---

## Reference documents

- `REFERENCE.html` — public API reference (with code examples)
- inline doc comments in each `src/` file — detailed implementation intent recorded
