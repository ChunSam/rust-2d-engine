# Engine UI interaction fixes after scene-flow review

**Date:** 2026-05-31
**Status:** IN PROGRESS
**Bead(s):** none
**Epic:** playable example validation
**Chain:** `scene-flow-ui-interaction` seq `2`
**Parent:** `HANDOFF_scene-flow-ui-interaction_2026-05-31.md`
**Prior chain:** `HANDOFF_scene-flow-ui-interaction_2026-05-31.md` > this

---

## Related Handoffs

- `HANDOFF_platformer-example-game_2026-05-30.md` — earlier playable-example candidate A workstream.
- `HANDOFF_maze-escape-example-game_2026-05-31.md` — playable-example candidate B workstream. It is related by project direction, but not the parent of this UI fix chain.

## Since Last Handoff

- The parent handoff stopped with scene-flow example behavior still under review and a suspected UI/mouse interaction problem.
- The scene-flow playable example itself was later committed and pushed as `74732ed feat: add scene flow playable example`.
- Maze-escape work happened after that and produced commits `dfe1946` and `77e778a`; current `HEAD` is `77e778a session: maze-escape playable example [maze-escape-example-game]`.
- The user then asked whether scene-flow required engine-side changes, requested a Korean code review of those engine changes, asked for a fix plan, reduced ambiguity, and finally supplied the concrete `Engine UI Interaction Fix Plan`.
- This handoff captures implementation of that plan: `ButtonClicked` release semantics, unified `DrawRect`/`DrawImage` z sorting, docs update, and verification.
- Runtime smoke improved enough to confirm rendering and keyboard flow, but automated macOS mouse click injection still did not transition the scene-flow window.
- Current work is not committed.

## Reference Documents

- `AGENTS.md` — repository instructions, module map, default workflow, docs language rule.
- `CLAUDE.md` — parallel quick reference and module map.
- `docs/VISION.md` — project direction: fork-friendly 2D skeleton engine with playable examples as validation.
- `docs/NEXT_WORK.md` — active next-work candidate list for playable examples.
- `docs/AGENT_WORKFLOW.md` — detailed implementation and verification expectations.
- `REFERENCE.html` — public API reference updated in this session for UI primitive ordering.

## The Goal

Fix the two engine issues discovered while dogfooding scene-flow: button click semantics and UI primitive draw ordering.
`ButtonClicked(Entity)` should preserve the documented meaning: press a button, release inside the same button, emit exactly once.
`DrawRect` and `DrawImage` should no longer be rendered as two separate layer passes that can invert expected z order; they should sort together in the same screen-space UI primitive layer.
The user explicitly required no public API removal or rename.
The result should make `scene_flow_game` a stronger regression example without adding new engine API surface unless needed.

## Where We Are

- Working directory: `/Users/jkl/Projects/skeleton-engine`.
- Current branch: `docs/english-conversion`.
- Current `HEAD`: `77e778a session: maze-escape playable example [maze-escape-example-game]`.
- Git status at handoff time:
  - `M REFERENCE.html`
  - `M examples/games/maze_escape/maze_escape.rs`
  - `M src/app.rs`
  - `M src/pathfinding.rs`
  - `M src/renderer/sprite.rs`
  - `M src/ui/system.rs`
- The intended changes for this task are:
  - `REFERENCE.html`
  - `src/app.rs`
  - `src/renderer/sprite.rs`
  - `src/ui/system.rs`
- The unrelated-looking current diffs are:
  - `examples/games/maze_escape/maze_escape.rs` — rustfmt-only wrapping.
  - `src/pathfinding.rs` — rustfmt-only wrapping.
- Do not revert the unrelated diffs blindly. They may be from earlier user/agent work or from repo-wide formatting during this session.
- `src/ui/system.rs` now emits `ButtonClicked` only on release in bounds:
  - condition: `just_released && in_rect && (prev == ButtonState::Pressed || just_pressed)`.
  - press-only frame emits no click.
  - normal press then release emits one click on release.
  - same-frame press/release emits one click.
  - release outside bounds emits no click.
