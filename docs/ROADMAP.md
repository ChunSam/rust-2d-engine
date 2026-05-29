# skeleton-engine v1.0 completion roadmap

> Written: 2026-05-25 | Document status: historical roadmap
> A record preserving the scope and completion criteria of the once-targeted **v1.0.0 official-release preparation**. Post-v1.0 new plans are covered in separate documents.

---

## Principles at the time

- **Detailed plans drafted just before each Phase** — this document preserves the goals, scope, and priorities of the time
- **Parallelism first** — work without file conflicts always proceeds concurrently
- **Explicit completion criteria** — each Phase counts as done only when it meets quantitative criteria
- **Backward compatibility** — breaking changes allowed until v1.0.0, strict semver afterward

---

## Milestone overview

```
v0.44  ████████████████████████ done
         │
v0.50  ──┤ Milestone 1 — game-making capable, done
         │  essential render/physics/input features complete
         │
v0.57  ──┤ Milestone 2 — commercial-shippable, done
         │  quality, stability, platform, packaging complete
         │
v1.0.0 █─┤ Milestone 3 — release-ready
            docs, API freeze, ecosystem
```

---

## Milestone 1 — game-making capable (v0.45 ~ v0.50)

> Goal at the time: a level able to make a finished indie game in any genre

### Phase 44 — physics extension + audio effects

**Priority**: high | **Parallel**: 44b + 44c possible

| Sub | Feature | Completion criteria |
|------|------|-----------|
| 44b | Physics joints (`DistanceJoint`, `RevoluteJoint`, `PrismaticJoint`) | Wrap the Rapier2D joint API, unit tests |
| 44c | Audio effects (`LowPassFilter`, pitch shift, volume envelope) | Applied at runtime, parameters changeable in real time |

**Background**: platformer/puzzle genres are impossible without joints. Audio effects directly affect game-feedback quality.

---

### Phase 45 — explicit system execution order

**Priority**: very high (structural foundation) | **Standalone**

| Sub | Feature | Completion criteria |
|------|------|-----------|
| 45a | `SystemLabel` + `before/after` dependency declarations | DAG ordering, cycle-detection error |
| 45b | `SystemSet` (group-level on/off) | Bulk-disable debug-mode systems |

**Background**: relied on `add_system` call order — a scene swap could flip ordering and cause bugs. A core indicator of engine maturity.

---

### Phase 46 — render textures

**Priority**: high | **Standalone**

| Sub | Feature | Completion criteria |
|------|------|-----------|
| 46a | `RenderTarget` resource — offscreen rendering | Minimap example works |
| 46b | Render texture → used as a Sprite | Split-screen example |

**Background**: without minimap, portals, UI cameras, and split screen, strategy/action genres are limited.

---

### Phase 47 — touch input + mobile support

**Priority**: high | **Parallel**: 47a + 47b possible

| Sub | Feature | Completion criteria |
|------|------|-----------|
| 47a | `TouchState` — multitouch, swipe, pinch-zoom | Works on Android/iOS simulators |
| 47b | Virtual joystick UI widget | Mobile game demo works |

**Background**: a large share of indie-game revenue is mobile. Without touch, you cannot ship to the iOS/Android App Store.

---

### Phase 48 — physics layers/masks + triggers

**Priority**: medium | **Parallel**: 48a + 48b possible

| Sub | Feature | Completion criteria |
|------|------|-----------|
| 48a | `CollisionGroups` (bitmask) — selective collision | Per-layer collision-ignore works |
| 48b | Sensor (trigger zone) — detect entry into a non-colliding area | `TriggerEvent` confirmed to fire |

**Background**: all objects colliding with each other causes performance/logic problems in complex games.

---

### Phase 49 — text completion

**Priority**: medium | **Parallel**: 49a + 49b + 49c possible (49a/b are Track B, 49c is Track A)

| Sub | Feature | Completion criteria |
|------|------|-----------|
| 49a | Multiline text + word wrap + alignment | Multilingual paragraphs render correctly |
| 49b | Rich text (`[color]...[/color]`, bold, italic) | In-game dialogue-box example |
| 49c | IME composition input (KR/JP/CN) — handle winit `Ime` events, render preedit preview during composition | Nickname/chat input via direct Hangul entry works |

**Background**: text supported only a single font/size/color — a bottleneck for dialogue/UI quality. Also, `TextInput` handled only ASCII chars, so nickname/chat input was impossible for an Asian release → winit IME composition events are essential (49c).

---

## Milestone 2 — commercial-shippable (v0.51 ~ v0.57)

> Goal at the time: reach the quality and stability needed for actual store (Steam/iOS/Android) distribution

### Phase 50 — localization (i18n)

**Priority**: high | **Standalone**

| Sub | Feature | Completion criteria |
|------|------|-----------|
| 50a | `LocaleResource` — load `.ftl` or `.json` translation files | KR/EN/JP switching works |
| 50b | `t!("key")` macro or API + per-locale fonts | Basic RTL-language (Arabic) support |

