# Platformer playable example game added and fully verified

**Date:** 2026-05-30
**Status:** COMPLETED
**Bead(s):** none
**Epic:** post-v1.0 playable example validation
**Chain:** `platformer-example-game` seq `1`
**Parent:** `none — first in chain`
**Prior chain:** none — first in chain

---

## Reference Documents

- `AGENTS.md` — project quick reference, module map, default agent workflow, documentation language policy.
- `CLAUDE.md` — parallel quick reference for Claude-family agents.
- `docs/VISION.md` — current project direction: fork-friendly 2D skeleton, breadth-first feature coverage, feature acceptance through playable examples.
- `docs/NEXT_WORK.md` — living next-work candidates; updated this session to mark Platformer done.
- `docs/PATTERNS.md` — ECS/query, render layering, UI/system order, physics access, task recipes.
- `docs/AGENT_WORKFLOW.md` — detailed agent operating rules.
- `REFERENCE.html` — full public API reference.
- `ARCHITECTURE.html` — engine structure reference.
- `docs/HANDOFF.md` — historical development handoff/dev history for earlier phases.

## The Goal

The user wanted the next documented work plan implemented, specifically the first
playable example game from `docs/NEXT_WORK.md`: a platformer that validates engine APIs
through actual play instead of isolated feature demos. During planning, the user clarified
that `examples/` top-level files are feature demos and playable games should live in a
separate nested path. The final target became a runnable example game at
`examples/games/platformer/platformer.rs`, with generated pixel-art assets beside it and
full verification completed.

## Where We Are

- Current branch from `git branch --show-current`: `docs/english-conversion`.
- Worktree is intentionally dirty and uncommitted.
- `git status -s` before writing this handoff showed:
  - `M Cargo.toml`
  - `M docs/NEXT_WORK.md`
  - `?? examples/games/`
- `plans/handoffs/` was created by this handoff request and now contains this file.
- `Cargo.toml` now includes nested example game files in the package include list:
  - added `"examples/games/**"`.
- `Cargo.toml` now declares a new explicit example target:
  - `name = "platformer_game"`
  - `path = "examples/games/platformer/platformer.rs"`
- `cargo metadata --no-deps --format-version 1` confirms `platformer_game` is registered as an example target with source path `/Users/jkl/Projects/skeleton-engine/examples/games/platformer/platformer.rs`.
- New playable game source exists:
  - `examples/games/platformer/platformer.rs`
  - line count at handoff time: 622 lines.
- New generated game assets exist:
  - `examples/games/platformer/assets/player_atlas.png`
  - `examples/games/platformer/assets/tiles.png`
  - `examples/games/platformer/assets/goal.png`
- Asset dimensions and transparency were verified with Pillow:
  - `player_atlas.png`: `(256, 256)`, alpha extrema `(0, 255)`, top-left alpha `0`.
  - `tiles.png`: `(128, 128)`, alpha extrema `(0, 255)`, top-left alpha `0`.
  - `goal.png`: `(64, 64)`, alpha extrema `(0, 255)`, top-left alpha `0`.
- Asset SHA-256 hashes at handoff time:
  - `player_atlas.png`: `41f95bf41264273f84d460aea80bb3a0590d237d39f4f91cbadd223b20888382`
  - `tiles.png`: `de802b9cfcc9f6f3c50ab0409d155a869f48fe8ff147e190d6086be718479903`
  - `goal.png`: `d6b686fb7bbfc68192e72b6048ed5e5093e5863ad7fd1afa5040856c80b944ef`
- `docs/NEXT_WORK.md` now says playable example games live under `examples/games/`.
- `docs/NEXT_WORK.md` marks candidate A, Platformer, as done.
- `docs/NEXT_WORK.md` now identifies next recommended work as Scene flow first, then B/C/D.
- `docs/NEXT_WORK.md` records surfaced gaps:
  - one-way platforms remain future work.
  - tilemap-to-physics binding still wants a higher-level ergonomic helper.