- `src/ui/system.rs` tests were replaced/expanded:
  - `button_click_emits_once_on_release_in_bounds`
  - `button_click_handles_press_and_release_in_same_frame_once`
  - `button_click_does_not_emit_when_released_outside`
- `src/renderer/sprite.rs` now has private UI primitive helpers:
  - `UiPrimitiveKind`
  - `UiPrimitive`
  - `ui_quad_instance`
  - `sorted_ui_primitives`
- `UiPrimitiveKind::sort_rank()` fixes same-z type order:
  - Image rank `0`
  - Rect rank `1`
- `sorted_ui_primitives(rects, images)` creates one list and sorts by:
  - `z` ascending
  - image before rect for equal `z`
  - insertion order inside the same primitive type
- `SpriteRenderer::render_ui_primitives_from_slices(...)` is the new unified render path.
- `SpriteRenderer::render_ui_rects_from_slice(...)` remains and delegates to the unified path with empty images.
- `SpriteRenderer::render_ui_images_from_slice(...)` remains and delegates to the unified path with empty rects.
- `App::render()` now drains `UiImageQueue` and `UiQueue`, then calls `render_ui_primitives_from_slices` once if either list is non-empty.
- Text still renders after the UI primitive pass.
- `REFERENCE.html` now says `DrawRect` and `DrawImage` draw in a screen-space UI primitive layer above sprites and below text.
- `REFERENCE.html` now documents shared z sorting and equal-z image-before-rect behavior.
- `src/app.rs` currently also removes `scale_factor` division from cursor and touch event coordinates.
- That cursor/touch scale change was part of the user's supplied plan, but runtime smoke with synthetic mouse events did not confirm mouse transitions.
- `cargo run --example scene_flow_game` compiled and launched the example during smoke testing.
- A screen capture confirmed the scene-flow window rendered with dark full-window background and visible Start/Quit buttons.
- Keyboard `Enter` was injected and confirmed `Menu -> Play` transition.
- Several macOS synthetic mouse click attempts did not trigger Start/Pause transitions on screen.
- It is unknown whether the failed mouse smoke is due to OS synthetic event delivery, winit event behavior, coordinate conversion, or remaining engine input issue.
- Static verification is clean.

## What We Tried (Chronological)

1. Read the user-supplied `Engine UI Interaction Fix Plan`.
   - Scope was explicit: fix `ButtonClicked` semantics and unify `DrawRect`/`DrawImage` z sorting.
   - Public API removal or rename was out of scope.
   - Docs update was required.

2. Inspected `src/ui/system.rs`, `src/renderer/sprite.rs`, `src/app.rs`, and `REFERENCE.html`.
   - `UiSystem` had click emission coupled to `started_in_rect` and release-after-pressed behavior from the previous scene-flow fix.
   - `SpriteRenderer` had separate rect and image UI rendering methods.
   - `App::render()` drained and rendered UI images separately from UI rects.

3. Reworked `ButtonClicked` in `UiSystem`.
   - Old behavior emitted on press in bounds in the post-scene-flow state.
   - New behavior computes `clicked` before state mutation:
     - `let clicked = just_released && in_rect && (prev == ButtonState::Pressed || just_pressed);`
   - The button state still becomes `Pressed` on press while held.
   - The click event is pushed only if `clicked` is true.

4. Replaced the prior same-frame click test with targeted release-semantics tests.
   - Added `setup_button_world(cursor)` helper.
   - Added `click_count(world, entity)` helper.
   - Verified press-only frame produces zero events and `ButtonState::Pressed`.
   - Verified release in bounds produces one event.
   - Verified same-frame press/release produces one event.
   - Verified release outside bounds produces zero events.

