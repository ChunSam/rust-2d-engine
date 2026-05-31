# Scene-flow playable example and UI interaction fixes

**Date:** 2026-05-31
**Status:** IN PROGRESS
**Bead(s):** none
**Epic:** playable example validation
**Chain:** `scene-flow-ui-interaction` seq `1`
**Parent:** none — first in chain
**Prior chain:** none — first in chain

---

## Related Handoffs

- `plans/handoffs/HANDOFF_platformer-example-game_2026-05-30.md` — previous playable-example workstream; it completed the platformer example and pointed `docs/NEXT_WORK.md` toward scene flow next.

## Reference Documents

- `AGENTS.md` — repository agent instructions, module map, and workflow.
- `CLAUDE.md` — parallel quick reference for project conventions.
- `docs/VISION.md` — current product direction: fork-friendly 2D skeleton engine, breadth-first, playable examples as completion criteria.
- `docs/NEXT_WORK.md` — active next-work candidate list; now marks platformer and scene-flow examples done.
- `docs/AGENT_WORKFLOW.md` — detailed agent operating rules.

## The Goal

The user asked to implement and then fix a new `scene_flow_game` playable example.
The example exists to validate real scene-stack behavior, not to build deep gameplay:
`SceneCmd::Replace`, `SceneCmd::Push`, `SceneCmd::Pop`, UI buttons, keyboard shortcuts,
`GameState`, and scene-owned cleanup all need to be visible in one small game.
The user specifically reported visual and interaction problems after the first implementation:
background did not fill the viewport, font contrast/readability was weak, `M` behavior was unclear,
and mouse clicks did not activate buttons.

## Where We Are

- Working directory: `/Users/jkl/Projects/skeleton-engine`.
- Current branch at handoff time: `docs/english-conversion`.
- Latest committed HEAD at handoff time: `455f9d4 feat: add platformer playable example`.
- There are uncommitted changes in this workstream.
- `Cargo.toml` now includes a new example target:
  - `name = "scene_flow_game"`
  - `path = "examples/games/scene_flow/scene_flow.rs"`
- New example source file:
  - `examples/games/scene_flow/scene_flow.rs`
  - line count at handoff time: 677 lines.
- New generated assets:
  - `examples/games/scene_flow/assets/flow_bg.png`
  - `examples/games/scene_flow/assets/flow_badge.png`
- `docs/NEXT_WORK.md` now marks candidate `E`, Scene-flow game, as done.
- `docs/NEXT_WORK.md` now recommends `B / C / D` next to widen genre coverage.
- `src/app.rs` has two relevant engine changes:
  - UI render order changed so `UiImageQueue` is rendered before `UiQueue`.
  - cursor and touch coordinates are divided by `window.scale_factor()` before storing in `InputState`.
- `src/ui/system.rs` has one UI behavior change:
  - `UiEvent::ButtonClicked` is emitted when a button receives `just_pressed` inside its bounds, while retaining the old release-after-pressed path.
- `src/ui/system.rs` has a new unit test:
  - `ui::system::tests::button_click_handles_press_and_release_in_same_frame`.
- `scene_flow.rs` uses `SceneFlowStats` with `Arc<Mutex<StatsData>>`.
- `SceneFlowStats` is intentionally carried through scene constructors so stats survive `Replace`, because `reload_scene()` resets the `World`.
- `scene_flow.rs` defines four scenes:
  - `MenuScene`
  - `PlayScene`
  - `PauseScene`
  - `ResultScene`
- Base scenes (`MenuScene`, `PlayScene`) register:
  - `BackdropSystem`
  - `UiSystem`
  - scene-specific control system.
- Overlay scenes (`PauseScene`, `ResultScene`) do not register a new `UiSystem`.
- Overlay scenes hide existing UI on enter with `hide_existing_ui()`.
- Overlay scenes restore hidden UI on pop/exit with `restore_ui()`.
- Each scene tracks its spawned entities in a `Vec<Entity>` and calls `despawn_all()` in `on_exit()`.
- `PlaySystem` ignores play controls when `GameState != GameState::Playing`.
- `PauseScene` sets `GameState::Paused`.
- `ResultScene` sets `GameState::GameOver`.
- `PauseScene::on_exit()` resets `GameState` to `Playing`; this is fine for `Pop`, but note it also runs during `Replace(MenuScene)` before the new scene resets the state.
- `BackdropSystem` draws `flow_bg.png` with `DrawImage::textured(0, 0, w, h, BG_PATH)`.
- `BackdropSystem` draws `flow_badge.png` at `28,28,112,112`.
- `BackdropSystem` adds a dark full-screen tint rect with `[0.0, 0.025, 0.035, 0.72]`.
- `configure_window()` sets `WindowConfig` title to `skeleton-engine scene flow game`.
- `configure_window()` sets the clear color to dark `[0.0, 0.018, 0.026, 1.0]`.
- `BG_PATH` and `BADGE_PATH` are absolute at compile time using `env!("CARGO_MANIFEST_DIR")`.
- This absolute asset path is important because directly opening the binary changes the working directory.
- Runtime issue still open:
  - `cargo run --example scene_flow_game` starts a process but did not show a window during the last user-facing run.
  - Building then opening the binary directly did show a window.
