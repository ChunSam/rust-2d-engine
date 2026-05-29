# CLAUDE.md — skeleton-engine agent reference

> Version v1.0.0 | package `skeleton-engine`, library crate `engine` | wgpu-based Rust 2D game engine  
> WASM support: `cargo build --target wasm32-unknown-unknown` passes  
> Full API: `REFERENCE.html` | dev history / architecture decisions: `docs/HANDOFF.md`

---

## Project direction (read `docs/VISION.md`)

This engine is a **skeleton**: a hackable, MIT-licensed, genre-agnostic 2D engine meant
to be forked and extended. Priorities — (1) open-source skeleton others can fork,
(2) personal foundation for 2D games, (3) learning vehicle.

When doing feature work, follow the core loop from `docs/VISION.md`:

- **A new feature is not done until a small, playable example game in `examples/`
  exercises it in real play.** The example is the acceptance test.
- **If the API feels awkward while writing that example, fix the API before release.**
- Keep new code fork-friendly: clear module boundaries and extension points. Breadth
  first, but not by leaving an unreadable mess.

---

## Module map

Where to read to find a given thing:

| Looking for | File |
|---------|------|
| Engine entry point, main loop, render orchestration, `load_image` | `src/app.rs` |
| Handle<T>, ImageAsset, ScriptAsset, AssetServer (asset load / caching / hot reload) | `src/asset.rs` |
| TextureAtlas (uniform grid atlas), AtlasSprite (atlas tile render component) | `src/atlas.rs` |
| Reflect trait, ReflectValue (runtime field read/write, egui Inspector integration) | `src/reflect.rs` |
| ScriptAsset, ScriptRunner, ScriptingSystem (Rhai scripting) | `src/scripting.rs` |
| DebugUi (egui overlay, F1 toggle, custom panels via `ctx()`) | `src/debug_ui.rs` |
| Full public API re-export list | `src/lib.rs` |
| Entity / Component / Resource / Query | `src/ecs/world.rs` |
| Event bus (`Events<E>`) | `src/ecs/events.rs` |
| `System` trait | `src/ecs/system.rs` |
| Scene transitions (Scene, SceneCmd, SceneChange) | `src/scene.rs` |
| Transform, Sprite | `src/components.rs` |
| WindowConfig, GameState, ShouldQuit, DebugDrawQueue | `src/resources.rs` |
| Camera (coordinate transforms, zoom) | `src/camera.rs` |
| InputState, InputMap | `src/input/` |
| GamepadState, GamepadButton, GamepadAxis | `src/input/gamepad.rs` |
| PhysicsWorld, PhysicsBody, PhysicsSystem, CollisionEvent | `src/physics/` |
| CharacterController, RaycastHit, cast_ray, cast_ray_with_normal, move_character | `src/physics/character.rs`, `src/physics/world.rs` |
| add_kinematic_box, add_kinematic_circle | `src/physics/world.rs` |
| SpatialGrid, Collider, CollisionLayer | `src/collision/` |
| AnimationPlayer, AnimationClip, AnimationSystem, BlendWeight | `src/animation/player.rs`, `src/animation/system.rs` |
| AnimationStateMachine, StateMachineSystem, TransitionCond, AnimParam | `src/animation/state_machine.rs` |
| BlendTree1D, BlendEntry, BlendTreeSystem (1D parameter-driven auto transitions + crossfade) | `src/animation/blend_tree.rs`, `src/animation/blend_system.rs` |
| SkeletalAnimator, SkeletalClip, BoneTrack, BoneKeyframe, SkeletalAnimationSystem, SkeletonBuilder (2D cutout skeletal animation) | `src/skeletal.rs` (details: `docs/SKELETAL.md`) |
| UI (UiNode, Button, Label, TextInput, ScrollView, Panel, LayoutSystem, UiEvent) | `src/ui/` |
| Slider (horizontal slider), CheckBox (toggle checkbox) | `src/ui/slider.rs`, `src/ui/checkbox.rs` |
| Tag, EntityDef, SceneDef, Prefab, spawn_entity_def, spawn_scene_def | `src/prefab.rs` |
| Timer, Tween, Easing | `src/timer.rs`, `src/tween.rs` |
| ParticleEmitter, ParticleSystem | `src/particle.rs` |
| Tilemap, TilemapAtlas, TilemapSystem | `src/tilemap.rs` |
| AudioManager (playback, positional audio, bus mixer, fades) | `src/audio.rs` |
| save / load / load_or_default / exists / delete / save_path / SaveError | `src/save.rs` |
| PostProcessConfig, PostProcessRenderer | `src/renderer/post_process.rs` |
| wgpu render pipeline (rarely edited directly) | `src/renderer/` |

