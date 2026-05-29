# rust-survivors Texture Cache Key Fix Prompt

아래 프롬프트를 `rust-survivors` 게임 개발 작업 세션에 전달하세요.

---

## Prompt

`skeleton-engine`에서 이미지 텍스처 캐시 키 불일치가 수정되었습니다.

엔진 변경 내용:

- `AssetServer`가 반환하는 canonical `Handle<ImageAsset>::path()`와 `App::load_image(...)`에 전달한 원래 상대 경로가 모두 같은 GPU 텍스처를 찾을 수 있게 되었습니다.
- `Sprite::textured_with_handle(...)`, `DrawImage::textured_with_handle(...)`, `AtlasSprite`가 handle path를 우선 사용해도 더 이상 성공적으로 로드된 이미지가 흰색 사각형으로 렌더링되지 않아야 합니다.
- 공개 API 변경은 없습니다.

목표:

1. `rust-survivors`의 `skeleton-engine` 의존성을 texture cache key 수정이 포함된 엔진 커밋으로 갱신합니다.
2. 게임 쪽 임시 workaround인 `survivor_texture_handle` 계열 로직을 제거하고, `SurvivorTextureHandles`에 로드된 핸들을 항상 전달하도록 복구합니다.
3. 관련 문서의 open engine request 상태를 갱신합니다.
4. 테스트와 release build를 실행합니다.

## 수정 대상

우선 아래 파일들을 확인하세요.

```text
/Users/jkl/Projects/rust-survivors/docs/ENGINE_CHANGE_REQUESTS.md
/Users/jkl/Projects/rust-survivors/crates/game/src/survivor/sprites.rs
/Users/jkl/Projects/rust-survivors/crates/game/src/survivor/hud.rs
/Users/jkl/Projects/rust-survivors/crates/game/src/survivor/title_visual.rs
/Users/jkl/Projects/rust-survivors/crates/game/src/survivor/ui_icons.rs
```

검색 기준:

```sh
rg -n "survivor_texture_handle|textured_with_handle|DrawImage::textured_with_handle|handle\\.path\\(\\) ==|ENGINE_CHANGE_REQUESTS|Unify Image Texture Cache Keys" /Users/jkl/Projects/rust-survivors
```

## 구현 지침

- `handle.path() == requested_path`일 때만 핸들을 전달하던 조건부 workaround를 제거하세요.
- 이미지가 성공적으로 로드된 경우에는 `Sprite::textured_with_handle(path, Some(handle))` 또는 `DrawImage::textured_with_handle(..., path, Some(handle))`에 handle을 그대로 전달하세요.
- 핸들이 선택적으로 없을 수 있는 테스트/폴백 경로는 기존처럼 `None`을 허용해도 됩니다.
- `docs/ENGINE_CHANGE_REQUESTS.md`의 `2026-05-29 - Unify Image Texture Cache Keys` 항목은 엔진 수정 반영 완료 상태로 옮기거나 완료 메모를 추가하세요. 해당 문서의 기존 정책에 맞춰 open request에서 제거해도 됩니다.

## 검증 명령

`rust-survivors` 루트에서 실행하세요.

```sh
cargo test -p game --lib --locked -- --test-threads=1
cargo build -p game --bin survivor --release --locked
```

가능하면 전체 workspace 검증도 실행하세요.

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
```

## 시각 QA 체크리스트

아래 이미지들이 흰색 사각형으로 대체되지 않고 실제 PNG로 표시되는지 확인하세요.

- Title visual/title menu image assets
- HUD label and slot images
- Weapon/passive/powerup UI icons
- Level-up card images
- Shop and modal UI image elements

## 완료 기준

- `rg -n "handle\\.path\\(\\) ==" crates/game/src` 결과에 texture cache workaround가 남아 있지 않습니다.
- 기존에 workaround 때문에 생긴 "핸들이 있으면 쓰지 않고 path만 쓰는" 분기 없이도 이미지가 정상 표시됩니다.
- 게임 테스트와 release build가 통과합니다.
- `docs/ENGINE_CHANGE_REQUESTS.md`에서 해당 요청이 완료 처리되어 있습니다.