**Background**: bolting on i18n before a global launch would require rewriting the whole UI. Designing it early is essential.

---

### Phase 51 — async asset loading

**Priority**: high | **Parallel**: 51a + 51b possible

| Sub | Feature | Completion criteria |
|------|------|-----------|
| 51a | Async scene load — `LoadingScene` + progress resource | Loading-bar example works (no freezing) |
| 51b | WASM `fetch` API integration — async asset download in the browser | Loads external assets in a WASM build |

**Background**: synchronous loading → multi-second freezes on large scenes. Fatal in store reviews.

---

### Phase 52 — stability + panic recovery

**Priority**: very high | **Parallel**: 52a + 52b possible

| Sub | Feature | Completion criteria |
|------|------|-----------|
| 52a | `std::panic::catch_unwind` wrapper — isolate system panics | Engine survives a system crash |
| 52b | Error-log file writing + crash-report format | `crash.log` generated automatically |

**Background**: runtime errors like missing assets or a bad scene file crashed immediately → not commercial-ready.

---

### Phase 53 — save-data security

**Priority**: medium | **Parallel**: 53a + 53b possible

| Sub | Feature | Completion criteria |
|------|------|-----------|
| 53a | Save-file encryption (ChaCha20-Poly1305 AEAD) | Replaces plaintext RON, loads with decryption |
| 53b | Checksum verification — detect file tampering | `SaveError::Corrupted` on tampering |

**Background**: a plaintext RON save can be easily manipulated, so AEAD encryption and tamper detection were planned during v1.0 prep.

---

### Phase 54 — editor completion

**Priority**: medium | **Parallel**: 54a + 54b possible

| Sub | Feature | Completion criteria |
|------|------|-----------|
| 54a | Multi-select + group move + Ctrl+C/V | Edit multiple entities at once |
| 54b | Prefab system improvements — instance override, prefab break | Complete level-design workflow |

---

### Phase 55 — distribution packaging

**Priority**: very high (release prerequisite) | **Track B** (build infra, unrelated to app.rs → parallelizable in another session)

| Sub | Feature | Completion criteria |
|------|------|-----------|
| 55a | Desktop bundling — executable binary + bundled `assets/`, relative-path asset loading | Win/Mac/Linux run immediately after unzip |
| 55b | Asset packing (optional) + WASM distribution bundle — `index.html` + wasm + assets | itch.io-upload zip, static hosting works |

**Background**: only `crate-type = ["cdylib", "rlib"]` + `wasm-pack` builds existed. There was no packaging path to get a native game onto itch.io/Steam — a core gap for "commercial-shippable" (Milestone 2).

---

## Milestone 3 — official release (v0.58 ~ v1.0.0)

> Goal at the time: docs, stability, and ecosystem at a level external developers can use independently

### Phase 56 — advanced rendering

**Priority**: low | **Parallel**: 56a + 56b possible

| Sub | Feature |
|------|------|
| 56a | GPU particles (compute shader, complementing/replacing existing CPU particles) |
| 56b | Color grading (LUT-based post-processing) |

---

### Phase 57 — rustdoc completion + CI

**Priority**: very high (entry point for external users) | **57a/b are Solo, 57c is Track B**

| Sub | Feature | Completion criteria |
|------|------|-----------|
| 57a | Rustdoc for the whole public API | `cargo doc --no-deps` with 0 warnings |
| 57b | Per-module `#[doc = include_str!("...")]` guides | Each subsystem includes overview + examples |
| 57c | GitHub Actions CI — `cargo build`/`test`/`fmt --check`/`clippy` workflow | Auto-pass gate per PR (`.github/workflows/ci.yml`) |

---

### Phase 58 — examples + sample game

**Priority**: high | **Parallel**: 58a + 58b possible (Track B — edits only examples/)

| Sub | Feature | Completion criteria |
|------|------|-----------|
| 58a | 10 `examples/` — a minimal example per subsystem | All `cargo run --example <name>` work |
| 58b | 1 sample minigame (platformer or top-down RPG) | Actual finished-game quality |

---

### Phase 59 — API freeze + v1.0.0 release

**Priority**: top | **Solo (last)**

| Item | Content |
|------|------|
| API audit | Full public-API review — naming consistency, remove unnecessary exposure |
| Semver declaration | Breaking changes afterward go to v2.0.0 — introduce CHANGELOG format |
| Performance benchmark | Document the 10,000-entity 60fps baseline |
| Security audit | Full review of unsafe blocks, confirm memory safety |
| Release gate | Confirm green CI + `cargo publish --dry-run` passing as preconditions |
| Official release | Prepare GitHub Release + crates.io publish |

---

## Priority matrix

```
                 high impact            low impact
easy to build  system ordering (45)   save encryption (53)
               physics layers (48)     color grading (56b)
               distribution pkg (55)
hard to build  render textures (46)    GPU particles (56a)
               touch input (47)        2D skeletal animation (post-v1.0)
               localization (50)
```

