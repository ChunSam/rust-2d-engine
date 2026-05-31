# Candidate D — simple shooter playable example + particle-burst engine gap

**Date:** 2026-05-31
**Status:** PLANNED
**Bead(s):** none (`bd` not installed)
**Epic:** Playable examples for v1.0.0 dogfooding (`docs/NEXT_WORK.md`)
**Chain:** `sokoban-example-game` seq `1` (paired with the Sokoban handoff; D is the next work this chain points to)
**Context:** See `HANDOFF_sokoban-example-game_2026-05-31.md` for the just-shipped Sokoban session, the `History<T>` precedent, and the **pre-gathered candidate-D engine API inventory** (particle/pool/timer/collision/audio signatures).

---

## Problem Statement

The engine's dogfooding loop (`docs/VISION.md`) says a feature is not done until a small playable example exercises it. Candidates A/B/C/E/F are done; **D (simple shooter)** is the last recommended breadth item. A shooter is the first example to stress `ParticleEmitter`, `Timer`-driven spawning, collision layers under many simultaneous entities, and audio buses together — the combination most likely to surface real friction (`docs/NEXT_WORK.md` predicts "pooling / spawn bursts, perf"). It also complements the sibling `rust-survivors` game. The key known gap (see handoff Evidence): `ParticleEmitter` is **continuous-only**, so hit/explosion bursts have no clean API.

## Key Findings (from the Sokoban session + research)

