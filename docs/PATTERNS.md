# Core architecture & task patterns

Detailed engine patterns and task recipes, extracted from `CLAUDE.md` / `AGENTS.md` to
keep those quick-reference files under 200 lines. Both reference this document.

---

## Core architecture patterns

### ECS query API

```rust
// Single component
for (entity, comp) in world.query::<MyComp>() { ... }

// Multiple components (query2 / query3 / query4)
for (e, a, b) in world.query2::<A, B>() { ... }

// A required, B optional
for (e, a, b_opt) in world.query_opt2::<A, B>() { ... }

// System signature
impl System for MySystem {
    fn run(&mut self, world: &mut World, dt: f32) { ... }
}
```

### Borrow checker workaround pattern (required)

You cannot call `get_mut` on the same World while a query iterator is alive. Standard pattern:

```rust
// First collect the entity list, then iterate and get_mut
let entities: Vec<Entity> = world.query::<Foo>().map(|(e, _)| e).collect();
for entity in entities {
    world.get_mut::<Foo>(entity).unwrap().update();
}
```

### Render layer separation

- `AnimationSystem` → syncs the `UvRect` component → the renderer reads only `UvRect`  
  (the renderer referencing `AnimationPlayer` directly is a layer violation)
- `DebugDrawQueue` = pure data (`DebugRect`) → converted to `DrawRect` in the `App` render stage
- Render order: Systems → Events flush → Input flush → Scene command handling → Render (sprites → UI → text)

### UI system registration order

When using `Panel`, register `LayoutSystem` **before** `UiSystem`:

```rust
app.add_system(Box::new(LayoutSystem));  // recomputes child UiNode.offset
app.add_system(Box::new(UiSystem));      // reads positions and renders
```

`UiEvent` implements `Clone` but not `Copy` (TextChanged/TextSubmitted carry a String).  
`InputState::text_chars()` — this frame's char slice. `'\x08'`=Backspace, `'\n'`=Enter.

### Animation state machine registration order

Register `StateMachineSystem` **after** `AnimationSystem` so `is_finished()` is reflected in the same frame:

```rust
app.add_system(Box::new(AnimationSystem));     // frame advance + UvRect sync
app.add_system(Box::new(StateMachineSystem));  // evaluate transition conditions → call play()
```

Manipulate parameters inside a system via `world.get_mut::<AnimationStateMachine>(entity)`:

```rust
sm.set_bool("is_running", true);   // for BoolEq conditions
sm.set_float("speed", 3.5);        // for FloatGt / FloatLt conditions
sm.fire_trigger("jump");           // for Trigger conditions (auto-consumed each frame)
```

`TransitionCond::AnimationEnd` becomes true when a non-looping clip reaches its last frame.

### PhysicsWorld encapsulation

Internal rapier2d fields are `pub(crate)`. Do not access them directly from outside. Available accessors:

```
rigid_body() / rigid_body_mut()
get_collider() / get_collider_mut()
add_dynamic_circle() / add_dynamic_box() / add_static_box()
remove_body()
```

---

## Common task patterns

### Add a new component

1. Define the struct in `src/components.rs` or the relevant module file
2. Add a re-export in `src/lib.rs`

### Add a new system

1. Implement the `System` trait
2. Register with `app.add_system(Box::new(MySystem))` or in `Scene::on_enter`

### Add a new resource

1. Define the struct in `src/resources.rs`
2. Register with `app.world.insert_resource(MyResource { ... })`
3. Add a re-export in `src/lib.rs` if needed

### Add a new event

```rust
// 1. Define the type (needs Clone + 'static)
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
// SceneCmd::Push(Box::new(MyScene)) — push onto the stack
// SceneCmd::Pop                      — return to the previous scene
```