5. Added unified UI primitive ordering in `src/renderer/sprite.rs`.
   - Added private `UiPrimitiveKind` enum.
   - Added private `UiPrimitive` struct.
   - Added `ui_quad_instance(...)` so rects and images share quad conversion logic.
   - Added `sorted_ui_primitives(rects, images)`.

6. Implemented deterministic sorting.
   - Primary sort: `z.partial_cmp(...).unwrap_or(Ordering::Equal)`.
   - Secondary sort: `kind.sort_rank()`, so images draw before rects at the same z.
   - Tertiary sort: `order`, preserving insertion order inside each primitive type.
   - Note: order is per-type insertion order, matching the plan's “same type” wording.

7. Added renderer ordering test.
   - Test name: `renderer::sprite::tests::ui_primitives_sort_by_z_type_then_queue_order`.
   - Rect z values: `1.0`, `0.5`, `1.0`.
   - Image z values: `1.0`, `0.25`, `1.0`.
   - Expected order:
     - `image-b.png`
     - `rect-1`
     - `image-a.png`
     - `image-c.png`
     - `rect-0`
     - `rect-2`

8. Added `SpriteRenderer::render_ui_primitives_from_slices`.
   - It writes the UI camera uniform once.
   - It builds sorted primitive entries.
   - It writes one UI instance buffer.
   - It begins one render pass labelled `"ui primitive pass"`.
   - It batches adjacent entries by texture key using existing `bind_group_for_texture_key`.
   - Rects use `None` texture key, which maps to the white texture.
   - Images keep their existing texture key / handle / uv / color behavior.

9. Kept compatibility wrappers.
   - `render_ui_rects_from_slice` now calls the unified method with `images = &[]`.
   - `render_ui_images_from_slice` now calls the unified method with `rects = &[]`.
   - No public method was removed.

10. Updated `App::render()`.
   - Drains `UiImageQueue` first into `ui_images`.
   - Drains `UiQueue` next into `ui_rects`.
   - Calls `sr.render_ui_primitives_from_slices(..., &ui_rects, &ui_images, logical_w, logical_h)`.
   - Keeps text rendering after this block.

11. Updated `REFERENCE.html`.
   - UI rect section now describes the screen-space UI primitive layer.
   - UI image section now says images use the same layer.
   - Shared z sorting and equal-z image-before-rect are documented.

12. Ran targeted tests.
   - `cargo test ui::system::tests::button_click`
   - `cargo test renderer::sprite::tests::ui_primitives_sort_by_z_type_then_queue_order`
   - Both passed.

13. Ran broad static verification.
   - `cargo fmt --check` passed.
   - `cargo check --example scene_flow_game` passed.
   - `cargo test` passed.
   - `cargo clippy --all-targets -- -D warnings` passed.

14. Ran `scene_flow_game`.
   - Used `cargo run --example scene_flow_game`.
   - The binary launched successfully.
   - Screen capture showed the game window and scene-flow menu.
   - Background filled the visible game window.
   - Buttons rendered above the background/tint.

15. Tried Computer Use direct targeting.
   - `mcp__computer_use.get_app_state({"app":"scene_flow_game"})` failed with `Invalid app: scene_flow_game`.
   - This matches the parent handoff: native example binary is not a `.app` bundle target.

16. Used macOS screen capture and OS-level input as fallback.
   - `screencapture -x /private/tmp/scene_flow_smoke.png`.
   - `osascript` was used to focus/click the `scene_flow_game` process.
   - Swift `CGEvent` was used for mouse move/down/up and keyboard events.

17. Confirmed keyboard flow.
   - Injected `Enter`.
   - Screen capture after Enter showed `Play Scene`.
   - This confirmed the window was active enough for keyboard input and the example scene transition logic worked.

18. Could not confirm mouse runtime smoke by automation.
   - Start button clicks at apparent screen-pixel coordinates did not transition.
   - Start button clicks at apparent logical coordinates did not transition.
   - Mouse move + click did not transition.
   - `System Events click at` returned the window object but did not cause button transition.
   - The root cause remains unresolved.

