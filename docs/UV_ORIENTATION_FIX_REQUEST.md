# UV Orientation Fix Request

> Status: completed in `skeleton-engine`; this file is retained as historical request context. See `docs/CHANGELOG.md` and `docs/RUST_SURVIVORS_UV_MIGRATION_PROMPT.md` for the current migration note.

## Summary

Before the fix, `skeleton-engine` rendered normal PNG textures upside down unless callers applied
`UvRect::flipped_y()`. This is visible in `rust-survivors` whenever a new image is
added through `Sprite`, `DrawImage`, or atlas/grid UV paths without a manual vertical
flip.

The likely root cause is the engine's default sprite quad UV orientation. The engine
camera and UI coordinate system are top-left anchored with Y increasing downward, but
the quad vertex UVs assign `v = 1.0` to the top edge and `v = 0.0` to the bottom edge.

## Target

- Repository: `skeleton-engine`
- Consumer project: `/Users/jkl/Projects/rust-survivors`
- `rust-survivors` currently uses engine commit:
  `0e01b0f383202acea4d2ff606be4c62f73371471`

## Evidence

`src/camera.rs` documents the engine coordinate convention:

- Camera is top-left anchored.
- Y increases downward.

`src/renderer/sprite.rs` defines the shared quad vertices with inverted V coordinates:

```rust
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5],
        uv: [0.0, 0.0],
    },
];
```

`src/renderer/shaders/sprite.wgsl` passes that orientation through directly:

```wgsl
out.uv = inst.uv_offset + v.uv * inst.uv_size;
```

Because of this, `UvRect::FULL`, `UvRect::from_pixels(...)`, and
`UvRect::from_grid(...)` sample the correct rectangle but display it vertically
mirrored.

## Expected Behavior

- `Sprite::textured(...)` and `Sprite::textured_with_handle(...)` should render a
  normal PNG in the same orientation as the source file without requiring
  `UvRect::flipped_y()`.
- `DrawImage::textured(...)`, `DrawImage::with_handle(...)`, and
  `DrawImage::textured_with_handle(...)` should behave the same way.
- `AtlasSprite` and `UvRect::from_grid(col, row, cols, rows)` should treat `row = 0`
  as the top row of the image.
- `UvRect::from_pixels(x, y, w, h, texture_w, texture_h)` should treat `x, y` as a
  top-left-origin pixel crop.
- `UvRect::flipped_y()` should remain an explicit vertical mirror operation, not a
  required correction for normal sprite rendering.

## Suggested Engine Change

Update the default quad UVs in `src/renderer/sprite.rs` so the top edge uses
`v = 0.0` and the bottom edge uses `v = 1.0`:

```rust
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5],
        uv: [0.0, 1.0],
    },
];
```

This makes the default quad match the engine's top-left/Y-down rendering convention
and the image crate's decoded row order.

## Compatibility Risk

This is a behavior change for existing games that already compensate with
`UvRect::flipped_y()`. Those games will show textures double-flipped after the engine
fix until they remove the workaround.

Please treat this as a breaking rendering behavior fix, or provide a migration note
or compatibility flag.

## rust-survivors Current Workarounds

`rust-survivors` currently applies vertical flip corrections in several places:

- Atlas/crop sprites: `UvRect::from_pixels(...).flipped_y()`
- Icon grids: `UvRect::from_grid(...).flipped_y()`
- Full-image world/UI textures: `UvRect::FULL.flipped_y()`

After the engine fix lands, these workaround flips should be removed from the game
code.

## Requested Tests

Add or update engine tests that lock down the intended orientation:

- Default full-texture `Sprite` renders top-left image pixels on the top-left of the
  quad.
- `DrawImage` full-texture rendering uses the same orientation as `Sprite`.
- `AtlasSprite` / `UvRect::from_grid(...)` treats row `0` as the top row.
- `UvRect::from_pixels(...)` uses top-left-origin pixel coordinates.
- `UvRect::flipped_y()` still mirrors the selected region vertically.

If full GPU screenshot tests are not practical, add unit tests around the shared quad
vertex UV data and UV helper semantics so the default orientation cannot regress.

## rust-survivors Migration Steps After Engine Update

1. Update the `skeleton-engine` dependency commit in `rust-survivors`.
2. Remove the game-side `.flipped_y()` workarounds listed above.
3. Run:

```bash
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
```

4. Perform visual QA for:

- Title backdrop and title menu image buttons
- HUD slot frames and modal panels
- Level-up card and shop row frames
- Weapon/passive/powerup icons
- Actor animation frames
- Combat effect sprites