- The example uses `PhysicsWorld`, `PhysicsSystem`, `PhysicsBody`, `CharacterController`, and `PhysicsWorld::move_character`.
- The example uses a goal sensor and reads `Events<TriggerEvent>` to set the win state.
- The example uses `AnimationPlayer`, `AnimationClip`, `AnimationSystem`, `AnimationStateMachine`, `StateMachineSystem`, `TransitionCond`, and `UvRect`.
- The example uses `AtlasSprite` and `App::load_atlas` for player and tile atlas rendering.
- The example uses `Sprite::textured_with_handle` and `App::load_image` for the goal portal image.
- The example uses a camera anchor entity so existing `Camera::follow_entity` follows a top-left camera target rather than centering on the player incorrectly.
- Runtime controls in the example:
  - Move: `A/D` or arrow keys.
  - Jump: `Space`, `W`, or `ArrowUp`.
  - Restart: `R`.
  - Quit: `Escape`.
- Game states:
  - `Playing`
  - `Won`
  - `Failed`
- Win condition:
  - player intersects the goal sensor and `TriggerEvent::Entered` is observed.
- Fail condition:
  - player falls below `FALL_Y = 620.0`.
- Restart behavior:
  - `PlatformerPhysicsSystem::reset_player` resets the Rapier body translation, transform position, controller grounded flag, and session timers/velocity.
- Public engine API was not changed.
- No dependency version changes were made.
- No commits, staging, or pushes were performed.
- The user later requested the game be launched; `cargo run --example platformer_game` was started and reported `Running target/debug/examples/platformer_game`.
- Process inspection after the handoff request was noisy/restricted:
  - `pgrep -fl platformer_game` failed with `sysmon request failed with error: sysmond service not found`.
  - `ps ax | rg platformer_game` failed due `operation not permitted`.
  - Full `ps ax -o pid,command` output was too large/truncated; no reliable platformer status was extracted from it.

## What We Tried (Chronological)

1. Looked for existing planned work in docs.
   - Searched `docs`, `README.md`, `AGENTS.md`, and `CLAUDE.md` for next/todo/plan keywords.
   - Found `docs/NEXT_WORK.md` as the active living next-work document.
   - Confirmed `docs/ROADMAP.md` is historical, not active next work.

2. Read the active vision and next-work docs.
   - `docs/VISION.md` says feature work should be validated through small playable examples.
   - `docs/NEXT_WORK.md` recommended Platformer first, then Scene flow, then B/C/D.
   - Entity Generation v2 and dependency security follow-up were confirmed cancelled/archived.

3. Planned the work in Plan Mode with user input.
   - Initial plan placed a playable platformer example under `examples/games/platformer.rs`.
   - User corrected that top-level `examples/` is for feature demos and asked whether a separate game file would be better.
   - User selected `examples/games` as the location category.
   - User later corrected the exact final path to `examples/games/platformer/platformer.rs`.

4. Asked and locked product/implementation choices.
   - Collision/movement: user selected engine physics first.
   - Playable depth: user selected a short one-stage platformer.
   - Visual assets: user selected small PNG assets.
   - PNG source: user selected image generation.
   - Asset location: user selected `examples/games/platformer/assets`.
   - Animation validation: user selected real atlas animation.
   - Asset style: user selected bright pixel art.
   - Atlas shape: user selected 4x4 atlas.
   - Docs status: user selected Done + gaps.
   - Level data: user selected code constants.
   - Goal detection: user selected physics sensor.
   - Transparency: user selected chroma-key removal.

5. Generated image assets with the `imagegen` skill.
   - Used built-in `image_gen`, not CLI fallback.
   - Generated a 4x4 player atlas on green chroma-key.
   - Generated a tile atlas on magenta chroma-key.
   - Generated a goal portal on green chroma-key.
   - Generated files initially lived under `/Users/jkl/.codex/generated_images/019e78ee-e5a4-7fe1-a793-4787a455e81e/`.

6. Normalized generated assets for project use.
   - Used bundled Python with Pillow from `/Users/jkl/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3`.
   - Removed approximate chroma-key backgrounds.
   - Resized with nearest-neighbor:
     - player atlas to 256x256.
     - tiles to 128x128.
     - goal to 64x64.
   - Saved final assets under `examples/games/platformer/assets/`.