19. Revisited cursor/touch scale factor.
   - `rg` showed current code still had cursor/touch scale division before the final plan implementation.
   - The user plan specifically requested reverting the recent scale-factor division.
   - `src/app.rs` was patched so `CursorMoved` stores raw `position.x/y`, and touch stores raw `location.x/y`.
   - After that change, static checks remained green.
   - Automated mouse smoke still failed to transition.

20. Stopped the running example.
   - `pkill -x scene_flow_game` was used after smoke attempts.
   - A previous attempt to send Ctrl-C to the `cargo run` session failed because stdin was closed.

## Key Decisions

- Preserve release-in-bounds semantics for `ButtonClicked`.
  - The supplied plan and `REFERENCE.html` meaning both require click on release, not press.
  - The previous parent-session press-based workaround was replaced.

- Support same-frame press/release without changing the public event.
  - Same-frame click is handled by allowing `just_pressed` in the release condition.
  - This avoids missing fast clicks while preserving release semantics.

- Keep public rendering methods.
  - Removing `render_ui_rects_from_slice` or `render_ui_images_from_slice` would be unnecessary API churn.
  - Compatibility wrappers are lower risk.

- Use one unified UI primitive pass.
  - Separate passes made image-vs-rect layering depend on pass order rather than the documented `z`.
  - A unified sorted list makes behavior deterministic.

- Use image-before-rect at equal z.
  - This exactly matches the user's plan.
  - It lets a rect tint or overlay cover an image at the same z.

- Keep text after UI primitives.
  - The plan explicitly says UI rect/image primitives are above sprites and below text.
  - `App::render()` preserves that order.

- Remove cursor/touch scale-factor division for now.
  - The user-supplied final plan explicitly required this.
  - Runtime smoke did not prove this fixed mouse interaction, so this remains a risk to revisit.

- Do not touch the maze/pathfinding diffs as part of this task.
  - They are unrelated formatting-only diffs in current status.
  - They should be reviewed separately before committing.

- Do not commit automatically.
  - User requested handoff, not commit.

## Evidence & Data

### Git State

| Item | Value |
|---|---|
| Working directory | `/Users/jkl/Projects/skeleton-engine` |
| Branch | `docs/english-conversion` |
| HEAD | `77e778a session: maze-escape playable example [maze-escape-example-game]` |
| Parent chain file | `plans/handoffs/HANDOFF_scene-flow-ui-interaction_2026-05-31.md` |
| New handoff status | uncommitted |

### Current Dirty Files

| File | Purpose / status |
|---|---|
| `REFERENCE.html` | Intended docs update for UI primitive ordering |
| `src/app.rs` | Intended render call update and cursor/touch coordinate change |
| `src/renderer/sprite.rs` | Intended unified UI primitive sorting/rendering |
| `src/ui/system.rs` | Intended `ButtonClicked` semantics and tests |
| `examples/games/maze_escape/maze_escape.rs` | Unrelated rustfmt wrapping diff |
| `src/pathfinding.rs` | Unrelated rustfmt wrapping diff |

### Diff Stat

```text
REFERENCE.html                            |   4 +-
examples/games/maze_escape/maze_escape.rs |  24 ++-
src/app.rs                                |  39 +----
src/pathfinding.rs                        |   7 +-
src/renderer/sprite.rs                    | 240 ++++++++++++++++++------------
src/ui/system.rs                          |  85 +++++++++--
6 files changed, 239 insertions(+), 160 deletions(-)
```

### Recent Commits

| Hash | Summary |
|---|---|
| `77e778a` | `session: maze-escape playable example [maze-escape-example-game]` |
| `dfe1946` | `feat: add maze-escape playable example + BT/SpatialGrid/PathGrid API gaps` |
| `74732ed` | `feat: add scene flow playable example` |
| `455f9d4` | `feat: add platformer playable example` |
| `05b2915` | `Add audio channel playback state` |