- Direct run workaround:
  - `cargo build --example scene_flow_game`
  - `open /Users/jkl/Projects/skeleton-engine/target/debug/examples/scene_flow_game`
- At the end of the session, a direct-open window was confirmed:
  - process owner: `scene_flow_game`
  - title: `skeleton-engine scene flow game`
  - bounds from CGWindow list: `Height = 572`, `Width = 960`, `X = 237`, `Y = 116`.
- `Computer Use` cannot target `scene_flow_game` directly because it is a native example binary, not a `.app` bundle with a bundle identifier.

## What We Tried (Chronological)

1. Checked the documented next-work plan.
   - Found `docs/NEXT_WORK.md`.
   - It listed Scene-flow as the recommended next playable example after platformer.
   - User asked for ambiguity checks before implementation.

2. Planned the scene-flow example.
   - User supplied final implementation plan.
   - Scope: `examples/games/scene_flow/scene_flow.rs`, no public API changes, new PNG assets, docs update.
   - Required flow: `Menu -> Play -> Pause overlay -> Resume -> Result overlay -> Retry/Menu`.

3. Added the new Cargo example target.
   - Edited `Cargo.toml`.
   - Added `[[example]] name = "scene_flow_game"`.
   - Path points to `examples/games/scene_flow/scene_flow.rs`.

4. Created the initial scene-flow game.
   - Implemented `MenuScene`, `PlayScene`, `PauseScene`, `ResultScene`.
   - Implemented scene commands:
     - Menu Start/Enter -> `Replace(PlayScene)`.
     - Menu Quit/Esc -> `ShouldQuit(true)`.
     - Play Complete/Enter -> `Push(ResultScene)`.
     - Play Pause/P/Esc -> `Push(PauseScene)`.
     - Pause Resume/P/Esc -> `Pop`.
     - Pause Menu/M -> `Replace(MenuScene)`.
     - Result Retry/R -> `Replace(PlayScene)`.
     - Result Menu/Esc/M -> `Replace(MenuScene)`.
   - Later added `M` from Play to Menu too.

5. Added generated visual assets.
   - Used the image generation flow before this handoff.
   - Created `flow_bg.png` and `flow_badge.png`.
   - Saved under `examples/games/scene_flow/assets/`.

6. First QA found scene overlap and UI overlap problems.
   - User reported previous scenes remained visible when changing scenes.
   - User later reported UI overlap still existed.
   - Fix used explicit scene-owned entity cleanup plus overlay UI hiding/restoring.

7. Added stats display and cleanup validation.
   - `SceneFlowStats` tracks enter/exit counts for Menu/Play/Pause/Result.
   - Stats label displays current scene, overlay, enter counts, and exit counts.
   - Stats survive replace by carrying a shared handle to the new scene constructor.

8. User asked what to test.
   - Main test list was:
     - background/readability
     - keyboard transitions
     - mouse transitions
     - UI cleanup/overlay behavior
     - stats counter correctness.

9. User reported visual and interaction defects.
   - Background was smaller than the screen.
   - Font was too thin.
   - White background was uncomfortable.
   - One validation case was not possible.
   - Mouse interaction did not work.

10. Added `BackdropSystem` inside the example.
    - Removed reliance on sprite-based background for this example.
    - Draws a full-screen UI image every frame using current `ViewportSize`.
    - Uses dark clear color and dark tint to avoid bright empty areas.

11. Found direct-open asset failure.
    - Running the binary with `open target/debug/examples/scene_flow_game` initially showed magenta fallback.
    - Cause: asset paths were relative to the process working directory.
    - Fix: use `concat!(env!("CARGO_MANIFEST_DIR"), "...")` for asset paths.

