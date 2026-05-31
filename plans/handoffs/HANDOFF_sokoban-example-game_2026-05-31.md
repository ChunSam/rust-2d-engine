# Sokoban playable example + reusable `History<T>` undo

**Date:** 2026-05-31
**Status:** COMPLETED
**Bead(s):** none (beads/`bd` not installed in this repo)
**Epic:** Candidate C from `docs/NEXT_WORK.md` (playable examples for v1.0.0 dogfooding)
**Chain:** `sokoban-example-game` seq `1`
**Parent:** `none — first in chain`
**Prior chain:** none — first in chain

---

## Related Handoffs

Sibling work streams in the same playable-examples program — independent, not chain parents:

- `HANDOFF_platformer-example-game_2026-05-30.md` — candidate A (platformer).
- `HANDOFF_scene-flow-ui-interaction_2026-05-31.md` / `..._engine-ui-fixes_...` — candidate E (scene-flow).
- `HANDOFF_maze-escape-example-game_2026-05-31.md` — candidate B (maze escape). **Closest precedent**: same per-candidate structure, same ASCII-grid layout pattern, same "build example → surface engine gap → fix engine" loop. This session deliberately diverged from maze on rendering (see Key Decisions).

## Reference Documents

- `CLAUDE.md` — engine module map and task recipes (≤200-line agent reference; now 151 lines).
- `docs/VISION.md` — feature+example loop ("a new feature is not done until a small example game exercises it in real play").
- `docs/NEXT_WORK.md` — candidate table; C now marked done, **D (simple shooter) is the next recommended item**.
- `docs/PATTERNS.md` — extracted architecture patterns.
- `docs/HANDOFF.md` — per-phase dev history (Sokoban row appended; status line updated).

## The Goal

Ship candidate **C** under the dogfooding loop: a small playable Sokoban (box-pushing) mini-game that exercises discrete grid logic, multi-level progression, undo/redo, and progress save/load — and close whatever engine friction that surfaces by patching the engine before release. End state: `cargo run --example sokoban_game` is an actual playable puzzle, and a fork user can get undo/redo from the engine without writing their own.

Scope and proof bar were locked through a `/grill-me` interrogation before coding (see User Feedback). Locked: Sokoban (not match-3); multi-level + progress save/load; fix surfaced gaps in the engine per maze precedent; proof bar = native+wasm builds, 0 clippy, existing tests green, manual play.

## Where We Are

- Branch `feat/sokoban-example`, **based on `docs/english-conversion`** (which is 8 commits ahead of `main`). Two commits this session:
  - `eb89728 style: apply rustfmt to maze_escape example and pathfinding` — pre-existing uncommitted rustfmt-only working-tree changes, committed first to keep the feature branch clean.
  - `67fcb9c feat: add Sokoban playable example + reusable History<T> undo` — 7 files, the actual work.
- **PR #1 opened against `main`**: https://github.com/ChunSam/skeleton-engine/pull/1 — user explicitly chose `main` as base despite being told it bundles all 10 unmerged commits (English docs, platformer, scene-flow, maze-escape, audio, + sokoban). Diff vs main: +6747/-1757 across 33 files.
- Working tree clean after commit.
- New engine module `src/history.rs`: genre-agnostic snapshot `History<T>` (`record`/`undo`/`redo`, `can_undo`/`can_redo`, `undo_depth`, `clear`, `with_capacity` bounded depth). Re-exported as `engine::History` (`src/lib.rs`). 4 unit tests, all pass.
- New example `examples/games/sokoban/sokoban.rs` (~390 lines): 3 levels, push rules, undo/redo, level nav, save/load progress.
- `Cargo.toml`: registered `[[example]] name = "sokoban_game"`.
- Docs updated: `docs/NEXT_WORK.md` (C done + next=D), `docs/HANDOFF.md` (row + status), `CLAUDE.md` (module-map row for History).

## What We Tried (Chronological)

