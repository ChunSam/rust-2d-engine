# skeleton-engine

`skeleton-engine` is a lightweight Rust 2D game engine built on `wgpu`, a custom ECS, Rapier2D physics, input, UI, audio, particles, tilemaps, scripting, and WASM support.

The package name is `skeleton-engine`; the library crate name is intentionally `engine`, so examples use `use engine::*`.

## Why skeleton-engine

The name is the thesis: a **skeleton** — a clean, MIT-licensed 2D engine meant to be
forked and fleshed out. It favors being hackable and readable over being a sealed black
box, so you can take the source, modify engine code directly, and grow it into your own
engine.

Goals, in priority order:

1. **An open-source skeleton others can fork and extend.**
2. **A personal foundation for building 2D games.**
3. **A learning vehicle for how a 2D engine works from the ground up.**

Scope is **genre-agnostic 2D**: platformers, shooters, RPGs, puzzles, top-down action.
New features are validated through small, playable example games rather than in
isolation. See [`docs/VISION.md`](docs/VISION.md) for the full rationale.

## Requirements

- Rust 1.88 or newer
- Native Linux builds need common window/audio development packages such as `libasound2-dev`, `libudev-dev`, `libxkbcommon-dev`, Wayland/X11 headers, and `pkg-config`
- WASM builds use the `wasm32-unknown-unknown` target

## Install

```toml
[dependencies]
skeleton-engine = "1.0.0"
```

```rust
use engine::*;
```

## Quick Start

```rust
use engine::{
    App, Entity, InputState, KeyCode, ShouldQuit, Sprite, System, Transform, Vec2, WindowConfig,
    World,
};

#[derive(Clone)]
struct Player;

struct PlayerSystem;

impl System for PlayerSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        let mut direction = Vec2::ZERO;
        let mut should_quit = false;

        if let Some(input) = world.resource::<InputState>() {
            if input.is_pressed(KeyCode::KeyA) || input.is_pressed(KeyCode::ArrowLeft) {
                direction.x -= 1.0;
            }
            if input.is_pressed(KeyCode::KeyD) || input.is_pressed(KeyCode::ArrowRight) {
                direction.x += 1.0;
            }
            if input.is_pressed(KeyCode::KeyW) || input.is_pressed(KeyCode::ArrowUp) {
                direction.y += 1.0;
            }
            if input.is_pressed(KeyCode::KeyS) || input.is_pressed(KeyCode::ArrowDown) {
                direction.y -= 1.0;
            }
            should_quit = input.just_pressed(KeyCode::Escape);
        }

        if should_quit {
            if let Some(quit) = world.resource_mut::<ShouldQuit>() {
                quit.0 = true;
            }
        }

        let entities: Vec<Entity> = world.query::<Player>().map(|(entity, _)| entity).collect();
        let velocity = direction.normalize_or_zero() * 220.0 * dt;

        for entity in entities {
            if let Some(transform) = world.get_mut::<Transform>(entity) {
                transform.position += velocity;
                transform.rotation += dt;
            }
        }
    }
}

fn main() {
    let mut app = App::new();
    app.world.insert_resource(WindowConfig {
        title: "skeleton-engine basic".to_string(),
        width: 960,
        height: 540,
        clear_color: [0.04, 0.05, 0.08, 1.0],
    });

    let player = app.world.spawn();
    app.world.add_component(
        player,
        Transform {
            position: Vec2::new(480.0, 270.0),
            scale: Vec2::splat(64.0),
            ..Default::default()
        },
    );
    app.world.add_component(player, Sprite::colored(0.2, 0.8, 1.0));
    app.world.add_component(player, Player);

    app.add_system(PlayerSystem);
    app.run();
}
```

Run the included beginner example:

```sh
cargo run --example basic
```

Run the runtime policy configuration example:

```sh
cargo run --example runtime_policies
```

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release
cargo build --target wasm32-unknown-unknown
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo package --locked --list
cargo package --locked
cargo publish --dry-run --locked
```

## WASM

```sh
rustup target add wasm32-unknown-unknown
./scripts/build_wasm.sh
python3 -m http.server --directory dist 8080
```

## Documentation

- [`REFERENCE.html`](REFERENCE.html) in the repository root contains the public API reference and subsystem examples.
- [`ARCHITECTURE.html`](ARCHITECTURE.html) explains the maintainer-oriented engine structure and frame flow.
- Contributor handoff and agent notes live in the repository, outside the crates.io package.

## License

MIT. See `LICENSE`.
