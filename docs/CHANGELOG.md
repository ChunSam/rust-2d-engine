# Changelog

All notable changes to `skeleton-engine` are documented here.

The package follows semantic versioning beginning with 1.0.0.

## Unreleased

### Added

- Re-exported `AssetId`, `SaveKey`, `save_with_key`, and `load_with_key` from the crate root so public examples match the stable API surface.
- Added `ScheduleErrorPolicy` and `SystemPanicPolicy` so apps can opt into stricter schedule-cycle and system-panic behavior while keeping the existing fallback defaults.
- Added `examples/runtime_policies.rs` to show strict runtime policy configuration without opening a long-running window.
- Added `World::mark_changed<T>()` and `World::get_mut_tracked<T>()` for explicit ECS change tracking after direct component mutation.
- Added `docs/ENTITY_GENERATION_V2_PLAN.md` to lock the v2 design for generation-checked entity handles.

### Changed

- Aligned save encryption and async asset examples in the public reference with the current source.
- Native `AssetServer` cache keys now canonicalize existing file paths, reducing duplicate handles and hot-reload misses caused by mixed relative/absolute paths. Missing paths and WASM URLs keep their existing string behavior.
- `PhysicsSystem` now documents the physics-unit to pixel-unit boundary and defensively clamps invalid `pixels_per_unit` values in release builds while asserting in debug builds.
- Clarified that Rhai scripting is intended for trusted local game code, not hostile sandboxing, and documented the limits of temporary script spawn IDs.
- **Breaking rendering behavior fix:** fixed the default sprite quad UV orientation so `Sprite`, `DrawImage`, `AtlasSprite`,
  `UvRect::FULL`, `UvRect::from_grid(...)`, and `UvRect::from_pixels(...)` render
  normal top-left-origin PNGs upright without requiring `UvRect::flipped_y()`.
  Existing game-side `.flipped_y()` orientation workarounds should be removed after
  updating the engine.

## [1.0.0] - 2026-05-27

### Added

- Stable `skeleton-engine` package metadata with library crate name `engine`.
- Rust 1.88 minimum supported Rust version declaration.
- README, MIT license, changelog, and beginner `examples/basic.rs`.
- CI gates for formatting, clippy, full native tests, release build, WASM build, rustdoc warnings, `cargo package`, and `cargo publish --dry-run`.

### Changed

- Documented release package hygiene with an explicit crates.io include list.
- Updated public documentation examples for current `OffscreenCamera`, `Sprite`, `TouchState`, and `glam::Vec2` usage.