12. Changed UI image/rect render order.
    - Background was being drawn after UI rects in `src/app.rs`.
    - That caused `DrawImage` background to cover button rectangles.
    - Changed rendering so `UiImageQueue` draws before `UiQueue`.
    - This is engine-wide behavior, but matches normal background-image layering.

13. Investigated mouse clicks.
    - `Computer Use` list did not include `scene_flow_game`.
    - `mcp__computer_use.get_app_state` cannot use this native binary as a target.
    - Used `screencapture`, `CGWindowListCopyWindowInfo`, `osascript`, and Swift `CGEvent` as fallback.
    - Keyboard events worked; mouse events were unreliable from automation.

14. Revisited cursor scale-factor handling.
    - Earlier plan assumed reverting scale-factor division was correct.
    - Actual runtime evidence showed the UI viewport is logical but cursor/touch coordinates can arrive as physical pixels.
    - Re-added division by `window.scale_factor()` for cursor/touch in `src/app.rs`.
    - This is a correction to the plan based on observed behavior.

15. Fixed button click semantics.
    - Existing `UiSystem` only emitted `ButtonClicked` when previous button state was `Pressed` and the current frame had `just_released`.
    - Fast/automated clicks can deliver press and release in the same frame.
    - Modified `UiSystem` to emit `ButtonClicked` on `just_pressed` inside bounds.
    - Kept the old release path for compatibility with existing pressed-state flows.

16. Added a unit test for same-frame press/release.
    - Test sets cursor to `(20,20)`, presses and releases left mouse in the same frame, runs `UiSystem`, and asserts a `ButtonClicked` event is emitted.
    - Test name: `button_click_handles_press_and_release_in_same_frame`.

17. Ran full verification.
    - `cargo fmt --check` passed after applying `cargo fmt`.
    - `cargo check --example scene_flow_game` passed.
    - `cargo test` passed.
    - `cargo clippy --all-targets -- -D warnings` passed.
    - `cargo package --allow-dirty --list` included scene-flow source and assets.
    - `cargo package --allow-dirty` passed after rerun with escalated network access.

18. User asked to run the game and reported it did not run.
    - `cargo run --example scene_flow_game` showed a live process:
      - `scene_flow_game[34351:5071634]`
      - AppKit/LaunchServices messages included `Connection invalid`.
    - CGWindow list showed no `scene_flow_game` window for that process.
    - Killed it with `pkill -x scene_flow_game`.
    - Ran `cargo build --example scene_flow_game`.
    - Opened the built binary with `open /Users/jkl/Projects/skeleton-engine/target/debug/examples/scene_flow_game`.
    - Confirmed an on-screen window existed.

## Key Decisions

- Use a separate file for the game example:
  - User explicitly decided `examples/games/scene_flow/scene_flow.rs`.
  - This follows the repository convention that `examples/` top-level files are feature demos and `examples/games/` contains playable games.

- Do not change the public engine API for the scene-flow example.
  - Any friction should be documented rather than solved through API churn unless clearly necessary.
  - Engine changes made here are bug fixes/behavior fixes, not new public APIs.

- Use `UiImageQueue` for the background rather than world `Sprite`.
  - Reason: the defect was screen-space coverage, not world-space gameplay composition.
  - `ViewportSize` makes full-screen coverage direct.

- Keep generated PNG assets, but treat them as mood/supporting assets.
  - Functionality validation remains scene flow, UI, cleanup, and input.

- Render UI images before UI rects.
  - Rejected alternative: keep render order and push background as a rect-only tint.
  - Reason: examples and engine users need image-backed UI backgrounds that do not cover interactive rects.

- Use absolute compile-time asset paths for this example.
  - Rejected alternative: depend on current working directory.
  - Reason: direct binary launch via `open` is currently the reliable way to show the window on macOS in this environment.

- Treat `Computer Use` direct app targeting as unavailable for this binary.
  - Reason: it is not a `.app` bundle and does not appear in the app list.
  - Fallback is screen capture plus OS-level events.

- Correct the plan's cursor scale-factor assumption.
  - The plan said to revert scale-factor division.
  - Runtime observation suggested logical viewport vs physical cursor mismatch still mattered.
  - Current code divides cursor/touch coordinates by `window.scale_factor()`.

- Emit button click on press-in-rect.
  - Rejected alternative: only handle release after prior pressed state.
  - Reason: same-frame press/release and automation can otherwise miss clicks; checkbox/slider already operate on press semantics.

