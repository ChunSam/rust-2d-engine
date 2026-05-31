# Entity Generation v2 Plan

> **Status: Cancelled / archived (2026-05-29).** Removed from the planned/scheduled work
> during the vision reset (`docs/VISION.md`): it is a v2-only breaking change, not breadth
> and not example-validated, so it does not match the current direction. The design below
> is preserved for reference in case generation-checked handles are revisited for a future
> v2.0.0.

Written: 2026-05-29
Status: Cancelled / archived (originally: finalized v2 candidate design)

## Summary

Today `Entity(pub u32)` reuses IDs after despawn. A long-held `Entity` value can point at
a new entity, so `SelectedEntity`, `Parent`, `Children`, script despawn requests, and
game-code caches can act on the wrong entity.

In v1 this behavior is documented and `is_alive()` checks are recommended. In v2, `Entity`
becomes a generation-tagged handle so stale handles fail automatically.

## Decisions

- In v2, `Entity` becomes an opaque handle rather than a tuple struct.
  ```rust
  pub struct Entity {
      index: u32,
      generation: u32,
  }
  ```
- Public access is via methods.
  - `Entity::index(self) -> u32`
  - `Entity::generation(self) -> u32`
  - `Entity::from_raw_parts(index: u32, generation: u32) -> Self`
- Remove `Entity(pub u32)` and direct `entity.0` access. UI and logs use `entity.index()`;
  debug strings use the `Debug` output or `format!("Entity {}:{}", entity.index(), entity.generation())`.
- `World` keeps the current generation per slot.
  - rename `next_id: u32` to `next_index: u32`.
  - rename `free_ids: VecDeque<u32>` to `free_indices: VecDeque<u32>`.
  - add `generations: Vec<u32>` so `generations[index]` is the current generation.
- `spawn()` behavior:
  - a fresh slot starts at `generation = 0`.
  - a reused slot uses the already-incremented `generations[index]`.
- `despawn(entity)` behavior:
  - if `is_alive(entity)` is false, it is an immediate no-op.
  - on successful removal, increment that index's generation by 1 and push the index onto the free queue.
  - if generation reaches `u32::MAX`, retire that index instead of reusing it.
- All `World` APIs operate only on live handles with a matching generation.
  - `get`, `get_mut`, `add_component`, `remove_component`, `take_component`, `mark_changed`, `clone_entity`, `has_component_typeid` treat a stale handle as a missing entity.
  - `query*`, `entities()`, `par_query*` always return only live handles of the current generation.
- The `entity_location` key keeps `Entity`. A different generation is a different key, so stale-handle lookups fail.
- The change-tracking sets (`added_this_tick`, `changed_this_tick`) keep `(Entity, TypeId)` as-is. On despawn, removing only the current handle's entries avoids colliding with stale-generation entries.

## Migration impact

- External code must change `entity.0` to `entity.index()`.
- The old behavior where a long-held `Entity` manipulated a new entity fails in v2. This is the intended breaking fix.
- `Commands::despawn/insert/remove` follow the stale-handle no-op policy with no extra change.
- `Parent(Entity)`, `Children(Vec<Entity>)`, `SelectedEntity`, and `Entity` in event payloads keep their types but gain stale safety.
- `ScriptingSystem::despawn_entity(id)` drops the current `i64 -> Entity(index)` conversion in v2. The script API changes to either `despawn_entity(index, generation)` or an engine-issued opaque handle string. The recommendation is two arguments `index, generation`, matching the Rust API.
- Scene/prefab serialization keeps tag- and hierarchy-name-based restoration. Runtime `Entity` values are not written to the save format.

## Implementation order

1. Add the `Entity` struct and accessor methods, and replace internal `entity.0` use with `entity.index()`.
2. Add `generations` and `free_indices` to `World` and switch `spawn/despawn/is_alive` to generation-checked logic.
3. Add unit tests that the stale-handle no-op policy holds across all component APIs and command paths.
4. Update editor/debug UI display, hierarchy, prefab, scripting, and physics/network event call sites to the new accessors.
5. Add the v2 breaking change and migration notes to `REFERENCE.html`, `README.md`, and `docs/CHANGELOG.md`.

## Required tests

- After `despawn`, even if the same index is reused, the previous `Entity` must fail or no-op for `is_alive`, `get`, `get_mut`, `add_component`, `remove_component`, and `despawn`.
- A reused new `Entity` must have the same index and an incremented generation, and must query normally.
- Pushing a stale handle into `Commands` and calling `apply_commands` must not change the new entity.
- A stale handle left in `Parent`/`Children` must not let a hierarchy update mistake the new entity for a parent/child.
- `clone_entity(stale)` must not create an empty new entity — either change it to return `Option<Entity>`, or, if the existing signature is kept, explicitly test the empty-entity creation policy. The recommendation is to change it to `clone_entity(src) -> Option<Entity>` in v2.

## Decisions that will not be deferred

- v2 keeps no backward-compatible tuple field.
- v2 does not panic on stale handles. To keep the game loop stable, it keeps the current no-op-family policy.
- This change does not land in v1.x. It breaks the public API and the `entity.0` usage pattern, so it is v2-only.