---

## Core patterns & task recipes

Detailed in **`docs/PATTERNS.md`**:

- **Architecture patterns** — ECS query API (`query2`/`query_opt2`), borrow-checker
  workaround (collect entities then `get_mut`), render-layer separation
  (`AnimationSystem` → `UvRect` → renderer), UI system order (`LayoutSystem` before
  `UiSystem`), animation state-machine order (`StateMachineSystem` after
  `AnimationSystem`), `PhysicsWorld` encapsulation accessors.
- **Task recipes** — adding a component / system / resource / event, and scene transitions.

---

## Agent working notes

### Context management

The longer a session runs, the more accumulated context degrades response quality. Split the approach by task type:

| Situation | Recommended approach |
|------|-----------|
| Single-file edit (clear requirements) | Edit directly in the main session |
| Feature spanning multiple files | Split out into a Task subagent |
| Exploration needs 3+ files | Explore subagent |
| Writing code after a long conversation | Task subagent (avoid context pollution) |

### Efficient exploration

- Locate symbols/keywords with `grep` before reading whole files
- If the path is already known, use Read directly (no Explore subagent needed)
- Reading order: `src/lib.rs` → module map → narrow down to the target file

### Subagent prompt principles

A subagent starts without knowing the current conversation context. Always include in the prompt:

1. **Paths to edit** (absolute paths)
2. **Patterns to apply** — pass a summary of this file's core-pattern sections (borrow workaround, layer separation, etc.)
3. **Expected result** — what behavior should change

---

## Documentation rules

- **Language**: Write doc prose in **English** to minimize token cost (English ≈ ⅓ the
  tokens of equivalent Korean). Code, file paths, identifier tables, and API names stay
  as written.
- **Exceptions kept in Korean**: the beginner glossary (`docs/ENGINE_TERMS_FOR_BEGINNERS.md`)
  and personal/gitignored one-off prompt or plan docs.
- New `docs/HANDOFF.md` entries are written in English.
- **Length**: keep `CLAUDE.md` / `AGENTS.md` ≤200 lines. Prefer concision, but **do not
  drop needed content to hit the limit** — when trimming would risk losing information,
  move the detail into a new `docs/*.md` (e.g. `docs/PATTERNS.md`, `docs/SUBSYSTEM.md`)
  and leave a one-line reference here.

---

## Related projects

| Repo | Path | Role |
|--------|------|------|
| skeleton-engine | `/Users/jkl/Projects/skeleton-engine` | Engine core (this repo) |
| rust-survivors | `/Users/jkl/Projects/rust-survivors` | Game project that uses the engine |

`rust-survivors` consumes the `skeleton-engine` package under the crate name `engine`.
On breaking changes to the engine's public API, check the impact on the game side.

---

## Document map

| Document | Purpose |
|------|------|
| `CLAUDE.md` (this file) | Agent quick reference — module map, task checklists |
| `docs/PATTERNS.md` | Core architecture patterns + task recipes (extracted from this file) |
| `REFERENCE.html` | Full public API + code examples (detailed) |
| `docs/HANDOFF.md` | Per-phase dev history, background on architecture decisions |

> **Growth strategy**: when content would push this file past 200 lines, move detail into
> a `docs/*.md` (a new subsystem doc, or `docs/PATTERNS.md`) and leave only a one-line
> reference here. Never drop needed content just to stay under the limit.
