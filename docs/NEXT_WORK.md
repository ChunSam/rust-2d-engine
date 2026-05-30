# Next Work — candidates and alignment

> Status: living document. Derived from `docs/VISION.md` (reset 2026-05-29).
> This lists what to build next and why, under the vision's core loop:
> **a feature is not done until a small, playable example game in `examples/`
> exercises it in real play.**

## Context

`examples/` now separates top-level feature demos from playable example games under
`examples/games/`. The first playable examples are the platformer
(`cargo run --example platformer_game`), scene-flow game
(`cargo run --example scene_flow_game`), and maze-escape
(`cargo run --example maze_escape_game`), which start closing the previous validation gap.
The active direction remains: widen the feature set breadth-first, and prove each feature
with a small playable example.

## Candidate feature × playable-example pairs

Each candidate pairs an example game with the engine capability it validates/extends and
the API gaps it is likely to surface.

| # | Example game | Engine capability validated/extended | Likely gaps to surface |
|---|--------------|----------------------------------------|------------------------|
| **A** | Platformer (jump, run, platforms) ✅ done | `CharacterController`, `move_character`, physics platforms/sensors, `AnimationStateMachine`, atlas animation, camera follow | surfaced gaps: one-way platforms remain future work; tilemap↔physics binding still wants a higher-level ergonomic helper |
| **B** | Top-down maze escape (chasing enemies) ✅ done | `PathGrid`/`find_path`, `BehaviorTree`, `SpatialGrid` collision (`examples/games/maze_escape/maze_escape.rs`) | surfaced + fixed: `BehaviorTree`/`Sequence`/`Selector`/`Inverter`/`AlwaysSucceed`/`BehaviorSystem` were not re-exported from `engine::`; `SpatialGrid` was trapped inside `CollisionGridSystem` (now mirrored to a `World` resource each frame); no `PathGrid::from_tilemap` (added). Still open: `BlackboardValue` cannot hold `Vec<IVec2>`, so `ComputePathToPlayer` writes only the next step and recomputes each tick. |
| **C** | Puzzle (match-grid / Sokoban) | grid logic, `Tween`/`Easing`, `save`/`load`, UI | grid movement, undo, progress-save API friction |
| **D** | Simple shooter (bullets, waves) | `ParticleEmitter`, `Timer`, collision layers, audio buses | pooling/spawn bursts, perf; complements rust-survivors |
| **E** | Scene-flow game (menu → play → result) ✅ done | `SceneCmd` Push/Replace/Pop, UI buttons, `GameState`, scene-owned systems, explicit entity cleanup | surfaced gap: preserving cross-scene diagnostics/state across `Replace` requires carrying a handle outside the reset `World` |
| **F** | Skeletal-animation showcase character ✅ done | NEW: 2D cutout skeletal animation (`src/skeletal.rs`, `examples/skeletal_puppet.rs`) | surfaced + fixed `HierarchySystem` depth-3 cap; scale-vs-attachment-size rule noted in `docs/SKELETAL.md` |

## Recommended order

1. **C / D** to widen genre coverage. (A, B, E, F now done.)

## Alignment check — previously "planned" items vs the reset vision

Vision criteria: (1) fork-friendly skeleton, (2) genre-agnostic 2D, breadth-first,
(3) validate via playable examples, (4) semver after v1.0.

| Planned item | Nature | Alignment | Verdict |
|--------------|--------|-----------|---------|
| **Entity Generation v2** (`docs/ENTITY_GENERATION_V2_PLAN.md`) | correctness/safety, breaking | Fits the fork-friendly/learning goal, but it is neither breadth nor example-validated; it is a v2-only breaking change | **Cancelled (archived)** — removed from planned work; design preserved in the archived doc for a possible future v2.0.0. |
| **Dependency security follow-up** (glyphon→lru `RUSTSEC-2026-0002`, `paste` unmaintained) | maintenance/hygiene | Needed for a trustworthy forkable engine, but it is a renderer/wgpu-major migration: high-risk, non-breadth, non-example | **Cancelled (archived)** — removed from planned work; recorded as accepted/known risk in `docs/SECURITY_HARDENING_2026_05.md`. |
| **2D skeletal animation** | new feature | Directly fits genre-agnostic 2D breadth and is naturally validated by a playable example | **Done** — implemented as candidate **F** (`src/skeletal.rs`, `examples/skeletal_puppet.rs`). |

**Takeaway:** of the three pre-existing planned items, only skeletal animation matched the
current breadth-first + dogfooding priority and is now done. The other two were cancelled
from planned work and archived: Entity Generation v2's design is preserved for a possible
v2.0.0, and the dependency advisories are recorded as accepted/known risk.