- Leave `cargo run` launch behavior as an open runtime issue.
  - Direct `open` works.
  - The root cause of cargo-run process-without-window on this Mac session is not fully diagnosed.

## Evidence & Data

### Git state at handoff time

| Command | Result |
|---|---|
| `git branch --show-current` | `docs/english-conversion` |
| `git status -s` | `M Cargo.toml`; `M docs/NEXT_WORK.md`; `M src/app.rs`; `M src/ui/system.rs`; `?? examples/games/scene_flow/` |
| `git log --oneline -20` latest | `455f9d4 feat: add platformer playable example` |

### Diff stat at handoff time

```text
 Cargo.toml        |  4 ++++
 docs/NEXT_WORK.md | 16 ++++++++--------
 src/app.rs        | 40 ++++++++++++++++++++++++++++------------
 src/ui/system.rs  | 35 +++++++++++++++++++++++++++++++++--
 4 files changed, 73 insertions(+), 22 deletions(-)
```

Note: `git diff --stat` did not include untracked `examples/games/scene_flow/`; it is present in `git status -s`.

### Line counts

| File | Lines |
|---|---:|
| `examples/games/scene_flow/scene_flow.rs` | 677 |
| `src/app.rs` | 3144 |
| `src/ui/system.rs` | 577 |
| `docs/NEXT_WORK.md` | 51 |
| `Cargo.toml` | 111 |
| `plans/handoffs/HANDOFF_platformer-example-game_2026-05-30.md` | 504 |

### Verification commands already run

| Command | Result |
|---|---|
| `cargo fmt --check` | passed after running `cargo fmt` |
| `cargo check --example scene_flow_game` | passed |
| `cargo test ui::system::tests::button_click_handles_press_and_release_in_same_frame` | passed |
| `cargo test` | passed: 236 unit tests; doctests 32 passed, 19 ignored |
| `cargo clippy --all-targets -- -D warnings` | passed |
| `cargo package --allow-dirty --list` | passed and listed scene-flow source/assets |
| `cargo package --allow-dirty` | passed after escalated network access |

### `cargo package --allow-dirty` evidence

```text
Packaged 114 files, 2.0MiB (1.1MiB compressed)
Verifying skeleton-engine v1.0.0 (/Users/jkl/Projects/skeleton-engine/target/package/skeleton-engine-1.0.0)
Compiling skeleton-engine v1.0.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.84s
```

### Package list evidence

`cargo package --allow-dirty --list` included:

```text
examples/games/scene_flow/assets/flow_badge.png
examples/games/scene_flow/assets/flow_bg.png
examples/games/scene_flow/scene_flow.rs
```

It also included `.DS_Store` entries:

```text
examples/games/.DS_Store
examples/games/platformer/.DS_Store
```

Do not delete them automatically unless the user asks; they were observed but are not part of this task.

### Runtime launch evidence

