# Security Hardening Notes - 2026-05

This note records the security and memory-safety hardening work from the May 2026 review.

## Completed

- Guarded the egui render path against custom paint callbacks before entering the internal unsafe render-pass helper. Callback primitives are skipped with a warning because the current `egui-wgpu` API requires a `'static` render pass lifetime.
- Added bounded inbound network event buffering with `NetworkConfig::max_pending_events` and `NetworkEvent::ReceiveQueueFull`.
- Updated the native websocket stack from `tungstenite 0.21` to `0.29`, which moves TLS verification to `rustls-webpki 0.103.13`.
- Updated the multiplayer server example for the `tungstenite 0.29` `Utf8Bytes` message API.
- Hardened `PathGrid` construction and indexing against negative sizes, integer overflow, and oversized allocations.
- Hardened `TilemapAtlas::uv_for` so zero-sized atlas grids return `UvRect::FULL` instead of panicking.
- Removed the redundant manual `unsafe impl Send/Sync for BehaviorTree`; the `BehaviorNode: Send + Sync` trait bound already guarantees thread-safety, and the hand-written impl would have silently masked unsoundness if that bound were ever relaxed.
- Repaired the `wasm32-unknown-unknown` build, which had regressed because the WebSocket `wasm_impl` module called `push_event_bounded` without importing it (native builds were unaffected).

## Verification

- `cargo check`
- `cargo test`
- `cargo tree -i rustls-webpki` confirms `rustls-webpki v0.103.13`.
- OSV batch query no longer reports the previous `rustls-webpki` advisories.

## Remaining Risk

- `glyphon 0.6.0` still depends on `lru 0.12.5`, which is affected by `RUSTSEC-2026-0002`.
- `lru >= 0.16.3` cannot be resolved under the current `glyphon 0.6` requirement.
- `glyphon 0.10+` moves to newer `wgpu` major versions, so fully removing this advisory should be handled as a separate renderer dependency migration.
- `paste 1.0.15` remains reported as unmaintained via transitive dependencies; this is not an active vulnerability but should be monitored during future dependency upgrades.
