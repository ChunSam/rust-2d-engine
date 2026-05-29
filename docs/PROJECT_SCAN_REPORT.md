# Project Scan Report

작성일: 2026-05-29 / 갱신: 2026-05-29

## 개요

`skeleton-engine`은 Rust 기반 2D 게임 엔진이다. 패키지 이름은 `skeleton-engine`이고, 라이브러리 크레이트 이름은 `engine`이다.

주요 구성은 다음과 같다.

- 렌더링: `wgpu`, 스프라이트, 텍스트, UI, 포스트 프로세스, 렌더 텍스처, GPU 파티클
- ECS: custom archetype storage, commands, events, schedule ordering
- 물리: Rapier2D 기반 physics world, body, trigger, raycast, character controller, joints
- 게임 기능: input, gamepad, touch, audio, particle, tilemap, animation, scene, prefab, save/load
- 확장 기능: scripting, reflection, networking, localization, debug UI, WASM support

## 주요 파일

| 파일 | 역할 |
| --- | --- |
| `Cargo.toml` | 패키지 메타데이터, 의존성, feature/target별 설정, package include 목록 |
| `src/lib.rs` | 공개 모듈과 re-export 진입점 |
| `src/app.rs` | 앱 루프, winit/wgpu 초기화, 렌더링 orchestration, asset load helper |
| `README.md` | 사용자용 설치, quick start, 검증 명령 |
| `AGENTS.md` | 에이전트용 모듈 맵과 작업 규칙 요약 |
| `REFERENCE.html` | 공개 API reference |

## 검증 결과

아래 검증은 로컬에서 실행해 통과를 확인했다. 문서 정리 후 최종 릴리즈 검증도 추가로 통과했다. post-v1.0 안정성 개선 작업 후에는 전체 테스트와 clippy를 다시 통과했다.

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --all-targets` | 초기 스캔 시 통과: library 203 tests, `mp_server` example 3 tests |
| `cargo test --doc` | 통과: 31 passed, 19 ignored |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` | 통과 |
| `cargo build --target wasm32-unknown-unknown` | 통과 |
| `cargo test` | 문서 정리 후 통과: unit 207 passed, doctest 31 passed / 19 ignored |
| `cargo clippy --all-targets --locked -- -D warnings` | 문서 정리 후 통과 |
| `cargo package --locked --allow-dirty --list` | 문서/API 정리 후 통과: 101 files, 1.1MiB |
| `cargo publish --dry-run --locked --allow-dirty` | 문서/API 정리 후 통과: dry-run upload aborted as expected |
| `cargo test --all-targets` | post-v1.0 안정성 개선 후 통과: library 218 tests, `mp_server` example 3 tests |
| `cargo clippy --all-targets -- -D warnings` | post-v1.0 안정성 개선 후 통과 |
| `cargo run --example runtime_policies` | 런타임 정책 예제 추가 후 통과 |

참고: 일부 Cargo 명령은 sandbox 환경에서 `target/debug/.cargo-lock` 접근이 `Operation not permitted`로 막혀, sandbox 밖 실행 권한으로 검증했다. 현재 문서/API 정리 변경은 커밋 전이라 패키징 검증에는 `--allow-dirty`를 사용했다.

## 작업트리 상태

초기 스캔 시점의 브랜치는 `main`이고 최근 커밋은 `0e01b0f feat: add UI image and UV helper APIs`였다. 당시 작업트리는 문서 재배치와 UV 수정 때문에 dirty 상태였다.

갱신 기준 커밋에서는 문서 재배치와 UV 수정이 `6e6f5fa fix: align UV orientation and docs layout`로 커밋되어 `origin/main`에 push됐다. 그 커밋 기준 tracked 작업트리는 clean이며, 남은 항목은 `.gitignore` 대상 로컬 산출물뿐이었다. 이후 문서/API 정리 변경은 별도 검증 및 커밋 대상이다.

- 공식 문서: 루트 `README.md`, `REFERENCE.html`, `AGENTS.md`, `CLAUDE.md`, `docs/*.md`
- crates.io 패키지 포함 문서: `README.md`, `REFERENCE.html`, `docs/CHANGELOG.md`
- ignored 로컬 문서: 작업 프롬프트/개인 계획으로 공식 문서에서 제외

## 관찰 사항

- 코드, 테스트, rustdoc, WASM 빌드 상태는 양호하다.
- `TODO`, `FIXME`, `HACK`, `BUG` 표식은 `src`, `docs`, `examples`에서 발견되지 않았다.
- 런타임 `unwrap` 후보는 대부분 테스트, 문서 예제, 또는 쿼리로 수집한 엔티티를 같은 프레임에 재조회하는 내부 불변식 기반 코드였다.
- 문서 재배치는 완료됐고, 현재 문서/API 정리 변경은 검증 완료 후 커밋 대기 상태다. 남은 실무 리스크는 문서 내용이 소스와 계속 동기화되는지 확인하는 유지보수다.
- 루트 `README.md`는 유지되며, Cargo 패키지 readme도 `README.md`로 지정되어 있다.
- post-v1.0 안정성 개선으로 스케줄 순환/시스템 panic 정책을 opt-in으로 엄격화할 수 있게 됐고, 기본 동작은 기존 호환성을 유지한다.
- `examples/runtime_policies.rs`가 추가되어 엄격 런타임 정책 설정 형태를 별도 창 실행 없이 확인할 수 있다.
- `Entity` 세대 번호 도입은 v2 파괴적 변경으로 분리하고, `docs/ENTITY_GENERATION_V2_PLAN.md`에 구현 결정과 테스트 기준을 정리했다.
- ECS 직접 필드 수정은 여전히 자동 changed 감지가 아니며, `mark_changed<T>()` 또는 `get_mut_tracked<T>()`를 사용해야 한다.
- 네이티브 에셋 경로는 존재하는 파일에 한해 canonical path로 정규화된다. WASM URL/상대 경로와 누락 파일 fallback은 기존 동작을 유지한다.

## 권장 다음 작업

1. 공식 릴리즈를 실제 게시하려면 `cargo publish --locked`를 별도로 실행한다.
2. v2 개발을 시작할 때 `docs/ENTITY_GENERATION_V2_PLAN.md`를 기준으로 `Entity` 세대 번호 구현을 진행한다.
3. ignored 로컬 문서가 더 이상 필요 없으면 사용자 확인 후 삭제한다.