`cargo run --example scene_flow_game` compiled and started:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.72s
Running `target/debug/examples/scene_flow_game`
```

Then process output included:

```text
2026-05-31 00:48:17.622 scene_flow_game[34351:5071634] Failure on line 688 in function id scheduleApplicationNotification(...)
2026-05-31 00:48:17.706 scene_flow_game[34351:5071634] Error received in message reply handler: Connection invalid
2026-05-31 00:48:17.706 scene_flow_game[34351:5071658] Connection Invalid error for service com.apple.hiservices-xpcservice.
```

`CGWindowListCopyWindowInfo` found no `scene_flow_game` window for that cargo-run process.

### Direct open evidence

After:

```bash
cargo build --example scene_flow_game
open /Users/jkl/Projects/skeleton-engine/target/debug/examples/scene_flow_game
```

Window list showed:

```text
pid=35529 owner=scene_flow_game name=skeleton-engine scene flow game bounds={
    Height = 572;
    Width = 960;
    X = 237;
    Y = 116;
}
```

### User-facing test checklist last provided

1. Background/readability:
   - no white right/bottom gap
   - background fills viewport
   - button/text readable over background
2. Keyboard:
   - `Enter`: Menu -> Play
   - `P`: Play -> Pause
   - `P` or `Esc`: Pause -> Play
   - `Enter`: Play -> Result
   - `R`: Result -> Play
   - `Esc`: Result -> Menu
   - `M`: Play/Pause/Result -> Menu
3. Mouse:
   - `Start`: Menu -> Play
   - `Pause`: Play -> Pause
   - `Resume`: Pause -> Play
   - `Complete Mission`: Play -> Result
   - `Retry`: Result -> Play
   - `Menu`: Pause/Result -> Menu
4. Cleanup:
   - overlay should not leave duplicate UI
   - Play buttons should not be clickable through overlay
   - stats counters should progress plausibly

## Code Analysis

- `scene_flow.rs::clicked(world)` reads `Events<UiEvent>` and collects all `ButtonClicked(Entity)` events for the current frame.
- `scene_flow.rs::key_pressed(world, key)` reads `InputState::just_pressed(key)`.
- `scene_flow.rs::add_button()` creates a centered `UiNode` with fixed size `360 x 68`, z `0.94`, and a `Button` with larger `font_size = 28.0`.
- `scene_flow.rs::add_modal_scrim()` uses a disabled `Button` as a dark modal scrim at z `0.95`.
- Overlay buttons are z `0.97`, overlay titles/stats z `0.96`.
- Base scene UI nodes are hidden before overlay UI is spawned, so the overlay has an explicit input/visual layer.
- `BackdropSystem` runs before `UiSystem` in base scenes.
- Because `BackdropSystem` pushes `DrawImage` and `DrawRect`, the engine render order in `src/app.rs` matters: images must render before rects.
- `src/app.rs` now drains `UiImageQueue` and renders images first, then drains `UiQueue` and renders rects.
- `src/app.rs::WindowEvent::CursorMoved` now stores logical coordinates:
  - `position.x as f32 / scale_factor`
  - `position.y as f32 / scale_factor`
- `src/app.rs::WindowEvent::Touch` applies the same logical-coordinate conversion before updating `TouchState` and emulated mouse input.
- `src/ui/system.rs` button pass now computes `started_in_rect = in_rect && just_pressed`.
- `src/ui/system.rs` emits `ButtonClicked` if:
  - `started_in_rect`, or
  - `in_rect && just_released && prev == ButtonState::Pressed`.
- The new unit test uses crate-private `InputState::set_cursor`, `press_mouse`, and `release_mouse`, so it lives inside `src/ui/system.rs` test module where crate privacy allows access.
- `WindowConfig` is reinserted in every scene `on_enter()` and once after `app.set_scene()` in `main()`, because scene reload can reset resources.

## Files Changed

### Source code

- `examples/games/scene_flow/scene_flow.rs`
  - New playable scene-flow example.
  - Contains `MenuScene`, `PlayScene`, `PauseScene`, `ResultScene`.
  - Contains `SceneFlowStats`.
  - Contains `BackdropSystem`.
  - Contains UI helper functions and cleanup helpers.

- `src/app.rs`
  - UI image queue now renders before UI rect queue.
  - Cursor and touch event positions are normalized by `window.scale_factor()`.

- `src/ui/system.rs`
  - Button clicks are emitted on press-in-rect.
  - Release-after-pressed behavior is retained.
  - Added regression test for same-frame press/release.

### Config

- `Cargo.toml`
  - Added `scene_flow_game` example target.
  - Existing `include = ["examples/games/**", ...]` already covers new assets/source.

### Documentation

- `docs/NEXT_WORK.md`
  - Marks Scene-flow game as done.
  - Updates context to list both platformer and scene-flow playable examples.
  - Updates recommended next order to `B / C / D`, then `F`.
  - Records the surfaced gap about cross-scene diagnostics/state across `Replace`.

### Assets

- `examples/games/scene_flow/assets/flow_bg.png`
  - Generated background asset used by `BackdropSystem`.

- `examples/games/scene_flow/assets/flow_badge.png`
  - Generated badge asset displayed near top-left.

### Handoffs

- `plans/handoffs/HANDOFF_scene-flow-ui-interaction_2026-05-31.md`
  - This file.

## User Feedback & Preferences

- User asked to check documented next work first.
- User asked to build a plan from the confirmed next-work document.
- User clarified that `examples` are feature-specific examples, and the game should have its own separate file.
- User explicitly selected `examples/games/platformer/platformer.rs` for the earlier platformer game.
- User then accepted the scene-flow plan and asked to implement it.
- User repeatedly asked for ambiguity checks until ambiguity was low enough.
- User wanted `scene_flow_game` to be tested as an actual game, not just compiled.
- User reported “scene 넘어 갈 때 이전 scene이 남아있는 문제가 있어”.
- User reported “여전히 ui 곂침 문제 있어”.
- User later confirmed “ui 곂침 해결”.
- User asked for the key things to test in the game.
- User reported the background was too small for the screen.
- User reported the font weight was too thin and hard to read.
- User reported white background was uncomfortable.
- User reported validation item 6 was not possible.
- User reported mouse interaction did not work.
- User specifically asked to use Computer Use for testing.
- User reported the background was still a problem.
- User reported `M` key did not work.
- User reported buttons did not respond to mouse clicks.
- User asked to run the game.
- User reported “게임 실행 안됨” after `cargo run --example scene_flow_game`.
- User prefers direct, factual Korean updates with actionable next steps.
- User expects implementation and verification, not only planning, once the plan is accepted.

## Where We're Going

1. First next action: investigate the `cargo run --example scene_flow_game` no-window behavior.
   - Direct `open target/debug/examples/scene_flow_game` works.
   - Need determine whether this is local macOS activation behavior, terminal/session state, or a deeper winit/App lifecycle issue.

2. Re-run manual QA with the direct-open window.
   - User should check keyboard and mouse flows.
   - If mouse still fails with real user clicks, inspect `InputState::cursor()` values or add temporary debug output with user approval.

3. If manual QA passes, stage and commit only if user asks.
   - Do not commit automatically.
   - Likely commit message: `feat: add scene flow playable example`.

4. If mouse QA fails, focus next on input coordinate conversion.
   - Current code divides cursor/touch by scale factor.
   - Verify against actual user click positions, not only synthetic `osascript`/`CGEvent` clicks.

5. Consider whether `docs/NEXT_WORK.md` needs one more gap note.
   - Possible gap: direct example binary launch behavior under `cargo run` on this Mac environment.
   - Do not add unless it is reproducible beyond this local session.

6. After scene-flow is accepted, next work from `docs/NEXT_WORK.md` is `B / C / D`.
   - B: top-down maze escape
   - C: puzzle grid/Sokoban
   - D: simple shooter

## Risks & Blockers

- `cargo run --example scene_flow_game` can start a live process without an on-screen window in this desktop environment.
- `Computer Use` cannot directly target `scene_flow_game` because it is not a `.app` bundle.
- Synthetic mouse automation is not a reliable substitute for real user clicks on this native window.
- Engine-wide render-order change (`UiImageQueue` before `UiQueue`) is likely correct, but it can affect future apps that assumed UI images draw above rects.
- Engine-wide button behavior change now fires `ButtonClicked` on press instead of only release; this matches checkbox/slider press semantics, but it is a behavior change.
- `.DS_Store` files appear in `cargo package --allow-dirty --list`; unrelated to this task but visible in package contents.

## Open Questions

- Does a real user mouse click now activate `Start`, `Pause`, `Resume`, `Complete Mission`, `Retry`, and `Menu` after the latest coordinate/button changes?
- Why does `cargo run --example scene_flow_game` create a process but no visible window in this session?
- Should the engine keep press-based button click semantics long-term, or should it track press-origin per button and emit on release only?
- Should example launch instructions mention direct `open` fallback on macOS, or is this only local environment noise?
- Should `.DS_Store` files be removed from package contents in a separate hygiene task?

## Quick Start for Next Session

```bash
# Restore context
cd /Users/jkl/Projects/skeleton-engine

# Reference docs
sed -n '1,220p' AGENTS.md
sed -n '1,120p' docs/NEXT_WORK.md
sed -n '1,220p' plans/handoffs/HANDOFF_scene-flow-ui-interaction_2026-05-31.md

# Key files to read first
sed -n '1,220p' examples/games/scene_flow/scene_flow.rs
sed -n '220,700p' examples/games/scene_flow/scene_flow.rs
sed -n '2360,2410p' src/app.rs
sed -n '2738,2810p' src/app.rs
sed -n '65,105p' src/ui/system.rs

# Verify current state
git status -s
cargo fmt --check
cargo check --example scene_flow_game
cargo test ui::system::tests::button_click_handles_press_and_release_in_same_frame

# Launch workaround that showed a window
cargo build --example scene_flow_game
open /Users/jkl/Projects/skeleton-engine/target/debug/examples/scene_flow_game

# Confirm the window exists
swift -e 'import CoreGraphics; if let info = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as? [[String: Any]] { for w in info { let owner = String(describing: w[kCGWindowOwnerName as String] ?? ""); if owner.contains("scene_flow_game") { print(w) } } }'

# Next action
# Ask the user to manually test the direct-open game window, especially mouse clicks.
```
