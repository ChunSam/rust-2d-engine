# Project Scan Report

작성일: 2026-05-29

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

아래 검증은 로컬에서 실행해 통과를 확인했다.

| 명령 | 결과 |
| --- | --- |
| `cargo fmt --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --all-targets` | 통과: library 203 tests, `mp_server` example 3 tests |
| `cargo test --doc` | 통과: 31 passed, 19 ignored |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` | 통과 |
| `cargo build --target wasm32-unknown-unknown` | 통과 |

참고: 일부 Cargo 명령은 sandbox 환경에서 `target/debug/.cargo-lock` 접근이 `Operation not permitted`로 막혀, sandbox 밖 실행 권한으로 검증했다.

## 작업트리 상태

스캔 시점의 브랜치는 `main`이고 최근 커밋은 `0e01b0f feat: add UI image and UV helper APIs`였다.

작업트리는 dirty 상태였다.

- 수정됨: `AGENTS.md`, `CLAUDE.md`, `Cargo.toml`
- 삭제됨: 루트 `CHANGELOG.md`, `HANDOFF.md`, `REFERENCE.md`, `ROADMAP.md`
- 추가됨: `REFERENCE.html`, `docs/`
- ignored 상태: `docs/ENGINE_REVIEW_FIX_PROMPT.md`, `docs/PARALLEL_TASKS.md`, `docs/REMAINING_WORK.md`, `docs/rust_game_engine_plan.md`

`cargo package --locked --list`는 dirty worktree 때문에 실패했다. `--allow-dirty`를 붙이면 package include 목록 생성은 가능했다.

## 관찰 사항

- 코드, 테스트, rustdoc, WASM 빌드 상태는 양호하다.
- `TODO`, `FIXME`, `HACK`, `BUG` 표식은 `src`, `docs`, `examples`에서 발견되지 않았다.
- 런타임 `unwrap` 후보는 대부분 테스트, 문서 예제, 또는 쿼리로 수집한 엔티티를 같은 프레임에 재조회하는 내부 불변식 기반 코드였다.
- 현재 가장 큰 실무 리스크는 코드 품질보다 문서 재배치와 미커밋 상태 정리다.
- 루트 `README.md`는 유지되며, Cargo 패키지 readme도 `README.md`로 지정되어 있다.

## 권장 다음 작업

1. 문서 재배치가 의도된 변경인지 확정한다.
2. ignored 처리된 `docs/*.md` 문서를 로컬 작업 문서로 둘지, 공유 문서로 추적할지 결정한다.
3. 릴리즈/패키징 전에 dirty worktree를 정리한 뒤 `cargo package --locked`와 `cargo publish --dry-run --locked`를 다시 실행한다.