### Verification Commands

| Command | Result |
|---|---|
| `cargo fmt --check` | passed |
| `cargo check --example scene_flow_game` | passed |
| `cargo test ui::system::tests::button_click` | passed targeted UI click tests |
| `cargo test renderer::sprite::tests::ui_primitives_sort_by_z_type_then_queue_order` | passed targeted renderer ordering test |
| `cargo test` | passed, 241 unit tests |
| doctests via `cargo test` | passed, 33 passed, 19 ignored |
| `cargo clippy --all-targets -- -D warnings` | passed |
| `cargo run --example scene_flow_game` | launched window for runtime smoke |

### Runtime Smoke Evidence

| Check | Result |
|---|---|
| Window visible | yes, screen capture showed `skeleton-engine scene flow game` |
| Background fills viewport | visually yes in captures |
| Buttons above background/tint | visually yes in captures |
| Keyboard Enter Menu -> Play | confirmed by capture showing `Play Scene` |
| Computer Use direct app target | failed: `Invalid app: scene_flow_game` |
| Synthetic mouse Start click | not confirmed; no transition observed |
| Synthetic mouse Pause click | not confirmed; no transition observed |

### Important Raw Snippets

`UiSystem` click condition now:

```rust
let clicked =
    just_released && in_rect && (prev == ButtonState::Pressed || just_pressed);
```

Renderer ordering rule now:

```rust
primitives.sort_by(|a, b| {
    a.z.partial_cmp(&b.z)
        .unwrap_or(Ordering::Equal)
        .then_with(|| a.kind.sort_rank().cmp(&b.kind.sort_rank()))
        .then_with(|| a.order.cmp(&b.order))
});
```

`App::render()` now calls the combined primitive path:

```rust
sr.render_ui_primitives_from_slices(
    &gpu.device,
    &gpu.queue,
    render_view,
    &mut enc,
    &ui_rects,
    &ui_images,
    logical_w,
    logical_h,
);
```

## Code Analysis

- `UiSystem::run` reads cursor and mouse button edges from `InputState` once at the top of the system.
- `ButtonState` is updated before event emission, but `prev` is captured first, so release-after-pressed can still be detected.
- Same-frame press/release only works because `InputState` can have both `mouse_just_pressed` and `mouse_just_released` true before `flush()`.
- `ButtonClicked` events are stored in `Events<UiEvent>` and counted by tests via `read()`.
- `sorted_ui_primitives` is private, which keeps the public renderer surface stable.
- `DrawRect` maps to `texture_key: None`, so it uses the renderer's white texture bind group.
- `DrawImage` keeps `image.texture_key()` and therefore existing path/handle aliasing behavior.
- Unified rendering still batches only adjacent equal texture keys after sorting; this is correct because z order takes precedence over batching.
- `render_ui_primitives_from_slices` has `#[allow(clippy::too_many_arguments)]`, consistent with the existing renderer API style.
- The current cursor/touch raw-coordinate storage in `src/app.rs` is the least-settled part of this change and should be manually rechecked on real mouse input.

## Files Changed

### Source code

- `src/ui/system.rs` — changed `ButtonClicked` to release-in-bounds semantics; added focused tests.
- `src/renderer/sprite.rs` — added private UI primitive abstraction, deterministic mixed rect/image sorting, unified render path, and compatibility wrappers.
- `src/app.rs` — drains UI image and rect queues together and renders them through one primitive pass; cursor/touch scale-factor division removed per plan.

### Tests

- `src/ui/system.rs` — button click tests for press-only, release in bounds, same-frame press/release, and release outside.
- `src/renderer/sprite.rs` — mixed primitive ordering test.

### Docs

- `REFERENCE.html` — documents the shared UI primitive layer and `DrawRect`/`DrawImage` ordering rules.

