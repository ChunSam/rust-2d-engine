# AGENTS.md — skeleton-engine agent reference

> Version v1.0.0 | package `skeleton-engine` | library crate `engine`
> wgpu-based Rust 2D game engine | Full API: `REFERENCE.html` | structure: `ARCHITECTURE.html` | dev history: `docs/HANDOFF.md`

## Project direction (read `docs/VISION.md`)

A **skeleton**: a hackable, MIT-licensed, genre-agnostic 2D engine meant to be forked
and extended. Core feature-work loop: **a new feature is not done until a small,
playable example game in `examples/` exercises it in real play**; if the API feels
awkward while writing that example, fix the API before release. Keep new code
fork-friendly (clear module boundaries, extension points). See `docs/VISION.md`.

## Module map

| Looking for | File |
| --- | --- |
| Engine entry point, main loop, render orchestration, `load_image` | `src/app.rs` |
| `Handle<T>`, `ImageAsset`, `ScriptAsset`, `AssetServer` | `src/asset.rs` |
| `TextureAtlas`, `AtlasSprite` | `src/atlas.rs` |
| `Reflect`, `ReflectValue` | `src/reflect.rs` |
| `ScriptAsset`, `ScriptRunner`, `ScriptingSystem` | `src/scripting.rs` |
| `DebugUi` | `src/debug_ui.rs` |
| Full public API re-export list | `src/lib.rs` |
| `Entity`, `Component`, `Resource`, `Query` | `src/ecs/world.rs` |
| Event bus `Events<E>` | `src/ecs/events.rs` |
| `System` trait | `src/ecs/system.rs` |
| Scene transitions: `Scene`, `SceneCmd`, `SceneChange` | `src/scene.rs` |
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
| `SkeletalAnimator`, `SkeletalClip`, `BoneTrack`, `SkeletonBuilder`, `SkeletalAnimationSystem` (2D cutout) | `src/skeletal.rs` (details: `docs/SKELETAL.md`) |
| UI: `UiNode`, `Button`, `Label`, `TextInput`, `ScrollView`, `Panel`, `LayoutSystem`, `UiEvent` | `src/ui/` |
| `Slider`, `CheckBox` | `src/ui/slider.rs`, `src/ui/checkbox.rs` |
| `Tag`, `EntityDef`, `SceneDef`, `Prefab`, prefab spawn functions | `src/prefab.rs` |
| `Timer`, `Tween`, `Easing` | `src/timer.rs`, `src/tween.rs` |
| `ParticleEmitter`, `ParticleSystem` | `src/particle.rs` |
| `Tilemap`, `TilemapAtlas`, `TilemapSystem` | `src/tilemap.rs` |
| `AudioManager` | `src/audio.rs` |
| save/load API, `SaveError` | `src/save.rs` |
| `PostProcessConfig`, `PostProcessRenderer` | `src/renderer/post_process.rs` |
| wgpu render pipeline | `src/renderer/` |

## Core architecture patterns

### ECS query API

```rust
// Single component
for (entity, comp) in world.query::<MyComp>() { ... }

// Multiple components: query2 / query3 / query4
for (e, a, b) in world.query2::<A, B>() { ... }

// A required, B optional
for (e, a, b_opt) in world.query_opt2::<A, B>() { ... }

// System signature
impl System for MySystem {
    fn run(&mut self, world: &mut World, dt: f32) { ... }
}
```

### Borrow checker workaround pattern

You cannot call `get_mut` on the same `World` while a query iterator is alive. Collect the entity list first, then mutate.

```rust
let entities: Vec<Entity> = world.query::<Foo>().map(|(e, _)| e).collect();
for entity in entities {
    world.get_mut::<Foo>(entity).unwrap().update();
}
```

### Render layer separation

- `AnimationSystem` → syncs the `UvRect` component → the renderer reads only `UvRect`
- The renderer referencing `AnimationPlayer` directly is a layer violation
- `DebugDrawQueue` holds pure data (`DebugRect`) only, converted to `DrawRect` in the `App` render stage
- Render order: Systems → Events flush → Input flush → Scene command handling → Render (sprites → UI → text)

### UI system registration order

When using `Panel`, register `LayoutSystem` before `UiSystem`.

```rust
app.add_system(Box::new(LayoutSystem)); // recomputes child UiNode.offset
app.add_system(Box::new(UiSystem));     // reads positions and renders
```

- `UiEvent` implements `Clone` but not `Copy`. `TextChanged`/`TextSubmitted` carry a `String`.
- `InputState::text_chars()` is this frame's char slice. `'\x08'` = Backspace, `'\n'` = Enter.

### Animation state machine registration order

Register `StateMachineSystem` after `AnimationSystem` so `is_finished()` is reflected in the same frame.

