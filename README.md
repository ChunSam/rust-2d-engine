# skeleton-engine

`skeleton-engine` is a lightweight Rust 2D game engine built on `wgpu`, a custom ECS, Rapier2D physics, input, UI, audio, particles, tilemaps, scripting, and WASM support.

The package name is `skeleton-engine`; the library crate name is intentionally `engine`, so examples use `use engine::*`.

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