### Unrelated dirty files to treat carefully

- `examples/games/maze_escape/maze_escape.rs` — formatting-only diff seen in current worktree.
- `src/pathfinding.rs` — formatting-only diff seen in current worktree.

## User Feedback & Preferences

- User wanted documented next work checked before implementing examples.
- User distinguishes feature examples from playable game examples; games belong under `examples/games/...`.
- User explicitly placed scene-flow game at `examples/games/scene_flow/scene_flow.rs`.
- User repeatedly asks for ambiguity checks before plans and wants ambiguity under 10%.
- User prefers direct Korean summaries and practical next steps.
- User expects implementation after approving a plan, not more discussion.
- User asked to run the game and visually/interaction-test it.
- User reported concrete QA issues: background too small, font too thin, white background too bright, `M` key issue, mouse click not working.
- User asked to use Computer Use for testing, but this native binary cannot be directly targeted by Computer Use.
- User asked whether scene-flow required engine changes, then requested review of engine-side modifications.
- User asked for a fix plan, ambiguity review, and then supplied the final implementation plan.
- User's final plan prioritized release-in-bounds `ButtonClicked` semantics over the earlier press-based workaround.
- User requested this handoff; do not commit or archive unless explicitly asked next.

## Where We're Going

1. Review the current diff before commit, with special attention to `src/app.rs` cursor/touch coordinate handling.
2. Decide whether the unrelated maze/pathfinding formatting diffs should be included, reverted, or separated. Do not assume.
3. Manually test real mouse input in `scene_flow_game` if possible, not only synthetic OS events.
4. If real mouse clicks still fail, inspect `winit` cursor coordinate units vs `ViewportSize`/logical render size and add a focused integration or unit test around coordinate conversion if feasible.
5. If real mouse clicks pass, commit the four intended engine/doc files separately from unrelated formatting diffs.
6. Consider adding a short docs/NEXT_WORK or handoff note only if the coordinate friction remains unresolved.

## Risks & Blockers

- Synthetic macOS mouse events did not trigger scene transitions, so runtime mouse behavior is not fully verified.
- Cursor/touch scale-factor handling is still the highest-risk part of the implementation.
- The worktree contains unrelated maze/pathfinding formatting diffs that could pollute a commit.
- `Computer Use` cannot directly target `scene_flow_game`, limiting GUI automation.
- `REFERENCE.html` is hand-edited generated/static documentation; ensure this is the repo's expected doc update path before release.

## Open Questions

- Does a physical mouse click on the current build trigger Start/Pause/Resume/Complete/Retry/Menu transitions?
- Should cursor/touch coordinates be raw winit positions, scale-factor divided positions, or converted against `ViewportSize` in a single helper?
- Should the unrelated maze/pathfinding rustfmt diffs be kept with this work, committed separately, or reverted with user confirmation?
- Should a regression test be added for app-level cursor coordinate conversion, or is `UiSystem` unit coverage enough?

## Quick Start for Next Session

```bash
# Restore context
sed -n '1,260p' plans/handoffs/HANDOFF_scene-flow-ui-interaction_2026-05-31.md
sed -n '1,360p' plans/handoffs/HANDOFF_scene-flow-ui-interaction_engine-ui-fixes_2026-05-31.md

# Key files to read first
sed -n '60,130p' src/ui/system.rs
sed -n '430,520p' src/renderer/sprite.rs
sed -n '1340,1445p' src/renderer/sprite.rs
sed -n '2375,2425p' src/app.rs
sed -n '2728,2795p' src/app.rs

# Verify current state
git status -s
cargo fmt --check
cargo check --example scene_flow_game
cargo test
cargo clippy --all-targets -- -D warnings

# Runtime check
cargo run --example scene_flow_game

# Next action
# Manually click scene_flow_game buttons with a real mouse/trackpad and decide whether src/app.rs cursor/touch coordinate handling is correct.
```