```rust
app.add_system(Box::new(AnimationSystem));    // frame advance + UvRect sync
app.add_system(Box::new(StateMachineSystem)); // evaluate transition conditions -> call play()
```

Access parameters inside a system via `world.get_mut::<AnimationStateMachine>(entity)`.

```rust
sm.set_bool("is_running", true); // for BoolEq conditions
sm.set_float("speed", 3.5);      // for FloatGt / FloatLt conditions
sm.fire_trigger("jump");         // for Trigger conditions, auto-consumed each frame
```

`TransitionCond::AnimationEnd` is true when a non-looping clip reaches its last frame.

### PhysicsWorld encapsulation

Internal rapier2d fields are `pub(crate)`, so do not access them directly from outside. Available accessors:

```text
rigid_body() / rigid_body_mut()
get_collider() / get_collider_mut()
add_dynamic_circle() / add_dynamic_box() / add_static_box()
remove_body()
```

## Common task patterns
| Task | Steps |
| --- | --- |
| New component | Define the struct in `src/components.rs` or the relevant module → re-export in `src/lib.rs` |
| New system | Implement the `System` trait → register via `app.add_system(Box::new(MySystem))` or in `Scene::on_enter` |
| New resource | Define the struct in `src/resources.rs` → register via `app.world.insert_resource(MyResource { ... })` → re-export if needed |

### Add a new event

```rust
// 1. Define the type: needs Clone + 'static
#[derive(Clone)]
struct MyEvent { pub data: f32 }

// 2. Register during App setup
app.register_event::<MyEvent>();

// 3. Use inside a system
world.resource_mut::<Events<MyEvent>>().unwrap().send(MyEvent { data: 1.0 });
for ev in world.resource::<Events<MyEvent>>().unwrap().read() { ... }
```

### Scene transitions

```rust
world.resource_mut::<SceneChange>().unwrap().0 =
    Some(SceneCmd::Replace(Box::new(MyScene)));

// SceneCmd::Push(Box::new(MyScene)) - push onto the stack
// SceneCmd::Pop                     - return to the previous scene
```

## Agent working notes

Follow `docs/AGENT_WORKFLOW.md` for detailed operating rules. `AGENTS.md` is a quick
reference, so keep it **under 200 lines**. If important content would exceed 200 lines,
create a new `docs/*.md` and leave only a summary and link here.

### Default flow

- Proceed in order: explore → scope → plan if needed → implement → verify → report summary.
- Locate symbols/keywords with `rg` before reading whole files; default reading order is `src/lib.rs` → module map → target file.
- Handle single-file edits with clear requirements directly in the main session.
- Use subagents freely for: exploring 3+ files, changing multiple subsystems, implementing after a long conversation, or work that benefits from parallel review.
- If public API/usage/examples are affected, check whether related docs need updating.
- Run default verification against the engine repo. Check `rust-survivors` only on user request or clear need.
- stage/commit/push only on user request.
- Confirm beforehand: public API removal/rename, dependency/version changes, large refactors, file deletion, destructive Git.
- Subagent prompts must include file paths, patterns to apply, expected behavior, and the do-not-change scope.

## Related projects
- `skeleton-engine`: `/Users/jkl/Projects/skeleton-engine` — engine core, this repo
- `rust-survivors`: `/Users/jkl/Projects/rust-survivors` — a game project using this engine under the crate name `engine`
- On breaking changes to the engine's public API, check the impact on the game side

## Documentation structure
Instruction files that agents must auto-detect live at the repo root. General docs are collected under `docs/`.

| Location | Purpose |
| --- | --- |
| `AGENTS.md` | Codex/agent shared quick reference: module map, core patterns, task checklists |
| `CLAUDE.md` | Quick reference for Claude-family agents |
| `README.md`, `REFERENCE.html`, `ARCHITECTURE.html`, `docs/HANDOFF.md` | Intro/usage, public API, engine structure, per-phase dev history |
| `docs/CHANGELOG.md`, `docs/ROADMAP.md` | Release change log, v1.0 historical roadmap |
| `docs/AGENT_WORKFLOW.md` | Detailed agent operating rules |
| ignored local docs | `.gitignore`d work prompts / personal plans. Not referenced as official docs |

**Documentation language**: Write doc prose in **English** to minimize token cost
(English ≈ ⅓ the tokens of equivalent Korean). Code, file paths, identifier tables, and
API names stay as written. Exceptions kept in Korean: the beginner glossary
(`docs/ENGINE_TERMS_FOR_BEGINNERS.md`) and personal/gitignored one-off prompt or plan
docs. New `docs/HANDOFF.md` entries are written in English. Prefer concision.

> **Growth strategy**: add new subsystem docs under `docs/` (e.g. `docs/SUBSYSTEM.md`)
> and add only a one-line reference to the module map here.