1. **`/grill-me` clarification (4 rounds)** before any code. Resolved: subgenre (Sokoban), level/save scope (multi-level + save/load), gap-fix scope (fix in engine), proof bar (build+test+manual), and — surfaced during execution — branch base and PR base. Full decision packet emitted.
2. **Researched engine for existing undo / grid / save.** Found editor-only private `EditorHistory` in `src/app.rs` (command-based, not reusable). No public game-facing undo. `save` module (`save_path`/`save`/`load_or_default`) already ergonomic. No grid-coordinate helper beyond `PathGrid`.
3. **Chose immediate-mode rendering over maze's persistent ECS sprites.** Verified `DebugDrawQueue.items: Vec<DebugRect>` is drained every frame in `app.rs` render stage (~line 2348) and converted to filled `DrawRect`s — always rendered, independent of the F1 debug toggle. Confirmed UI primitive z-sort is ascending (higher z on top) via `src/renderer/sprite.rs:500` and its test.
4. **Implemented `History<T>`** as snapshot-based (not command/diff) — simplest, fully generic over `T: Clone`, can't desync. Documented the memory-vs-simplicity trade-off in the module doc.
5. **Implemented the example**, built, hit 1 clippy warning (`needless_borrows_for_generic_args` on `&format!(...)` into `DrawText::new(impl Into<String>)`), fixed by dropping `&`.
6. **Verified** native + wasm builds (lib + example), clippy clean, fmt clean, 245 lib tests, startup smoke-run (6s, no panic).
7. **Committed, pushed, opened PR #1 against main** (after confirming the bundle implication with the user).

## Key Decisions