- `ParticleEmitter` has no one-shot burst — only `spawn_rate`/sec + `emit: bool`. → **drives Phase 3 engine gap fix.**
- `engine::Pool` (`acquire`/`release`) already exists and is the intended tool for bullet churn. → drives Phase 1 (use it, don't hand-roll).
- `CollisionGridSystem::new(cell)` mirrors `SpatialGrid` to a `World` resource each frame; `query_aabb(min,max,mask)` + `CollisionLayer` is the proven maze pattern. → drives Phase 2 hit detection.
- `Timer::repeating` covers fire cooldown and wave cadence. → drives Phases 1–2.
- `AudioManager::play_tone(channel, freq, dur, vol)` gives placeholder sfx with no asset files; `set_bus_volume(bus, v)` for mixing. → drives Phase 3 audio.
- Immediate-mode `DebugDrawQueue` rendering worked great for Sokoban, but a shooter has many moving sprites — **use persistent ECS `Sprite` entities (maze pattern), not immediate-mode**, for bullets/enemies/player.
- `History<T>` is irrelevant to a shooter; do not add undo.

## Anti-Goals (What NOT To Do)

- Do **not** re-render via `DebugDrawQueue` for the gameplay entities — that fit a static grid, not a particle-heavy action game. Use `Sprite` + `Transform`.
- Do **not** spawn/despawn bullets ad hoc each shot — route through `engine::Pool` to validate it (the whole point of surfacing the pooling gap).
- Do **not** build a full rust-survivors clone — keep it a *small* example (one enemy type, simple waves, score + lives).
- Do **not** add physics (`rapier2d`) — use `SpatialGrid`/`Collider` AABB overlap like maze; rapier is overkill and complicates wasm.
- Do **not** change any existing public API signature — engine additions must be additive (preserve `rust-survivors`).

## Plan

### Phase 1: Scaffold — player, movement, pooled bullets, fire cooldown

**Goal:** A window where the player ship moves and fires pooled bullets that travel and expire.

**Why this approach:** Bullets are the highest-churn entity; wiring `Pool` first validates the pooling path early and surfaces its ergonomics before waves pile on.

- Create `examples/games/shooter/shooter.rs`; register `[[example]] name = "shooter_game"` in `Cargo.toml` (mirror sokoban entry).
- `WindowConfig` ~720×900 portrait (vertical shooter), dark clear color.
- Player: `Sprite::colored` + `Transform`; read `InputState` (WASD/Arrows) for movement, clamp to screen; `Space` to fire gated by a `Timer::repeating(fire_cooldown)` stored in a `Shooter` session resource.
- Bullets: a `Pool` (capacity ~64) in the session; on fire, `pool.acquire(world, |w,e| { add Transform at muzzle, Sprite, Velocity, Bullet tag, CollisionLayer(BULLET) })`.
- `BulletSystem`: advance bullets by velocity*dt; when off-screen, `pool.release(e, world)`.
- HUD via `TextQueue` (controls line), like sokoban/maze.

**Files:** `examples/games/shooter/shooter.rs` (new), `Cargo.toml` (+example entry).
**Validates with:** `cargo run --example shooter_game` — ship moves, bullets stream up and disappear off-screen; `cargo build` + `clippy` clean.
**Rollback:** delete the example file + Cargo entry.

### Phase 2: Enemy waves + collisions + score/lives

**Goal:** Timer-spawned enemies descend; bullets kill them and award score; enemy-player contact costs a life; game-over/restart.

**Why this approach:** Reuses the maze `SpatialGrid` resource pattern (proven, wasm-safe) instead of rapier; `Timer::repeating` is the minimal wave scheduler.

- Add `CollisionGridSystem::new(cell)` to the schedule; tag enemies/bullets/player with `Collider::Aabb` + `CollisionLayer`.
- `WaveSystem`: a `Timer::repeating` spawns a row/cluster of enemies at the top with downward velocity; escalate spawn rate over time (optional).
- `CollisionSystem`: for each bullet, `grid.query_aabb` against the enemy layer; on hit, release bullet to pool, despawn enemy, `score += 1`, emit a hit event/flag for Phase 3 fx. For enemy↔player overlap, `lives -= 1` + brief invuln.
- Session `Status { Playing, GameOver }`; on GameOver show text; `R` restarts (reset pool, score, lives, clear enemies).
- HUD: score + lives.

**Files:** `examples/games/shooter/shooter.rs`.
**Validates with:** play — shooting enemies increments score, enemies reaching the player reduce lives, 0 lives → GameOver, `R` restarts cleanly (no leaked pooled entities: assert `pool.available_count()` returns to capacity after restart).
**Rollback:** revert to Phase 1 state (movement + bullets only).

### Phase 3: Juice (particles + audio) and the engine gap fix

**Goal:** Explosions on enemy death and fire/explosion sfx — and close the `ParticleEmitter` one-shot-burst gap in the engine.

**Why this approach:** This is the dogfooding payoff: the example *needs* a burst, the engine only does continuous, so we add the missing primitive — the same "example surfaces gap → fix engine" loop as Sokoban's `History<T>`.

- **Engine fix:** add a one-shot burst to `src/particle.rs`. Options to decide while implementing: (a) `ParticleEmitter::emit_count: Option<u32>` consumed once by `ParticleSystem`, or (b) a helper `ParticleEmitter::burst(n)` constructor + a `ParticleBurst { remaining }` component the system drains then despawns the emitter. Prefer the smallest additive change; keep continuous behavior unchanged (back-compat). Add a unit test (mirror `History` test rigor: spawn-count after one tick, emitter retired after burst).
- Re-export any new type from `engine::` (`src/lib.rs`); document in the field/doc comment.
- **Example use:** on enemy death, spawn a short-lived burst emitter at the enemy position; let `ParticleSystem` clean it up.
- **Audio:** `AudioManager::play_tone` for fire + explosion (no asset files needed); route to an "sfx" bus and set bus volume. Confirm channel→bus mapping in `src/audio.rs` while wiring.

**Files:** `src/particle.rs` (+ burst API + test), `src/lib.rs` (re-export if new type), `examples/games/shooter/shooter.rs`.
**Validates with:** `cargo test --lib particle` (new burst test passes); play — death spawns a finite puff that fades and is cleaned up; sfx audible. clippy clean.
**Rollback:** revert `src/particle.rs` + `src/lib.rs`; example falls back to no-fx (still playable).

### Phase 4: Verify, document, ship

**Goal:** Meet the same proof bar Sokoban met and land the work.

- `cargo build` + `cargo build --target wasm32-unknown-unknown` for **lib and `shooter_game` example**.
- `cargo clippy --lib --example shooter_game` → 0 warnings; `cargo fmt --check`.
- `cargo test --lib` → all pass (incl. new particle burst test).
- Startup smoke-run (background, ~6s, no panic). **Flag interactive play for the user** — GUI can't be visually verified here (same limitation as Sokoban).
- Docs: `docs/NEXT_WORK.md` (mark D done + note the particle-burst gap closed; A–F all done → update recommended order / declare breadth pass complete), `docs/HANDOFF.md` (row + status), `CLAUDE.md` module map if a new public type was added.
- Branch `feat/shooter-example` off the then-current base; commit (`Co-Authored-By: Claude Opus 4.8`); push; PR (ask user for base — `main` vs the english-conversion stack, per Open Questions).

**Files:** docs as above.
**Validates with:** all commands green; PR opened.
**Rollback:** n/a (final phase).

## Dependencies & Order

- Strictly sequential: Phase 1 → 2 → 3 → 4. Phase 2 needs Phase 1's bullets; Phase 3's example fx need Phase 2's death events.
- The Phase 3 **engine** burst fix is independent of the example and could be done first/in parallel (worktree) if desired, but is cheap enough to keep inline.

## Risks & Mitigations

- **`Pool` ergonomics awkward for ECS bullets** (likely, per NEXT_WORK) → that *is* a finding; capture it and, if bad, propose an additive helper rather than abandoning `Pool`. Don't silently hand-roll spawn/despawn.
- **Particle burst API shape bikeshed** (medium) → pick the smallest additive option, ship it, note the alternative in the doc comment; don't block the example on it.
- **Audio on wasm / no audio device** (medium) → `play_tone` may no-op without a device; wrap in best-effort and never panic (Sokoban's save did the same).
- **Perf with many bullets+enemies+particles** (low for a small example) → cap pool size and wave count; if frame drops, note it as a profiling finding (engine has `ProfilerData`).
- **wasm build breaks from a new dep** (low) → don't add deps; reuse existing modules only.

## Success Criteria

- **Minimum viable:** `cargo run --example shooter_game` is a playable shooter — move, shoot, kill enemies, score, lose lives, game-over, restart — and native+wasm build, clippy, and `cargo test --lib` are all green.
- **Full success:** the `ParticleEmitter` one-shot-burst gap is closed with an additive API + unit test and re-exported; explosions + sfx work; `pool.available_count()` returns to capacity after restart (no leaks); docs updated marking the breadth-first example pass (A–F) complete.

## Quick Start

```bash
# Restore full context
cat plans/handoffs/HANDOFF_sokoban-example-game_2026-05-31.md   # see "Candidate D engine API inventory"

# Sanity-check the just-shipped Sokoban before starting D
cargo run --example sokoban_game

# Key source files to read before Phase 1
src/particle.rs            # ParticleEmitter/ParticleSystem — confirm burst gap
src/pool.rs                # Pool::acquire/release for bullets
src/timer.rs               # Timer::repeating for cooldown/waves
examples/games/maze_escape/maze_escape.rs  # CollisionGridSystem + SpatialGrid + Sprite pattern to copy
src/audio.rs               # play_tone + bus volume

# Verify starting state (should be clean/green on the chosen base)
cargo build && cargo clippy --lib && cargo test --lib

# First concrete action (Phase 1)
# Create examples/games/shooter/shooter.rs and add the [[example]] entry to Cargo.toml,
# then implement player movement + a Pool<bullet> + Timer fire cooldown.
```