7. Wrote the playable platformer example.
   - Added source file with local component/resource/system types.
   - Implemented `PlatformerSession`, `PlatformerPhysicsSystem`, `GoalSystem`, `HudSystem`, and `CameraAnchorSystem`.
   - Used a wrapped `PhysicsSystem` because `PhysicsWorld::move_character` must operate on the same physics world that later steps and emits events.

8. First compile attempt found real integration issues.
   - Command: `cargo check --example platformer_game`.
   - Errors:
     - `cannot find module or crate nalgebra` from `vector!` macro.
     - `PhysicsWorld::new()` required a gravity `Vec2`.
     - borrow conflicts around mutably borrowing `AnimationStateMachine` while reading session/controller state.
   - Fixes:
     - imported `rapier2d::na as nalgebra`.
     - changed `PhysicsWorld::new()` to `PhysicsWorld::new(Vec2::ZERO)`.
     - computed animation parameters before taking the mutable state-machine borrow.

9. Second compile attempt found camera construction issue.
   - Error: private fields prevented `Camera { ..Default::default() }` struct update outside the crate.
   - Fix:
     - used `Camera::new(Vec2::ZERO, 1.0)`.
     - then set public fields `follow_entity` and `lerp_factor`.

10. Realized direct `follow_entity = player` would use the player position as camera top-left.
    - Existing camera coordinates are top-left anchored.
    - Added invisible `camera_anchor` entity.
    - `CameraAnchorSystem` sets the anchor to `player_pos - viewport_half`, with simple clamps.
    - `Camera::follow_entity` follows the anchor instead of the player.

11. Updated docs.
    - Changed `docs/NEXT_WORK.md` context from “no playable game yet” to first playable game available.
    - Marked Platformer done.
    - Moved Scene flow to first recommended next item.
    - Recorded the two surfaced gaps.

12. Ran full verification.
    - `cargo fmt --check` passed.
    - `cargo check --example platformer_game` passed.
    - `cargo test` passed.
    - `cargo clippy --all-targets -- -D warnings` passed.
    - `cargo package --allow-dirty --list` showed new game source/assets included.
    - `cargo package --allow-dirty` first hit DNS/network failure in sandbox, then passed after escalated network approval.
    - `cargo run --example platformer_game` launched successfully and held for several seconds without immediate crash during smoke checks.
    - `cargo test --all-targets` was run later as an extra audit and passed.

13. User requested “게임 실행 해줘”.
    - Ran `cargo run --example platformer_game`.
    - Output:
      - `Finished dev profile...`
      - `Running target/debug/examples/platformer_game`
    - Final response told the user the window was running and listed controls.
    - No stop was requested after that before this handoff request.

14. User invoked `$handoff`.
    - Read `/Users/jkl/.codex/skills/handoff/SKILL.md`.
    - Chose Deep mining pass due substantial coding/research and many tool calls.
    - Read `references/mining-deep-chunked.md`, `references/output-template.md`, and `references/validation.md`.
    - Created `plans/handoffs/`.
    - Wrote this handoff file.

## Key Decisions

- Put playable games under `examples/games/`, not top-level `examples/`.
  - Reason: user clarified top-level examples are feature demos.
  - Exact path chosen by user: `examples/games/platformer/platformer.rs`.

- Use an explicit Cargo example target named `platformer_game`.
  - Reason: Cargo does not automatically expose nested `examples/games/platformer/platformer.rs` as the desired example name.
  - Alternative rejected: top-level `examples/platformer_game.rs`.

- Keep public engine API unchanged.
  - Reason: user wanted a playable example and the planning phase locked “record API gaps, do not change API.”
  - Result: surfaced gaps were documented instead of adding helpers.

- Use engine physics first.
  - Reason: user selected this and `docs/NEXT_WORK.md` wanted validation of `CharacterController` and movement API.
  - Implementation: example-specific system owns/wraps `PhysicsSystem` so it can call `move_character` before stepping.

- Use a physics sensor for the goal.
  - Reason: user selected `Physics sensor`.
  - Benefit: validates `TriggerEvent` and sensor intersection, not just local AABB math.

- Use code constants for the level.
  - Reason: user selected code constants.
  - Rejected alternatives: ASCII map and RON/JSON file. Those would add parser/data-loading complexity.

