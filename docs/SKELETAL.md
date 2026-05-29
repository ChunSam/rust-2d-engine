# Skeletal Animation (2D cutout)

> Module: `src/skeletal.rs` · Re-exports in `src/lib.rs` · Demo: `examples/skeletal_puppet.rs`

2D **cutout (rigged)** skeletal animation, in the style of Spine/DragonBones. Bones are
hierarchy entities (`Transform` + `Parent`); each visible part is a sprite attached to a
bone. A clip keyframes each bone's **local `Transform`**, and the existing
`HierarchySystem` composes the world-space `GlobalTransform` that the renderer already
prefers. There is **no renderer change** — bone sprites draw through the normal path.

This is intentionally not mesh skinning (no vertex weights / deformation). Cutout reuses
existing infrastructure and is enough for most 2D games; mesh skinning would require a
custom vertex pipeline and is out of scope.

## Data model

| Type | Role |
|------|------|
| `BoneKeyframe { time, position, rotation, scale }` | one bone pose at a time (seconds) |
| `BoneTrack { bone: String, keys }` | keyframes for one bone, `time`-ascending |
| `SkeletalClip { name, duration, looping, tracks }` | a named animation |
| `SkeletalAnimator { clips, current, time, speed, playing, bones }` | component on the skeleton root; `bones` maps name → entity |
| `SkeletalAnimationSystem` | advances `time`, samples tracks, writes bone `Transform`s |
| `SkeletonBuilder` | spawns the bone hierarchy and the name→entity map |

Interpolation: position/scale are linear (`Vec2::lerp`); rotation uses shortest-path
angle interpolation (`lerp_angle`).

## Frame order

`SkeletalAnimationSystem` runs in the **user-system phase** and must be registered:

```rust
app.add_system(SkeletalAnimationSystem);
```

It writes each bone's local `Transform`. Afterwards the automatically-run `HierarchySystem`
(`src/app.rs`, right after user systems) composes `GlobalTransform` down the chain, and the
sprite renderer draws bone sprites at those world transforms.

## Authoring with `SkeletonBuilder`

```rust
let mut b = SkeletonBuilder::new(&mut world, "hip", root_transform);
b.add_bone(&mut world, "torso", "hip", local_transform, Some(Sprite::colored(..)));
// ... more bones ...
let root = b.finish(&mut world, vec![idle_clip, wave_clip]);
```

`add_bone(world, name, parent_name, local_transform, sprite)` reuses
`hierarchy::attach()` to wire `Parent`/`Children`. `finish()` inserts the
`SkeletalAnimator` on the root.

Switch clips at runtime: `animator.play(index)` or `animator.play_named("wave")`.

## Scale composition note

A bone entity's `Transform.scale` is used both for hierarchy composition **and** as the
sprite quad size, and scale multiplies down the chain. To avoid runaway sizes, keep
articulated **joint** bones at `scale = 1.0` (no sprite) and attach **leaf** visual
sprites as children whose `scale` is the part's pixel size. The demo
(`examples/skeletal_puppet.rs`) follows this pattern via its `add_visual` helper. A future
revision may separate "bone length" from "attachment size" to remove this manual rule.

## Hierarchy depth

`HierarchySystem` propagates `GlobalTransform` in topological (root→child) order in a
single pass, so **arbitrary depth** is supported (the demo's right-arm chain is depth 5:
hip→torso→r_upper_arm→r_forearm→r_hand). This replaced an earlier 2-pass implementation
that capped at depth 3 — a limit surfaced precisely by building this demo.

## Demo

`cargo run --example skeletal_puppet` — a humanoid puppet built from colored rectangles.
Space toggles idle ↔ wave; Esc quits.
