# Vision — skeleton-engine

> Status: living document. This is the "why" behind the engine. Feature work,
> roadmap, and agent behavior should trace back to the goals stated here.
> Last reset: 2026-05-29.

## Why this engine exists

The name is the thesis: **skeleton**. This is meant to be a *bone structure* — a
clean, MIT-licensed 2D engine that other developers can fork and flesh out with
their own muscle. It deliberately favors being hackable and readable over being a
sealed black box.

Three overlapping purposes, in priority order:

1. **An open-source skeleton others can fork and extend.** People should be able to
   take the source, modify engine code directly, and grow it into their own engine.
2. **A personal foundation for my own 2D games.** It is the base layer my games are
   built on (e.g. `rust-survivors`).
3. **A personal learning vehicle.** Building the internals by hand is itself a goal —
   understanding how a game engine works from the ground up.

## What success looks like (scope)

**Genre-agnostic 2D.** Success means a developer can build a complete 2D game in any
common genre — platformer, shooter, RPG, puzzle, top-down action — without hitting a
wall that forces them off the engine. Breadth across genres matters more than depth in
any single one.

## The current problem this reset addresses

The v1.0.0 feature set exists, but it has **not been validated by enough real
gameplay**. APIs that look reasonable in isolation may be awkward in practice. The risk
is an engine that is feature-complete on paper but unpleasant to actually ship a game
with.

## Direction: features proven by playable examples

The chosen path combines two things:

- **Keep widening the feature set** (breadth-first, toward genre-agnostic 2D).
- **Validate every feature through small, playable example games.**

These are not separate tracks — they are one loop. The dogfooding vehicle is a growing
set of **small playable example games**, one per genre/feature area, living in
`examples/`. Today `examples/` holds mostly tech demos (particles, minimap, split
screen, networking); it has **no actual playable game**. Closing that gap is the point.

### Operating principles

- **A new feature is not "done" until a small example game exercises it in real play.**
  The example is the acceptance test, not an afterthought.
- **If the API feels awkward while writing the example, fix the API before release.**
  The example exists precisely to surface tabletop-theory mistakes.
- **Add features in a fork-friendly shape.** Even while prioritizing breadth, new code
  should keep clear module boundaries and extension points, so the "skeleton" stays
  forkable. Breadth first, but not at the cost of leaving an unreadable mess.
- **Post-v1.0.0 honors semver.** Breaking changes wait for v2.0.0 (see
  `docs/ENTITY_GENERATION_V2_PLAN.md`).

## Non-goals (for now)

- 3D, or competing with Unity/Unreal/Godot on feature surface.
- A visual editor as the primary authoring path (the egui debug/inspector overlay is a
  tool, not the product).
- Locking into a single genre or a single game.