- Use generated bright pixel-art PNG assets.
  - Reason: user selected image generation and bright pixel-art style.
  - Rejected alternative: programmatic deterministic pixel art. It would be more reproducible but less visually polished.

- Use a 4x4 player atlas.
  - Reason: user selected 4x4, one row per state.
  - Mapping used:
    - row 0: idle
    - row 1: run
    - row 2: jump
    - row 3: fall

- Use chroma-key removal for transparency.
  - Reason: user selected chroma-key removal.
  - Implementation used local Pillow processing rather than the skill helper script, because the generated background had gradients and approximate color thresholds were needed.

- Use a camera anchor entity.
  - Reason: engine camera position is top-left anchored; following the player directly would place the player near screen top-left.
  - This keeps existing `Camera::follow_entity` validation while making gameplay framing usable.

- Update `docs/NEXT_WORK.md`, not `docs/HANDOFF.md`.
  - Reason: `NEXT_WORK` is the living plan for current playable example direction; `docs/HANDOFF.md` is historical dev history.

## Evidence & Data

### Git state

| Evidence | Result |
|---|---|
| Branch | `docs/english-conversion` |
| Status | `M Cargo.toml`, `M docs/NEXT_WORK.md`, `?? examples/games/` |
| Commit performed? | No |
| Stage performed? | No |
| Push performed? | No |

### Recent commit context

| Hash | Summary |
|---|---|
| `05b2915` | Add audio channel playback state |
| `4c55c12` | docs: extract patterns to docs/PATTERNS.md, keep quick refs <=200 lines |
| `c7f63d3` | docs: convert doc prose to English + add documentation-language rule |
| `8b3b778` | feat: add 2D cutout skeletal animation; archive two cancelled plans |
| `f6555da` | docs: record next-work candidates and vision alignment check |
| `67068ce` | docs: reset and document project vision |

### Verification matrix

| Command | Result | Notes |
|---|---|---|
| `cargo fmt --check` | passed | no output |
| `cargo check --example platformer_game` | passed | example target compiles |
| `cargo test` | passed | 235 unit tests; 32 doc tests passed; 19 ignored |
| `cargo clippy --all-targets -- -D warnings` | passed | no warnings |
| `cargo package --allow-dirty --list | rg "examples/games/platformer|Cargo.toml|docs/NEXT_WORK"` | passed | showed source and all 3 PNG assets included |
| `cargo package --allow-dirty` | passed after network approval | packaged 109 files, 1.3MiB, 418.0KiB compressed; verification compile passed |
| `cargo run --example platformer_game` | passed smoke | launched and stayed running for several seconds |
| `cargo test --all-targets` | passed | includes example test harnesses; `platformer_game` ran 0 tests successfully |

### First package attempt failure

Raw failure mode before escalation:

```text
Packaging skeleton-engine v1.0.0
Updating crates.io index
warning: spurious network error ... Couldn't resolve host name (Could not resolve host: index.crates.io)
```

Resolution:

```text
cargo package --allow-dirty
Packaged 109 files, 1.3MiB (418.0KiB compressed)
Verifying skeleton-engine v1.0.0 (.../target/package/skeleton-engine-1.0.0)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.57s
```

### Compile errors encountered and fixed

| Attempt | Error | Fix |
|---|---|---|
| first `cargo check --example platformer_game` | `cannot find module or crate nalgebra` from `vector!` macro | `use rapier2d::na as nalgebra;` |
| first `cargo check --example platformer_game` | `PhysicsWorld::new()` missing gravity arg | `PhysicsWorld::new(Vec2::ZERO)` |
| first `cargo check --example platformer_game` | immutable borrow while `AnimationStateMachine` mutably borrowed | compute `running` and `grounded_now` before `world.get_mut::<AnimationStateMachine>` |
| second check | private `Camera` fields with struct update syntax | use `Camera::new`, then set public fields |

### Asset evidence

