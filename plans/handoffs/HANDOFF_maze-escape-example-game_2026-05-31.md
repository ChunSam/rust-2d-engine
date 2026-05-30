# Maze-escape playable example + BT/SpatialGrid/PathGrid API gap closure

**Date:** 2026-05-31
**Status:** COMPLETED
**Bead(s):** none
**Epic:** Candidate B from `docs/NEXT_WORK.md` (playable examples for v1.0.0 dogfooding)
**Chain:** `maze-escape-example-game` seq `1`
**Parent:** `none — first in chain`
**Prior chain:** none — first in chain

---

## Related Handoffs

Sibling work streams in the same playable-examples program — independent, not chain parents:

- `HANDOFF_platformer-example-game_2026-05-30.md` — candidate A (platformer), separate work stream. Same project pattern (per-candidate handoff).
- `HANDOFF_scene-flow-ui-interaction_2026-05-31.md` — candidate E (scene-flow), authored by codex in parallel with this session. Codex committed `74732ed feat: add scene flow playable example` while this session was running.

## Reference Documents

- `CLAUDE.md` — engine module map and task recipes (200-line agent reference)
- `docs/VISION.md` — feature+example loop ("a new feature is not done until a small example game exercises it in real play")
- `docs/NEXT_WORK.md` — candidate table, recommended order, alignment check
- `docs/PATTERNS.md` — extracted architecture patterns (borrow workaround, render-layer separation, system order)
- `docs/HANDOFF.md` — per-phase dev history

## The Goal

Pick the next candidate from `docs/NEXT_WORK.md` and ship it under the engine's dogfooding loop: build a small playable example that **uses three existing v1.0.0 subsystems that have never been wired together** (`PathGrid` / `BehaviorTree` / `SpatialGrid`), surface the friction this exposes, and close that friction by patching the engine before release. End state: `cargo run --example maze_escape_game` is an actual playable mini-game, and a fork user can use `BehaviorTree` + `find_path` + `query_aabb` without reaching into engine internals.

## Where We Are

- Committed `dfe1946 feat: add maze-escape playable example + BT/SpatialGrid/PathGrid API gaps` on branch `docs/english-conversion`. 7 files changed, +773/-9.
- Working tree clean. codex's parallel scene-flow work landed as `74732ed` mid-session, so no shared-file conflict at commit time.
- New playable example: `examples/games/maze_escape/maze_escape.rs` — 667 lines, 17×11 ASCII tile maze, single enemy (originally planned 2; user accepted 1 after playtest).
- Engine gap 1 closed: `src/lib.rs:61` now re-exports `BehaviorTree`, `BehaviorNode`, `BehaviorStatus`, `Sequence`, `Selector`, `Inverter`, `AlwaysSucceed`, `BehaviorSystem`. Previously only `Blackboard`/`BlackboardValue` were public.
- Engine gap 2 closed: `src/collision/grid.rs` — `SpatialGrid` derives `Clone`; `CollisionGridSystem::run` now calls `world.insert_resource(self.grid.clone())` after rebuild. External systems read via `world.resource::<SpatialGrid>()`.
- Engine gap 3 closed: `src/pathfinding.rs` — `PathGrid::from_tilemap(&Tilemap, impl Fn(u32) -> bool) -> PathGrid`. Width = max-row-length, jagged rows treat missing cells as empty.
- `Cargo.toml:43-46` — registered `[[example]] name = "maze_escape_game"` pointing at the new file.
- `docs/NEXT_WORK.md` — row B marked ✅ done with surfaced-gaps text; intro lists new example command; recommended-order block reduced to "C / D to widen genre coverage. (A, B, E, F now done.)"
- `CLAUDE.md` module map gained two rows (`BehaviorTree…` → `src/behavior.rs`, `PathGrid…` → `src/pathfinding.rs`) and SpatialGrid row was annotated with the mirroring fact.
- Tests: `cargo test --lib` → **238 passed**, 0 failed. Two new unit tests under `pathfinding::tests`: `from_tilemap_marks_blocked_tiles_unwalkable`, `from_tilemap_with_jagged_rows_uses_max_width`.
- Lints: `cargo clippy --all-targets --no-deps` → clean (zero warnings).
- WASM: `cargo build --target wasm32-unknown-unknown --lib` → passes.
- User playtest: all 13 manual checklist items confirmed by user ("전체 테스트이상없음").

## What We Tried (Chronological)

