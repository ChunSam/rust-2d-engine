# Engine Follow-ups Implementation

This document summarizes the engine-side work completed from the rust-survivors follow-up requests.

## Source Request

The original requests came from:

- `/Users/jkl/Projects/rust-survivors/docs/ENGINE_FOLLOWUPS.md`

The review split the requests into:

- General engine features that belong in `skeleton-engine`.
- rust-survivors-specific migration/data/layout work that should remain in the game repo.

Game-side follow-up notes were documented separately in:

- `/Users/jkl/Projects/rust-survivors/docs/ENGINE_MIGRATION_NOTES.md`

## Implemented In skeleton-engine

### UV Helpers

File:

- `src/animation/player.rs`

Added:

- `UvRect::new(u_offset, v_offset, u_size, v_size)`
- `UvRect::from_pixels(x, y, width, height, texture_width, texture_height)`
- `UvRect::flipped_x()`
- `UvRect::flipped_y()`

Purpose:

- Support vertically flipped sheets without ad hoc negative `v_size` math in game code.
- Support hand-tuned pixel crop rectangles for packed or non-uniform atlases.
- Keep UV orientation and crop handling as reusable engine primitives.

### AtlasSprite Custom UV Override

Files:

- `src/atlas.rs`
- `src/renderer/sprite.rs`

Changed:

- `TextureAtlas::uv_rect(...)` now safely returns `UvRect::FULL` when `cols` or `rows` is zero.
- `AtlasSprite` rendering now checks whether the same entity has a `UvRect` component.
- If `UvRect` exists, the renderer uses it instead of the grid-derived atlas UV.
- If `UvRect` does not exist, existing `atlas.uv_rect(index)` behavior is unchanged.

Purpose:

- Preserve the existing `AtlasSprite` handle path and batching behavior.
- Allow custom crops and flipped UVs without adding new fields to the public `AtlasSprite` struct.
- Avoid breaking external code that may initialize `AtlasSprite` with struct literals.

### Sprite Handle + Path Fallback Helper

File:

- `src/components.rs`

Added:

- `Sprite::textured_with_handle(path, Option<Handle<ImageAsset>>)`

Purpose:

- Let runtime code prefer `Handle<ImageAsset>` when available.
- Keep a string path fallback for tests and small isolated worlds.
- Centralize a pattern that rust-survivors previously kept in its own helper.

### Screen-Space UI Images

Files:

- `src/renderer/ui.rs`
- `src/renderer/mod.rs`
- `src/renderer/sprite.rs`
- `src/app.rs`
- `src/lib.rs`

Added:

- `DrawImage`
- `UiImageQueue`
- Public re-exports for both types.
- Default `UiImageQueue` resource insertion in `App::new()` and `App::reload_scene()`.
- Screen-space image rendering through `SpriteRenderer::render_ui_images_from_slice(...)`.

Behavior:

- `DrawImage` uses logical viewport coordinates, like `DrawRect`.
- Images are camera-independent.
- Rendering occurs after UI rectangles and before GPU particles/text in the current render flow.
- `DrawImage::textured_with_handle(...)` mirrors the sprite handle/path fallback pattern.
- `DrawImage` supports `with_uv(...)`, `with_z(...)`, and `with_color(...)`.

Purpose:

- Replace game-side HUD/icon workarounds that spawn world entities, convert screen coordinates through the camera, and despawn/recreate icon sprites every frame.
- Provide a reusable primitive for HUD icons, card icons, inventory images, shop icons, and similar 2D UI textures.

## Documentation Updated

Files:

- `REFERENCE.html`
- `docs/HANDOFF.md`

Added coverage for:

- `Sprite::textured_with_handle(...)`
- `DrawImage` / `UiImageQueue`
- top-left-origin `UvRect::from_pixels(...)` crops and explicit `.flipped_y()` mirroring
- `AtlasSprite + UvRect` custom UV override behavior

## Tests Added

Files:

- `src/animation/player.rs`
- `src/atlas.rs`
- `src/components.rs`
- `src/renderer/ui.rs`

Covered:

- Pixel crop normalization.
- Horizontal and vertical UV flipping.
- Atlas grid index wrapping.
- Empty atlas grid fallback.
- Sprite handle/path fallback constructor behavior.
- `DrawImage` path fallback behavior.

## Verification

Commands run:

```text
cargo fmt
cargo test
```

Result:

- Historical implementation run: unit tests `203 passed`. Current release verification is tracked in `docs/PROJECT_SCAN_REPORT.md`.
- Doc tests: `31 passed`, `19 ignored`

## Game-Side Work Left For rust-survivors

The following should stay in the game repo:

- `SurvivorSprite` enum and frame/crop tables.
- `SurvivorTextureHandles` path-to-handle lookup policy.
- Weapon/passive/power-up icon indexes.
- HUD, level-up, and shop layout constants.
- Generated survivor atlas metadata that names game concepts directly.

Suggested migration order is documented in:

- `/Users/jkl/Projects/rust-survivors/docs/ENGINE_MIGRATION_NOTES.md`

## Notes

- The engine changes were kept backward compatible where practical.
- `AtlasSprite` was not given new public fields; the custom UV path uses the existing `UvRect` component.
- Existing `DrawRect` and `UiQueue` APIs were left unchanged.