| File | Dimensions | Alpha extrema | Top-left alpha | SHA-256 |
|---|---:|---:|---:|---|
| `examples/games/platformer/assets/player_atlas.png` | 256x256 | `(0, 255)` | `0` | `41f95bf41264273f84d460aea80bb3a0590d237d39f4f91cbadd223b20888382` |
| `examples/games/platformer/assets/tiles.png` | 128x128 | `(0, 255)` | `0` | `de802b9cfcc9f6f3c50ab0409d155a869f48fe8ff147e190d6086be718479903` |
| `examples/games/platformer/assets/goal.png` | 64x64 | `(0, 255)` | `0` | `d6b686fb7bbfc68192e72b6048ed5e5093e5863ad7fd1afa5040856c80b944ef` |

### Package include evidence

`cargo package --allow-dirty --list | rg "examples/games/platformer|Cargo.toml|docs/NEXT_WORK"` showed:

```text
Cargo.toml
Cargo.toml.orig
examples/games/platformer/assets/goal.png
examples/games/platformer/assets/player_atlas.png
examples/games/platformer/assets/tiles.png
examples/games/platformer/platformer.rs
```

### Cargo metadata evidence

`cargo metadata --no-deps --format-version 1` includes a target:

```text
kind: ["example"]
name: "platformer_game"
src_path: "/Users/jkl/Projects/skeleton-engine/examples/games/platformer/platformer.rs"
```

### Line counts

| File | Lines |
|---|---:|
| `AGENTS.md` | 109 |
| `CLAUDE.md` | 148 |
| `docs/NEXT_WORK.md` | 51 |
| `examples/games/platformer/platformer.rs` | 622 |
| `Cargo.toml` | 107 |

## Code Analysis

- `PlatformerSession` is the central example runtime resource. It stores `player`, `goal`, `camera_anchor`, `status`, `velocity`, `coyote_timer`, and `jump_buffer_timer`.
- `PlatformerPhysicsSystem` wraps `PhysicsSystem` instead of registering a stock `PhysicsSystem` separately. This is important because `PhysicsWorld::move_character` must be called on the same world before `PhysicsSystem::run` steps and syncs transforms.
- `reset_player` uses Rapier body methods:
  - `set_translation(vector![...], true)`
  - `set_next_kinematic_translation(vector![...])`
  - then updates ECS `Transform` and `CharacterController`.
- Movement constants are local to the example:
  - `PPU = 64.0`
  - `MOVE_ACCEL = 2600.0`
  - `GROUND_DECEL = 3200.0`
  - `AIR_DECEL = 900.0`
  - `MAX_SPEED_X = 270.0`
  - `GRAVITY = 1450.0`
  - `JUMP_SPEED = 560.0`
  - `COYOTE_TIME = 0.10`
  - `JUMP_BUFFER = 0.11`
- Level geometry is the `PLATFORMS` constant: tuples of `(x, y, width, height, tile_index)`.
- The fail boundary is `FALL_Y = 620.0`.
- `GoalSystem` reads `Events<TriggerEvent>` and only treats `TriggerEvent::Entered(a, b)` as success if one entity is the player and one is the goal.
- `HudSystem` uses `TextQueue` and `DrawText` for instructions and status messages.
- `CameraAnchorSystem` updates the invisible anchor to `player_pos - Vec2::new(WINDOW_W, WINDOW_H) * 0.5`, clamps x to at least `0.0`, and clamps y into `0.0..=120.0`.
- Animation clip rows are generated by `frames(row)` with `UvRect::from_grid(col, row, 4, 4)`.
- State machine transitions use params:
  - bool `is_running`
  - bool `is_grounded`
  - float `vertical_velocity`
  - trigger `jump`
- System registration order:
  - `PlatformerPhysicsSystem`
  - `GoalSystem`
  - `CameraAnchorSystem`
  - `AnimationSystem`
  - `StateMachineSystem`
  - `HudSystem`

## Files Changed

### Source code

- `examples/games/platformer/platformer.rs` — new playable platformer game example. Implements input, physics movement, goal sensor, animation state machine, camera follow anchor, HUD text, win/fail/restart loop.

### Assets

- `examples/games/platformer/assets/player_atlas.png` — generated bright pixel-art player 4x4 atlas, transparent background.
- `examples/games/platformer/assets/tiles.png` — generated bright pixel-art tile atlas, transparent background.
- `examples/games/platformer/assets/goal.png` — generated bright pixel-art portal/goal sprite, transparent background.