- **Snapshot `History<T>`, not command-based.** Generic, reusable, simple; ideal for small game state. Command/diff left as a documented alternative for large state. → the engine gap fix.
- **Immediate-mode `DebugDrawQueue` rendering, diverging from maze_escape's persistent `Sprite` entities.** Board is fully reconstructed from snapshot state each frame; level switching and undo become trivial with no entity despawn/respawn churn. This is the notable architectural departure from the B precedent.
- **Reuse `save` unchanged.** `NEXT_WORK` anticipated "progress-save API friction"; in practice `save_path` + `load_or_default::<Progress>` + `save` were friction-free. No new persistence API was added. Best-effort save (ignores `Err`) so wasm / read-only home no-ops gracefully.
- **Grid/board logic stays example-local.** Push rules and solvability are genre-specific, not engine concerns.
- **Branch base = `docs/english-conversion`**, not stale `main`, so the example inherits the `examples/games/` structure and `PATTERNS.md` it follows.
- **PR base = `main`** (user's explicit call) → 10-commit bundle.

## Evidence & Data

### Git state at handoff time

- Branch: `feat/sokoban-example` (tracking `origin/feat/sokoban-example`).
- `67fcb9c` is HEAD. Working tree clean.
- `main..feat/sokoban-example` = 10 commits (see PR body in #1 for the list).

### Diff stat of `67fcb9c`

7 files: `src/history.rs` (new, ~210 lines incl. tests), `examples/games/sokoban/sokoban.rs` (new, ~390 lines), `src/lib.rs` (+2: `pub mod history;` + `pub use history::History;`), `Cargo.toml` (+4: example entry), `docs/NEXT_WORK.md`, `docs/HANDOFF.md`, `CLAUDE.md` (module-map row).

### Engine gap closed — surface-area before/after

- Before: undo existed only as `EditorHistory` in `src/app.rs`, `pub(crate)`-private, command-based, editor-coupled. Games had to roll their own.
- After: `engine::History<T>` — public, genre-agnostic, snapshot-based. Methods: `new`, `with_capacity(cap)`, `record(snapshot)`, `undo(&mut T) -> bool`, `redo(&mut T) -> bool`, `can_undo`, `can_redo`, `undo_depth`, `clear`. `record` clears the redo branch (standard editor behavior). `Default` impl present.

### Surfaced-but-NOT-a-gap

- `save` module: zero friction. `NEXT_WORK`'s predicted "progress-save API friction" did not materialize. Recorded as such in the updated `NEXT_WORK` row.

### History semantics (verified by tests in `src/history.rs`)

- `undo_then_redo_round_trips`: record 0, record 1, undo→1, undo→0, undo→false; redo→1, redo→2.
- `recording_clears_redo_branch`: a fresh `record` after an undo invalidates redo.
- `capacity_drops_oldest_snapshots`: `with_capacity(2)` keeps only the 2 most-recent snapshots.
- `zero_capacity_never_retains`: `with_capacity(0)` makes undo a no-op.

### Sokoban level data (hand-verified solvable)

Notation: `#` wall, ` ` floor, `@` player, `$` box, `.` goal, `*` box-on-goal, `+` player-on-goal.
- L1 (7×5): one box pushed up one cell onto a goal in the top wall gap.
- L2 (9×3): `#.$@ $ .#` — left box pushed left 1 onto goal col1; right box pushed right 2 onto goal col7. Two independent sub-puzzles.
- L3 (7×6): two boxes at row3 pushed straight up two cells into top-row goals; player reaches the second box's underside via the open row2 / col4.

### Rendering / coordinate facts

- `DebugRect { min, max, color:[f32;4], z }` pushed into `DebugDrawQueue.items` → filled `DrawRect`. Drained each frame.
- z-order ascending = painter's order (floor 0.0, wall 0.1, goal 0.2, box 0.3, player 0.4).
- Window: `GRID_COLS=9 × GRID_ROWS=6 × TILE=56` + 56px HUD strip. Levels centered within the footprint via `cell_origin`.
- Box turns green (`[0.40,0.85,0.45,1.0]`) when on a goal.

### Controls

WASD/Arrows push · `U` undo · `Y` redo · `R` restart level · `N`/Enter next (after solve, or skip within unlocked) · `P` prev · `Esc` quit.

### `History<T>` public API (full signature, `src/history.rs`)

```rust
pub struct History<T> { /* past, future, capacity */ }
impl<T> Default for History<T>            // empty, unbounded (no T: Clone bound)
impl<T: Clone> History<T> {
    pub fn new() -> Self;
    pub fn with_capacity(capacity: usize) -> Self; // bounded undo depth; 0 = never retains
    pub fn record(&mut self, snapshot: T);          // call BEFORE mutating; clears redo branch
    pub fn undo(&mut self, current: &mut T) -> bool; // restore prev into current; pushes current to redo
    pub fn redo(&mut self, current: &mut T) -> bool;
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
    pub fn undo_depth(&self) -> usize;
    pub fn clear(&mut self);
}
```

### Literal `LEVELS` constant (for regression / extension)

```text
L1: "#######" / "###.###" / "#  $  #" / "#  @  #" / "#######"
L2: "#########" / "#.$@ $ .#" / "#########"
L3: "#######" / "#.   .#" / "#     #" / "#$   $#" / "#  @  #" / "#######"
```

### `/grill-me` locked decisions (pre-coding)

- Subgenre = Sokoban (not match-3 — match-3 has awkward undo + heavier matching logic).
- Multi-level + progress save/load (vs single-level / no-save) — to actually exercise the save gap.
- Fix surfaced gaps in the engine, per maze precedent (vs example-only / record-only).
- Proof bar = native+wasm build + clippy + existing tests + manual play (vs build+test only, or +new unit tests).
- Branch base = `docs/english-conversion` (surfaced mid-session; main was 8 behind).
- PR base = `main` (surfaced mid-session; user accepted the 10-commit bundle).

**Deferred (from grill packet):** `save_api_shape` — exact save ergonomics; default was "extend `save.rs` additively as friction appears." Outcome: no extension needed.

### Verification commands run

- `cargo test --lib history` → 4 passed.
- `cargo build --example sokoban_game` → ok.
- `cargo clippy --lib --example sokoban_game` → 0 warnings (after the `&format!` fix).
- `cargo test --lib` → 245 passed, 0 failed.
- `cargo fmt --check` → clean.
- `cargo build --target wasm32-unknown-unknown --lib` and `--example sokoban_game` → both ok.
- Startup smoke: backgrounded run, alive after 6s, no panic in log.

### NOT verified

- **Interactive play** (actual key input → push/undo/win/level-advance/save round-trip). The GUI window cannot be visually observed in this environment. Logic verified by review + the `History` unit tests only. Left for the user / next session to confirm with `cargo run --example sokoban_game`.

## Code Analysis

Per-frame system order in `main()`: `InputSystem` → `RenderSystem`. `InputSystem` reads `InputState.just_pressed` (discrete one-step-per-press, no DAS), mutates the `Session` resource. `Session::try_move` snapshots `state` into `History` *before* mutating and only on a real move; checks wall, then box-push (box blocked if wall/box beyond). `check_solved` sets `Solved`/`AllClear`. Undo/redo clone `state`, call `History`, write back, adjust `moves`/`status`. `RenderSystem` rebuilds the board into `DebugDrawQueue` + HUD into `TextQueue` from `Session` state each frame.

## Files Changed

### Source code (engine)
- `src/history.rs` — NEW. `History<T>` + tests.
- `src/lib.rs` — `pub mod history;`, `pub use history::History;`.

### Example (new)
- `examples/games/sokoban/sokoban.rs`.

### Config
- `Cargo.toml` — `[[example]] sokoban_game`.

### Documentation
- `docs/NEXT_WORK.md` — C done, recommended order now D.
- `docs/HANDOFF.md` — candidate table row + status line.
- `CLAUDE.md` — module-map row for `src/history.rs`.

### Pre-existing rustfmt (separate commit `eb89728`)
- `examples/games/maze_escape/maze_escape.rs`, `src/pathfinding.rs` — rustfmt-only, no logic change.

### Compatibility check — `rust-survivors` impact
- `History` is purely additive; no existing public API changed. No `rust-survivors` impact. (rust-survivors is the sibling game repo at `/Users/jkl/Projects/rust-survivors`.)

## User Feedback & Preferences

- Ran `/grill-me` to lock scope before coding — user prefers requirements pinned down via bounded questions first.
- Korean is the user's working language; doc prose stays English per `docs/doc-language-rule` (token cost).
- User chose the broad PR base (`main`) knowingly after the bundle implication was spelled out — comfortable with a large mixed PR here.
- Commit style: `Co-Authored-By: Claude Opus 4.8`. PR body ends with the Claude Code generated-with line.

## Candidate D — engine API inventory (pre-gathered for the plan)

Gathered this session so the next session does not re-derive. All confirmed present and re-exported from `engine::`:

- **`ParticleEmitter`** (`src/particle.rs:27`) — fields: `spawn_rate` (per sec), `lifetime`, `velocity`, `velocity_spread`, `color_start`/`color_end` (lerped over life), `size`, `texture: Option<String>` (None = solid quad), `emit: bool`. `ParticleSystem` updates + despawns expired particles. **GAP CANDIDATE: continuous-only — no one-shot burst.** A hit explosion wants "emit N now," not a stream; expect to add a burst helper (e.g. `ParticleEmitter::burst(n)` or a `ParticleBurst` component / `emit_count`). This is the most likely engine gap for D.
- **`Pool` / `Pooled`** (`src/pool.rs`) — `Pool::new(capacity)`, `acquire(&mut World, setup: impl FnOnce(&mut World, Entity)) -> Entity`, `release(entity, &mut World)`, `available_count`, `capacity`, `clear`. Use for bullet pooling to avoid spawn-burst churn (`NEXT_WORK` flagged pooling friction).
- **`Timer`** (`src/timer.rs`) — `once(d)`, `repeating(d)`, `tick(dt)`, `finished`, `just_finished`, `elapsed`, `duration`, `fraction`, `reset`. Use `repeating` for wave/spawn cadence and fire cooldown.
- **`CollisionLayer` / `Collider` / `SpatialGrid` / `CollisionGridSystem`** (`src/collision/`) — same pattern maze used: `CollisionGridSystem::new(cell)` rebuilds `SpatialGrid` into a `World` resource each frame; query via `world.resource::<SpatialGrid>()` + `query_aabb(min, max, mask)`. Use layers for player/enemy/bullet separation.
- **`AudioManager`** (`src/audio.rs`) — `play(channel, path, repeat)`, `play_fade_in`, `play_at` (positional), `set_volume(channel, v)`, `set_bus_volume(bus, v)`, `bus_volume`, `play_tone(channel, freq, dur, vol)`. Channel→bus relationship to confirm when wiring sfx/music buses. `play_tone` is handy for placeholder sfx with no asset files.

## Where We're Going

Next recommended item is **candidate D — simple shooter** (`docs/NEXT_WORK.md`): bullets + enemy waves. Validates `ParticleEmitter`, `Timer`, collision layers, and audio buses; likely surfaces pooling / spawn-burst / perf friction; complements `rust-survivors`. The paired `PLAN_sokoban-example-game_2026-05-31.md` details the phased approach for D.

## Risks & Blockers

- **Interactive play unverified** — if a control or solve edge case is broken, only a human playtest will catch it. Mitigation: next session/user runs the example first thing.
- **PR #1 is a large bundle** — review burden; if `main` later receives the english-conversion work separately, rebase may be needed. Not blocking.
- No beads tooling — phase tracking is via TaskCreate + the plan file, not `bd`.

## Open Questions

- Should the english-conversion stack merge to `main` independently of this PR, or is PR #1 the intended single landing? (User chose bundle; flagged for awareness.)
- For candidate D: pool bullets via the existing `engine::Pool`, or spawn/despawn ECS entities directly? (Plan Phase 1 decides based on `src/pool.rs` ergonomics — read it first.)

## Quick Start for Next Session

See the paired plan `PLAN_sokoban-example-game_2026-05-31.md`. Sokoban itself is DONE and in PR #1 — the next session executes candidate **D (shooter)**, not more Sokoban work. To sanity-check Sokoban before moving on: `cargo run --example sokoban_game`.

## Session Closed

Sokoban (candidate C) shipped: example + `History<T>` engine utility, all checks green except human playtest. Committed `67fcb9c`, pushed, PR #1 against main. Next: candidate D shooter, see plan.