**Recommended starting order at the time**: Phase 44 → 45 → 46 (these three are the foundation for all later features)

---

## Concurrency guide (multi-session)

> Criteria for distributing work across multiple Claude Code sessions. The key conflict bottleneck is **`src/app.rs` (~2160 lines)** — the whole render-pass chain (1501–1832), input dispatch (window_event 1893–2042), system update loop (630–651), and editor UI (668–1458) are all in one file.

### Track model

- **Track A (app.rs serial)** — modifies a specific region of app.rs. Only **one at a time**, sequential.
- **Track B (isolated/parallel)** — does not touch app.rs (new modules/independent files/build infra). One active Track A + multiple Track B can run **concurrently in different sessions**.
- **Solo** — doc comment/naming changes across all files. Always conflicts with other work → done alone, last.

### Per-phase file footprint

| Phase | Main files | app.rs region | Track | Concurrent with |
|-------|-----------|------------|------|----------------|
| 45 system ordering | `ecs/system.rs`, `ecs/world.rs`, `app.rs` | update loop (630–651) | A | 48, 50, 53, 55, 57c |
| 46 render textures | `renderer/*` (new), `app.rs` | render loop (1501–1832) | A | 48, 50, 53, 55, 57c |
| 47 touch input | `input/*`, `ui/`, `app.rs` | window_event (1893–2042) | A | 48, 50, 53, 55, 57c |
| 48 physics layers/sensors | `physics/*`, `collision/*` | none | **B** | almost all |
| 49a/b text | `renderer/text.rs`, `ui/` | none/minor | B | almost all |
| 49c IME | `input/state.rs`, `ui/text_input.rs`, `app.rs` | window_event | A | 48, 50, 53, 55, 57c |
| 50 i18n | `locale.rs` (new), `ui/` | none | **B** | almost all |
| 51 async loading | `asset.rs`, `scene.rs`, `app.rs` | scene/asset wiring | A | 48, 50, 53, 55, 57c |
| 52 panic recovery | `app.rs`, `ecs/` | update loop (630–651) | A | 48, 50, 53, 55, 57c |
| 53 save encryption | `save.rs` | none | **B** | all (fully isolated) |
| 54 editor | `app.rs`, `debug_ui.rs` | editor UI (668–1458) | A | 48, 50, 53, 55, 57c |
| 55 distribution pkg | `Cargo.toml`, build scripts, `examples/` | none | **B** | almost all |
| 56 advanced rendering | `particle.rs`, `renderer/*`, `app.rs` | render loop | A | 48, 50, 53, 55, 57c |
| 57c CI | `.github/workflows/` | none | **B** | all (code-independent) |
| 57a/b rustdoc | doc comments across all files | broad | **Solo** | — |
| 58 examples+sample | `examples/` | none | B | almost all (after API is stable) |
| 59 API freeze | all files | broad | **Solo** | — |

### Operating rules

1. **Always exactly one active Track A phase.** Even with different app.rs regions (render loop vs window_event), git auto-merge failure risk is high, so serial is recommended.
2. **Track B (48, 50, 53, 55, 57c) runs concurrently with an active Track A + with each other** — distribute freely across sessions.
3. **57a/b (rustdoc) and 59 (API freeze) edit all files → always solo, last.**
4. **Risky pair**: 45 and 52 both modify the update loop (630–651) → conflict even when sequential. Fix the order **45 first (loop restructure) → 52 (loop wrapping)**.
5. lib.rs conflicts are at the level of a one-line re-export → trivial manual merge, not a reason to block parallelism.

### Example recommended concurrent layout

```
session1 (Track A): 45 → 46 → 47 → 49c → 51 → 52 → 54 → 56   (sequential)
session2 (Track B): 48 → 50 → 53                             (independent parallel)
session3 (Track B): 55 → 57c → 58                            (independent parallel)
       (after all done) 57a/b rustdoc → 59 API freeze         (solo, last)
```

---

## Per-version checklist (summary)

| Version | Completion condition |
|------|-----------|
| v0.46 | Phase 44 + 45 done. System order guaranteed, joints work |
| v0.48 | Phase 46 + 47 done. Render textures, touch input |
| v0.50 | Phase 48 + 49 done (incl. IME). Milestone 1 reached |
| v0.52 | Phase 50 + 51 done. Localization, async loading |
| v0.54 | Phase 52 + 53 done. Stability, save security |
| v0.57 | Phase 54 + 55 done. Editor, distribution packaging. Milestone 2 reached |
| v0.59 | Phase 56 + 57 done. Advanced rendering, docs, CI |
| v0.60 | Phase 58 done. Examples + sample game |
| **v1.0.0** | Phase 59 done. API freeze, crates.io publish dry-run passing and release-ready |

---

*Each Phase's detailed implementation plan is drafted separately just before that Phase starts.*
*This document is a historical roadmap preserving the direction and priorities of the v1.0 prep process.*
