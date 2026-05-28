# Changelog

All notable changes to `skeleton-engine` are documented here.

The package follows semantic versioning beginning with 1.0.0.

## Unreleased

### Changed

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