1. **Doc/git inspection to figure out current state.** Read `docs/NEXT_WORK.md` and `git log --oneline -20`. Found A and F done (committed), E in-progress in working tree (codex), and B/C/D candidates remaining. Working tree had codex's `src/app.rs` HiDPI fix, `src/ui/system.rs` same-frame click fix, and `examples/games/scene_flow/*` (later committed by codex as `74732ed`).
2. **Picked candidate B.** Reasoning: NEXT_WORK's own recommended order put B/C/D first; B is the only candidate that ties together three already-built subsystems that lacked an integration example. C requires save/load polish, D overlaps with rust-survivors.
3. **Explore-agent recon (single Explore agent, ~400-word report).** Mapped existing surface: `PathGrid` + `find_path` at `src/pathfinding.rs:6,110`; `BehaviorTree`/`Sequence`/`Selector`/`Inverter`/`AlwaysSucceed`/`Blackboard`/`BehaviorSystem` at `src/behavior.rs`; `SpatialGrid`/`Collider`/`CollisionLayer`/`CollisionGridSystem` at `src/collision/grid.rs:66,176`; `query_radius`/`query_aabb` at `src/collision/query.rs:12,43`. Identified three gaps that an enemy-AI example would hit immediately: BT not re-exported, SpatialGrid trapped inside its system, no `Tilemap → PathGrid` bridge.
4. **Plan-mode iteration.** Wrote initial plan to `/Users/jkl/.claude/plans/tranquil-zooming-shamir.md`. User asked to "ambiguity → 10%" so issued one AskUserQuestion batch (4 questions: movement model, SpatialGrid exposure pattern, LoS algorithm, camera/restart flow). User picked the recommended option on all four (free-roam AABB / World-resource mirror / Bresenham over PathGrid / fixed camera + R reset). ExitPlanMode → approved.
5. **Engine gap 1a — `lib.rs` re-export.** Replaced single-line `pub use behavior::{Blackboard, BlackboardValue};` with the 9-name re-export block. Trivial.
6. **Engine gap 1c — `PathGrid::from_tilemap`.** Added `use crate::tilemap::Tilemap;` import; appended an `impl PathGrid` method that walks `tiles[row][col]`, calls `is_blocked(tile_id)`, and flips that cell unwalkable. Width derived as `tiles.iter().map(|r| r.len()).max().unwrap_or(0)`. Two unit tests added under `pathfinding::tests`.
7. **Doc-language slip caught mid-stream.** Initial `from_tilemap` doc comment + test comments were Korean. CLAUDE.md rule says new doc prose is English (`docs/PATTERNS.md` codified it). Ran `grep -n "바닥\|벽\|예시"` against `src/pathfinding.rs`, found 1 surviving Korean line inside the doctest, fixed with one more Edit.
8. **Engine gap 1b — SpatialGrid resource exposure.** Considered two designs: (a) move grid ownership entirely out of `CollisionGridSystem` into a resource, (b) keep the system owning its grid but clone-mirror into a resource each frame. Picked (b) to avoid any backward-compatibility risk for `rust-survivors` (verified via grep — survivors uses `SpatialGrid::new(128.0)` directly inside its own systems, not the engine's `CollisionGridSystem`, so neither design would break it; clone-mirror is still simpler and reuses the `PhysicsWorld` selfowned-+-exposed precedent at `src/physics/system.rs`). Derived `Clone` on `SpatialGrid`, added two lines in `CollisionGridSystem::run`.
9. **Initial lib build.** Ran `cargo build --lib`. Output "Finished … 0.21s" — suspected stale cache. `touch src/lib.rs && cargo build --lib` still no recompile message. Did `cargo clean -p skeleton-engine && cargo check --lib` → actually recompiled (1.27s). No errors. Treated as cache anomaly, not a real concern.
10. **Wrote `examples/games/maze_escape/maze_escape.rs`.** Single 667-line file. No external assets — used `Sprite::colored(r,g,b)` everywhere (skipped the platformer's PNG atlas pattern) so the example is self-contained.
11. **Imported `behavior::{Blackboard, BlackboardValue}` directly** despite re-export — the engine `pub use` only added top-level `BehaviorTree`/`BehaviorSystem` etc.; `Blackboard` was already a top-level re-export and `BlackboardValue` is used in pattern matching. Kept the `behavior::` path to make the dependency explicit in code.
12. **Marker components left as `#[allow(dead_code)]`.** `Player`/`Enemy`/`Goal`/`Wall` unit structs are attached as components but never queried (routing is through `MazeSession`). Considered removing them; kept them with the allow attr so a fork reader can grep for "Enemy" and find what's tagged.
13. **Cargo example wiring.** Inserted `[[example]] name = "maze_escape_game"` after codex's `scene_flow_game` entry. First compile of `--example maze_escape_game` → success (4.65s, no warnings).
14. **Docs.** Updated `docs/NEXT_WORK.md` (row B ✅ done + 3-sentence surfaced-gaps text + recommended-order pruned), added two rows + one annotation to `CLAUDE.md` module map.
15. **Verification batch.** Ran in parallel: `cargo test --lib` (238/238), `cargo clippy --all-targets --no-deps` (clean), `cargo build --target wasm32-unknown-unknown --lib` (pass). Then `cargo run --example maze_escape_game &` with 4s survival check → process stayed alive past startup. Could not visually verify gameplay from CLI — said so explicitly.
16. **First user playtest.** User reported items 1–5 (basic + player movement + wall slide) and 8, 9, 11–13 pass. Items 6, 7, 10 (enemy AI + lose trigger) couldn't be tested because the enemy was sitting in an isolated 1-cell pocket.
17. **Diagnosed the maze bug by hand.** `E` is at `(8,7)` in the ASCII layout. Traced its 4 neighbors: row 6 col 8 = `#`, row 8 col 8 = `#`, row 7 col 7 = `#`, row 7 col 9 = `#`. All four walls — enemy fully boxed. BT's LoS leaf and the path-follow leaf both compute and return Success, but the actual `move_enemy_toward` produces 0 movement against walls.
18. **Maze fix — single-cell patch.** Changed `MAZE` row 7 from `"#......#E#......#"` to `"#......#E.......#"` (opened col 9). Traced connectivity by hand: `E → (9,7) → (10..15,7) → (15,8) → (15,9) → row 9 corridor → (1,9) → up to (1..6,1) → P(1,1)`. Decided NOT to add a second enemy despite the plan saying "2마리"; user's immediate ask was "make it testable", and one enemy already validates the BT path. Recorded as out-of-scope in the bugfix plan.
19. **Second playtest.** User reported "전체 테스트이상없음" — all 13 items pass.
20. **Commit split.** Checked working tree: codex's scene_flow work had already been committed as `74732ed`, so all current modifications were mine. Staged 7 files (CLAUDE.md, Cargo.toml, docs/NEXT_WORK.md, src/collision/grid.rs, src/lib.rs, src/pathfinding.rs, examples/games/maze_escape/). One commit `dfe1946`. Tree clean.

## Key Decisions

- **Build B before C or D** — even though all three were "candidate" status, only B used three engine subsystems that lacked a single integration example. Rejected: starting on C (puzzle/save-load) which would have surfaced different gaps but doesn't make the BT/PathGrid/SpatialGrid debt visible.
- **Single-cycle: close gaps + ship example together** — instead of "fix gaps first, then example next session". Reasoning: the vision rule says "if the API feels awkward while writing the example, fix the API before release". Splitting would have meant shipping a v1.0.x release with three open gaps.
- **Clone-mirror SpatialGrid, not move-into-resource** — Option (a) (move ownership into a `World` resource) would change `CollisionGridSystem`'s signature and risk silent breakage for any downstream that touched `system.grid`. Option (b) clones each frame (O(entities)). For the demo entity counts (~200) the clone is cheap; for survivors (their own grid, not ours) zero impact. Cheaper to defer the ownership move to a future major.
- **`PathGrid::from_tilemap` lives on `PathGrid`, not `Tilemap`** — the helper depends on `Tilemap` but conceptually constructs a `PathGrid`, so the `impl PathGrid` block is its natural home. Required adding `use crate::tilemap::Tilemap;` in `src/pathfinding.rs`. Alternative (a free function in `tilemap.rs`) was rejected — would scatter `PathGrid` construction across modules.
- **Jagged rows accepted via "max width + missing = 0"** — rather than panic, the helper extends shorter rows with empty cells. Costs nothing, matches what a user from an editor-exported map would expect.
- **No physics dependency in maze_escape** — platformer uses rapier2d (`PhysicsWorld`/`PhysicsBody`/`move_character`). Maze uses raw AABB rejection per axis. Reasoning: keeps the example focused on BT/PathGrid/SpatialGrid and avoids tying maze movement to the rapier surface (which is also wasm-gated via `#[cfg(not(target_arch = "wasm32"))]`).
- **Wall collision via SpatialGrid `query_aabb`, not manual tile lookup** — manually checking the wall PathGrid would be faster, but using `query_aabb` *is the validation* for engine gap 1b. The same code that an external AI system would write.
- **One enemy, not two** — plan said "2마리". User's playtest priority was "let me test the AI", and the bug-fixed layout already validates the BT branches. Recorded as deliberate scope cut.
- **No PNG assets** — platformer ships `assets/{tiles,player_atlas,goal}.png`. Maze uses solid `Sprite::colored(...)` rectangles instead. Validates that the engine works without textures and keeps the example diff smaller.
- **R-key reset instead of Scene swap** — `MazeSession::reset(world)` re-poses entities and resets the BT (`tree.reset()`). Sibling handoff (codex's scene-flow) was specifically exercising `SceneCmd Push/Replace/Pop`, so using scenes here would have overlapped scope and risked merge conflicts mid-session.
- **Doc prose stays English** — caught one Korean comment leak in the new `from_tilemap` doctest, fixed it. Project-wide rule from `CLAUDE.md`.

## Evidence & Data

### Git state at handoff time

```
Branch: docs/english-conversion
Working tree: clean
HEAD: dfe1946 feat: add maze-escape playable example + BT/SpatialGrid/PathGrid API gaps
HEAD~1: 74732ed feat: add scene flow playable example                       (codex)
HEAD~2: 455f9d4 feat: add platformer playable example
HEAD~3: 05b2915 Add audio channel playback state
HEAD~4: 4c55c12 docs: extract patterns to docs/PATTERNS.md, keep quick refs <=200 lines
```

### Diff stat of `dfe1946`

```
 CLAUDE.md                                 |   4 +-
 Cargo.toml                                |   4 +
 docs/NEXT_WORK.md                         |  11 +-
 examples/games/maze_escape/maze_escape.rs | 667 ++++++++++++++++++++++++++++++
 src/collision/grid.rs                     |   9 +-
 src/lib.rs                                |   5 +-
 src/pathfinding.rs                        |  82 ++++
 7 files changed, 773 insertions(+), 9 deletions(-)
```

### Engine gaps closed — surface-area before/after

| Gap | Before | After | Where |
|---|---|---|---|
| BT re-exports | `pub use behavior::{Blackboard, BlackboardValue};` | + `BehaviorTree, BehaviorNode, BehaviorStatus, Sequence, Selector, Inverter, AlwaysSucceed, BehaviorSystem` | `src/lib.rs:61` |
| SpatialGrid access | `pub grid: SpatialGrid` field on `CollisionGridSystem` only | `world.resource::<SpatialGrid>()` after `CollisionGridSystem` runs | `src/collision/grid.rs` (Clone derive + `world.insert_resource` in `run`) |
| Tilemap→PathGrid | (nothing) | `PathGrid::from_tilemap(&Tilemap, impl Fn(u32) -> bool) -> Self` | `src/pathfinding.rs` |

### Surfaced-but-deferred gaps

- `BlackboardValue` enum (`Bool / Float / Int / Vec2 / String` — see `src/behavior.rs:50-56`) cannot hold `Vec<IVec2>`. `ComputePathToPlayer` works around it by writing only the next path step (`Vec2`) and re-running `find_path` every tick. Cheap on this 17×11 grid; would need attention before a non-toy game. Recorded in `docs/NEXT_WORK.md` row B.
- No `Tilemap::with_collision(...)` helper. The example spawns floor sprites, wall sprites + `Collider::Aabb` + `CollisionLayer(WALL_LAYER)` + `Wall` marker, then builds `PathGrid::from_tilemap` separately — three loops over the same tile data. A future helper could collapse this. Same itch as platformer's noted "tilemap↔physics binding" gap.
- No `LineOfSight::bresenham(&PathGrid, IVec2, IVec2) -> bool` helper. The maze inlines Bresenham as `line_clear` at the example level. Worth promoting to engine only after a second example needs it.

### Reference data — full `MAZE` constant (post-bugfix)

```
"#################",   row 0
"#P.....#.......G#",   row 1
"#.####.#.#####..#",   row 2
"#..#...#.#...#..#",   row 3
"##.#.###.#.#.#..#",   row 4
"#..#.....#.#....#",   row 5
"#.####.###.###.##",   row 6
"#......#E.......#",   row 7  ← bugfix patched col 9 from '#' to '.'
"##.####.#.####..#",   row 8
"#...............#",   row 9
"#################",   row 10
```

17 cols × 11 rows. `'#'` wall, `'.'` floor, `'P'` player spawn (1,1), `'E'` enemy spawn (8,7), `'G'` goal (15,1). `TILE = 40.0`, window 680×440.

### Verification commands run

```
cargo test --lib                                    → 238 passed; 0 failed; 0 ignored
cargo clippy --all-targets --no-deps                → Finished … 2.26s, zero warnings
cargo build --target wasm32-unknown-unknown --lib   → Finished … 4.98s, no errors
cargo build --example maze_escape_game              → Finished … 4.65s, no warnings
cargo run --example maze_escape_game                → launches; survives 4s; user-playtested 2 rounds
```

### Pathfinding test additions (under `pathfinding::tests`)

```
test pathfinding::tests::from_tilemap_with_jagged_rows_uses_max_width ... ok
test pathfinding::tests::from_tilemap_marks_blocked_tiles_unwalkable ... ok
```

Counting all `pathfinding::tests`: 7/7 ok (5 pre-existing + 2 new).

### Playtest checklist outcome

Two playtest rounds against this 13-item checklist:

| # | Item | Round 1 | Round 2 |
|---|---|---|---|
| 1 | Window title / size / palette correct | ✅ | ✅ |
| 2 | Player visible at left, enemy in middle, goal upper-right | ✅ | ✅ |
| 3 | WASD / arrows move player | ✅ | ✅ |
| 4 | Per-axis wall slide on collision | ✅ | ✅ |
| 5 | Maze reachable end-to-end for player | ✅ | ✅ |
| 6 | LoS enemy chases in straight line | (blocked: enemy boxed) | ✅ |
| 7 | Enemy detours via `find_path` when LoS broken | (blocked: enemy boxed) | ✅ |
| 8 | Enemy slower than player so escape possible | ✅ | ✅ |
| 9 | Goal triggers "Escaped!" message | ✅ | ✅ |
| 10 | Enemy contact triggers "Caught." message | (blocked: enemy boxed) | ✅ |
| 11 | R-key resets positions and Status::Playing | ✅ | ✅ |
| 12 | Esc quits | ✅ | ✅ |
| 13 | HUD text top-left | ✅ | ✅ |

### Single-cell maze patch — connectivity trace

Before (E fully walled at `(8,7)`):
```
row 6 (col 8 = '#'):    "#.####.###.###.##"
row 7 (cols 7,9 = '#'): "#......#E#......#"
row 8 (col 8 = '#'):    "##.####.#.####..#"
```

After (only row 7 changed, opened col 9):
```
row 7: "#......#E.......#"
```

Reachability E → P verified by hand: `E(8,7) → (9,7) → (10,7) → ... → (15,7) → (15,8) → (15,9) → row 9 corridor west to (1,9) → up to (1,5) → (2,5) → ... → (6,1) → ... → (1,1) = P`. Long path but valid — the LoS shortcut kicks in once the enemy emerges into the open east corridor.

### Time/effort estimate

- Planning + AskUserQuestion + ExitPlanMode loop: ~6 tool turns
- Engine gap fixes: ~10 tool turns (lib.rs, pathfinding.rs add + tests, collision/grid.rs + comment, doc-language fix)
- Example file (677 lines including blanks): single Write call after reading ~5 reference files
- Verification + first run: ~4 tool turns
- Bugfix cycle (re-enter plan mode for the maze layout patch, edit, rebuild, re-run): ~4 tool turns
- Commit split: 3 tool turns (status, stage, commit)

### Per-frame system order (registered in `main()`)

```
1. CollisionGridSystem::new(TILE * 2.0 = 80.0)   // rebuilds SpatialGrid + inserts as resource
2. PlayerInputSystem                              // reads InputState, queries SpatialGrid for wall slide
3. BehaviorSystem                                 // ticks each enemy's BehaviorTree
4. WinLoseSystem                                  // distance checks player↔goal / player↔enemy
5. HudSystem                                      // pushes DrawText into TextQueue
```

`CollisionGridSystem` must be first; otherwise tick 0 has no `SpatialGrid` resource and `resolve_walls` falls through to "no collision". This is documented in the comment above the `app.add_system(...)` block in `main()`. Same constraint as platformer's `PhysicsSystem` ordering.

### Code Analysis (data flow per BT tick)

For one enemy entity on one frame:

1. `BehaviorSystem::run` collects all entities with `BehaviorTree` component (just 1 here).
2. `world.take_component::<BehaviorTree>(entity)` removes the tree from the world (borrow-checker dodge).
3. `tree.tick(&mut world, entity, dt)` → root `Selector` ticks its first child (the LoS sequence).
4. `Sequence(HasLineOfSight, MoveTowardPlayer)`: 
   - `HasLineOfSight::tick` reads `world.resource::<MazeSession>().player`, then `world.get::<Transform>(player)`, then `world.resource::<PathGrid>()`, runs `line_clear` (Bresenham). Returns Success or Failure.
   - On Success: `MoveTowardPlayer::tick` reads player Transform again, calls `move_enemy_toward` which reads `world.resource::<SpatialGrid>()` for wall-slide rejection, writes back `Transform.position`. Returns Success (single-tick).
   - Sequence overall returns Success → Selector short-circuits, root returns Success.
5. On Failure (no LoS): Selector advances to second child.
6. `Sequence(ComputePathToPlayer, FollowPathStep)`:
   - `ComputePathToPlayer::tick` calls `engine::find_path(grid, enemy_tile, player_tile)`. On `Some(path)`: writes the first step's world center to `Blackboard["path_target"]` as `BlackboardValue::Vec2`. Returns Success. On `None`: Failure.
   - `FollowPathStep::tick` reads `Blackboard["path_target"]` (via pattern-match on `entries()`), calls `move_enemy_toward(world, entity, target, dt)`. Returns Success.
7. `world.add_component(entity, tree)` returns the tree to the world.

This pattern means `path_target` is overwritten every tick — the cached step never goes stale. Costs one A* per frame per enemy.

## Code Analysis

- **`SpatialGrid::clone()` cost.** `SpatialGrid` contains two HashMaps keyed by `(i32,i32)` and `Entity`. Each `rebuild` clears them; `clone` deep-copies. For ~200 wall entities + player + enemy in the maze, the clone is sub-millisecond. For survivor-scale (hundreds of bullets and enemies) the cost grows linearly — fine, but a forked game that hits 10k+ entities should consider moving ownership into the resource (the deferred Option-a design).
- **`PathGrid::from_tilemap` width rule.** `tilemap.tiles.iter().map(|row| row.len()).max().unwrap_or(0) as i32`. Cast to `i32` is fine; `PathGrid::new` already guards against overflow via `MAX_PATH_GRID_CELLS = 10_000_000`. For jagged rows, the missing-cells-as-empty rule means a user can't accidentally produce an unwalkable cell by trimming a row — only `is_blocked(tile_id)` returning true creates walls.
- **`BehaviorSystem` borrow workaround.** `src/behavior.rs:354-364` collects entities into a `Vec<Entity>`, then for each: `take_component::<BehaviorTree>` → `tree.tick(&mut world, ...)` → `add_component(entity, tree)`. The tree is *out of the world* while ticking, so leaves can freely `world.get/get_mut` anything including the entity itself. Maze's leaves rely on this for `world.resource::<MazeSession>()` and `world.get::<Transform>(player_entity)`.
- **`BlackboardValue` shape.** `Bool(bool) / Float(f32) / Int(i32) / Vec2(Vec2) / String(String)`. No vec/list variant. Forced the maze's `ComputePathToPlayer` to store only the next step (`Vec2`) and recompute the A* each tick. Cheap here, but a serious AI library would want at minimum a `Vec2List` variant or a generic `Box<dyn Any>` escape hatch.
- **Camera coordinate convention.** `Camera::position` is the **top-left** of the visible viewport in world coords (`src/camera.rs:7-15`). The maze uses `Camera::new(Vec2::ZERO, 1.0)` — origin top-left, no zoom — which lines up exactly with the tilemap origin and means tile centers `(col*40+20, row*40+20)` are directly screen-relative.
- **`world.resource::<SpatialGrid>()` requires the system order** input→`CollisionGridSystem`→AI→win/lose→HUD. If `CollisionGridSystem` ran *after* the AI/player systems on the first frame, the resource would be missing on tick 0. The `query_aabb` calls handle this via `let Some(grid) = world.resource::<SpatialGrid>() else { return proposed; };` — degrades gracefully to "no wall collision" for that one frame instead of panicking.
- **Sprite Z-order.** Floor `z = -1.0`, wall `z = 0.0`, goal `z = 0.5`, player/enemy `z = 1.0`. The renderer draws low-z first per `src/components.rs:16`.
- **`move_enemy_toward` early-return at `dist < 1.0`.** Prevents jitter once the enemy is sub-pixel away from a tile center — without it the normalize/scale would oscillate the position by floats every frame. Same guard appears in the path-target check.
- **`resolve_walls` is per-axis.** Tries X-only candidate first, accepts if no overlap, then Y-only candidate from the accepted X. Standard separating-axis wall slide. Order matters: trying both axes at once would refuse motion that should slide.
- **`Wall` marker component is dead-code by query.** `Player`/`Enemy`/`Goal`/`Wall` markers are all attached but the game routes entirely through `MazeSession.player`/`.goal`/`.enemies` (Entity handles). Markers are `#[allow(dead_code)]` so a fork reader can grep "Wall" to see what's tagged without the compiler nagging.
- **`ParsedMaze` shape.** `tiles: Vec<Vec<u32>>` (row-major like `Tilemap.tiles`), `player: Vec2`, `goal: Vec2`, `enemies: Vec<Vec2>`. Returned from `parse_maze()` which walks `MAZE: &[&str]` once and converts `'#'/'.'/'P'/'E'/'G'` into tile ids 2/1/1/1/1 plus the spawn coords.
- **`tile_center(col, row)`** = `Vec2::new(col*TILE + TILE*0.5, row*TILE + TILE*0.5)`. Tile-space ↔ world-space single source of truth. `world_to_tile(pos)` is the inverse (truncating).
- **HUD positioning gotcha.** `DrawText` for the always-on legend is at `Vec2::new(8.0, 8.0)`, the won/lost messages at `Vec2::new(WINDOW_W/2 - 170, WINDOW_H/2 - 20)`. These are world coords; for the fixed camera at `(0,0)` with zoom 1, world == screen so they land correctly. A camera-follow variant would need a separate "screen-space" text path — engine has `TextQueue` for screen-aligned text per `src/renderer/mod.rs`, but the maze uses the default world-space convention.

## Files Changed

### Source code (engine)

- `src/lib.rs` — 8 additional names in the `pub use behavior::{...}` block (line 61).
- `src/pathfinding.rs` — `use crate::tilemap::Tilemap;` import; new `impl PathGrid { pub fn from_tilemap(...) }` method (~40 lines including English doctest); 2 new unit tests.
- `src/collision/grid.rs` — `#[derive(Clone)]` added to `SpatialGrid` struct; doc comment mentions resource mirroring; `impl System for CollisionGridSystem::run` now calls `world.insert_resource(self.grid.clone());` after rebuild.

### Example (new)

- `examples/games/maze_escape/maze_escape.rs` (667 lines) — single file. Sections: constants, marker components, `MazeSession` resource + `Status` enum, ASCII maze parser, BT leaf nodes (`HasLineOfSight`, `MoveTowardPlayer`, `ComputePathToPlayer`, `FollowPathStep`), `move_enemy_toward`/`line_clear` helpers, `resolve_walls`/`overlaps_wall` (SpatialGrid query), `PlayerInputSystem`/`WinLoseSystem`/`HudSystem`, `reset(world)`, BT builder, `main()` (spawn floors/walls/goal/player/enemies, insert resources, register systems, run).

### Config

- `Cargo.toml` — `[[example]] name = "maze_escape_game" path = "examples/games/maze_escape/maze_escape.rs"` appended after the scene_flow_game entry (lines 43-46).

### Documentation

- `docs/NEXT_WORK.md` — intro paragraph mentions new example command; row B marked ✅ done with three-line surfaced-gaps text; recommended-order list pruned to "C / D to widen genre coverage. (A, B, E, F now done.)".
- `CLAUDE.md` — module map gained two rows (`BehaviorTree / Sequence / Selector / Inverter / AlwaysSucceed / BehaviorSystem / Blackboard → src/behavior.rs` and `PathGrid / find_path / PathGrid::from_tilemap → src/pathfinding.rs`); SpatialGrid row annotated with "(SpatialGrid is mirrored to a World resource by CollisionGridSystem)".

### Plan/handoff scratch

- `/Users/jkl/.claude/plans/tranquil-zooming-shamir.md` — written/rewritten 3 times across this session (initial plan, scope-narrowed plan, post-playtest bugfix plan). Not committed (lives outside repo).

### Sibling work that touched the same tree (NOT in `dfe1946`)

- codex committed `74732ed feat: add scene flow playable example` while this session was running. That commit also touched `src/app.rs` (HiDPI cursor fix + UI render queue reorder), `src/ui/system.rs` (same-frame button click fix + regression test), and added `examples/games/scene_flow/`. These were the "M" entries in `git status` at session start; by commit time they were already in HEAD via codex.

### Sibling timeline (this session's perspective)

| Event | Effect on our work |
|---|---|
| Session start | Working tree had codex's uncommitted `src/app.rs`, `src/ui/system.rs`, `Cargo.toml` (scene_flow entry), `docs/NEXT_WORK.md`, and `examples/games/scene_flow/`. We deliberately avoided staging any of these. |
| Mid-session | codex committed everything to HEAD as `74732ed`. We didn't notice the transition until staging — `git diff HEAD -- Cargo.toml` showed only our maze_escape entry (scene_flow_game already in HEAD context), which is when we realized the parallel branch had landed. |
| Our commit | `dfe1946` directly atop `74732ed`. No merge, no rebase, no conflict. |

The dual-agent boundary held because (a) we picked a candidate (B) that doesn't share files with E, (b) our `Cargo.toml` edit and `docs/NEXT_WORK.md` edit were additive in separate regions, (c) we never touched `src/app.rs`, `src/ui/system.rs`, or anything under `examples/games/scene_flow/`.

### Compatibility check — `rust-survivors` impact of SpatialGrid changes

`grep -rn "SpatialGrid\|CollisionGridSystem" /Users/jkl/Projects/rust-survivors --include="*.rs"` returned ~28 hits across `crates/game/src/survivor/{combat,bible,death,area,lightning,projectile,weapon}.rs`. Every usage is `SpatialGrid::new(128.0)` *inside the survivors' own systems* (combat, bible, weapon variants). None of them touch `CollisionGridSystem` or the new world resource. Conclusion: deriving `Clone` on `SpatialGrid` is forward-compatible; the resource-insert is purely additive. No survivors changes required.

## User Feedback & Preferences

- **"문서 확인 해서 진행 된 사항 파악해줘"** — initial direction was "audit docs and tell me where we are", not "pick a task". Drove the explore-NEXT_WORK-first opening.
- **"e는 codex로 작업중이야. 남은 후보에서 선택해서 작업 계획 세워줘"** — explicit: don't touch candidate E (scene-flow); pick from B/C/D and produce a plan. Established the parallel-work boundary.
- **"계획 모호성 검토하고 모호성 10%까지 계속 질문하여 수정"** — rejected the first ExitPlanMode and demanded ambiguity reduction via questioning before approving. Drove the 4-question AskUserQuestion batch (movement model / SpatialGrid exposure / LoS algo / camera+restart). User picked the recommended option on all four.
- **"잠깐만 게임 만들고 있는 경로 어디야?"** — mid-implementation interruption asking where the game file lives. Wanted reassurance about scope/location before continuing. Answered `examples/games/maze_escape/`.
- **"계속 진행해"** — short authoritative "keep going" after the location check.
- **"테스트 해야 할 부분 알려주고 게임 실행 해줘. 내가 직접 테스트 하고 알려줄게"** — opted into manual playtest rather than letting the agent claim success without visual verification. Set the pattern of 13-item checklist + background launch.
- **"기본동장. 플레이어 이동 이상 없음. 적 ai 적 스폰 위치가 갇힌공간이라 테스트 불가. 10번 적 검증 불가로 테스트 불가"** — terse bug report citing checklist numbers. Diagnosis-friendly format (item N + 1-line cause). Triggered the maze-layout bugfix loop.
- **"전체 테스트이상없음"** — full-pass signal after the patch. Short, definitive.
- **"커밋 분리"** — when offered the choice between "commit only mine" vs "leave it for you", picked separation. Generalized preference: keep author boundaries clean when working alongside other agents/devs.
- **Implicit cadence preference** — favored short status pings between tool turns rather than long verbose updates, and accepted recommended options without re-debating. Matches the CLAUDE.md "Tone and style" rules in the system prompt.
- **Korean ↔ English mixing tolerated.** User wrote in Korean, accepted English code/doc prose (per `CLAUDE.md` rule). Did not push back when the agent replied in Korean prose with English code comments. Doc-language rule survives — we caught one Korean leak in `from_tilemap`'s doctest mid-session and fixed it.
- **No commit unless asked.** User had to explicitly say "커밋 분리" ("split the commit") before we committed. Confirms the system-prompt default: don't commit unless requested.

### Cache anomaly noted but dismissed

After the `src/lib.rs` re-export edit, `cargo build --lib` reported "Finished … 0.21s" with no recompile message. Tried `touch src/lib.rs && cargo build --lib` — still cached. `cargo clean -p skeleton-engine && cargo check --lib` produced an actual `Checking skeleton-engine v1.0.0 … Finished … 1.27s`. No errors. Treated as cargo's `mtime`/fingerprint cache being too lazy across multiple parallel `Bash` calls; downstream verification (`cargo test`, `cargo clippy`, `cargo build --example`) confirmed the code change was real. Not investigated further.

### CLAUDE.md 200-line cap

CLAUDE.md line count: 148 → 150 (+2 rows added for `BehaviorTree…` and `PathGrid…`). Within the rule's 200-line cap. SpatialGrid row was edited in place, not added. No `docs/*.md` extraction needed this cycle.

## Where We're Going

- **Next candidate from `docs/NEXT_WORK.md`: C (Puzzle — match-grid or Sokoban) or D (Simple shooter — bullets/waves).** C exercises `Tween`/`Easing` + `save`/`load` + UI; D exercises `ParticleEmitter` + `Timer` + collision layers + audio buses. Either is fine; user has not chosen yet.
- **Surfaced-but-deferred gap to remember:** `BlackboardValue` needs a list/vec variant (or `Any` escape hatch) before a real game can lean on BT path-following. Add to a future engine maintenance commit, not its own feature work.
- **Optional polish for maze_escape that was deliberately deferred:**
  - Second enemy (the original plan said "2마리"; we shipped 1)
  - PNG assets to match the platformer's look
  - HUD positioned in screen-space rather than world-space (text currently lives at `Vec2::new(8.0, 8.0)` which is world coords; for fixed camera at origin they coincide, so this works but is fragile)
- **Documentation: README/REFERENCE.html refresh** — `REFERENCE.html` predates this commit. If next cycle adds another playable example, batch all three (B/C/D) into one regen rather than per-commit.

## Risks & Blockers

- **`SpatialGrid::clone()` per frame cost.** Not a problem at the maze's entity count. If a fork builds a 10k-entity shooter on this resource, profile before assuming it scales.
- **`Blackboard` list-variant gap** as above — workaround works but is non-obvious and re-runs A* every tick.
- **No CI surface for `cargo test --example`** — we verified the example *builds* (`cargo build --example`) and *runs* (`cargo run --example` 4s survival), but never had a headless render path. Future regressions in `CollisionGridSystem` ordering or `SpatialGrid` resource availability won't be caught until someone runs the example manually.

## Open Questions

- Does the user want C or D next? (No answer this session.)
- Should the engine gain a higher-level `Tilemap::with_collision(WallPredicate)` helper that spawns both the visual tilemap *and* the AABB collider entities + builds the PathGrid in one call? Maze had to do all three explicitly. NEXT_WORK row A already noted "tilemap↔physics binding still wants a higher-level ergonomic helper" — same itch.
- Should this branch (`docs/english-conversion`) be merged to `main` after C/D, or before? Three feature commits (`455f9d4`, `74732ed`, `dfe1946`) plus E's UI/input fixes accumulate here.

## Quick Start for Next Session

```bash
# Branch + working state
cd /Users/jkl/Projects/skeleton-engine
git status            # expect clean
git log --oneline -5  # latest should be dfe1946 maze-escape commit

# Reference docs
cat docs/NEXT_WORK.md       # candidate table — C and D are what's left
cat docs/VISION.md          # the feature+example loop
cat CLAUDE.md               # 200-line agent reference (module map + recipes)
cat docs/PATTERNS.md        # architecture patterns extracted from CLAUDE.md

# Sibling handoffs for the playable-examples program
cat plans/handoffs/HANDOFF_platformer-example-game_2026-05-30.md
cat plans/handoffs/HANDOFF_scene-flow-ui-interaction_2026-05-31.md
cat plans/handoffs/HANDOFF_maze-escape-example-game_2026-05-31.md  # this file

# Files most likely to read first for C (puzzle) or D (shooter)
src/tween.rs            # Tween + Easing — needed by C
src/timer.rs            # Timer — needed by D
src/save.rs             # save / load / load_or_default — needed by C
src/particle.rs         # ParticleEmitter / ParticleSystem — needed by D
src/ui/*                # UiNode, Button, Label — needed by both
examples/games/platformer/platformer.rs   # reference structure for new example
examples/games/maze_escape/maze_escape.rs # newest reference structure

# Verify the current state still works
cargo test --lib                                  # expect 238/238
cargo clippy --all-targets --no-deps              # expect zero warnings
cargo build --target wasm32-unknown-unknown --lib # expect pass
cargo run --example maze_escape_game              # should launch the maze game

# Next action
# Ask the user "C (puzzle) or D (shooter)?" then build under the same loop:
#   1. Explore agent → map the existing surface, identify API gaps
#   2. AskUserQuestion to lock scope (movement, win condition, asset style, etc.)
#   3. Close engine gaps + ship the example in one cycle (per VISION rule)
#   4. Run verification quartet (test / clippy / wasm / `cargo run --example`)
#   5. Manual playtest checklist with user
#   6. Single feat: commit; update NEXT_WORK.md + CLAUDE.md
```

---

## Session Closed

**Closed at:** 2026-05-31
**Commit:** `27f05a6` (handoff file) — feature work committed earlier as `dfe1946`
**Session status:** Handed off to next session