### Config

- `Cargo.toml` — added package include for `examples/games/**`; added explicit `[[example]]` target `platformer_game`.

### Documentation

- `docs/NEXT_WORK.md` — updated context to say playable games are under `examples/games/`; marked Platformer done; moved Scene flow to next recommended step; recorded surfaced gaps.
- `plans/handoffs/HANDOFF_platformer-example-game_2026-05-30.md` — this handoff.

## User Feedback & Preferences

- User asked: “문서에 남겨져 있는 다음 작업 계획 있는지 확인해줘”.
- User accepted that current active next work was from `docs/NEXT_WORK.md`.
- User asked to build a plan from confirmed documentation.
- User corrected that top-level `examples` are feature examples and asked whether example games should be a separate file/category.
- User selected `examples/games` as the preferred location category.
- User explicitly corrected final game source path to `examples/games/platformer/platformer.rs`.
- User repeatedly asked to reduce ambiguity by asking more questions before implementation.
- User selected engine physics-first collision/movement.
- User selected short one-stage playable depth.
- User selected small PNG assets.
- User selected image generation for PNG source.
- User selected `examples/games/platformer/assets` as asset location.
- User selected atlas animation for `AnimationStateMachine` validation.
- User selected bright pixel-art style.
- User selected a 4x4 player atlas.
- User selected Done + gaps for `docs/NEXT_WORK.md`.
- User selected code constants for level data.
- User selected physics sensor for goal detection.
- User selected chroma-key removal for transparency.
- User asked to start work once Plan Mode ended.
- User asked for “전체 검증 완료까지 진행”; this was treated as a persistent goal and completed after full verification.
- User asked “게임 실행 해줘”; the example was launched with `cargo run --example platformer_game`.
- User invoked `$handoff` to preserve session state.

## Where We're Going

1. If continuing this exact work, inspect the running game manually and decide whether the platformer feel or camera framing needs tuning.
2. If no tuning is needed, stage and commit the current changes only if the user asks.
3. Next planned engine example work per `docs/NEXT_WORK.md` is **E — Scene flow**.
4. Future platformer follow-up gaps are:
   - one-way platforms.
   - ergonomic tilemap-to-physics binding helper.
5. Keep top-level `examples/*.rs` as feature demos and put future playable games under `examples/games/<name>/`.

## Risks & Blockers

- The example game was smoke-tested for launch/no immediate crash, but a human has not yet reported full manual completion from start to goal.
- Current process state of `platformer_game` after the final user launch was not reliably verified due process-listing restrictions/noisy output.
- The generated art is acceptable and verified for dimensions/alpha, but it is AI-generated and not deterministic from source.
- `cargo package --allow-dirty` requires network access to update crates.io index in this environment; sandboxed attempt hit DNS failure.
- Worktree is dirty and uncommitted by design.

## Open Questions

- Does the user like the platformer feel after playing it?
- Should the next session tune movement/camera values or move directly to committing?
- Should generated assets be replaced later with deterministic/programmatic pixel art for reproducibility?
- Should `docs/HANDOFF.md` historical dev log get a short entry, or is `docs/NEXT_WORK.md` enough for this playable-example milestone?

## Quick Start for Next Session

```bash
# Restore context
cd /Users/jkl/Projects/skeleton-engine
git status -s
git branch --show-current

# Reference docs
sed -n '1,90p' docs/NEXT_WORK.md
sed -n '1,80p' docs/VISION.md

# Key files to read first
sed -n '1,220p' examples/games/platformer/platformer.rs
sed -n '220,460p' examples/games/platformer/platformer.rs
sed -n '460,660p' examples/games/platformer/platformer.rs
sed -n '1,70p' Cargo.toml

# Evidence / data files
ls -l examples/games/platformer/assets
shasum -a 256 examples/games/platformer/assets/*.png

# Verify current state
cargo fmt --check
cargo check --example platformer_game
cargo test --all-targets
cargo clippy --all-targets -- -D warnings

# Manual smoke
cargo run --example platformer_game

# Next action
# Ask the user whether to tune gameplay after they try it, or whether to stage/commit the completed platformer example.
```
