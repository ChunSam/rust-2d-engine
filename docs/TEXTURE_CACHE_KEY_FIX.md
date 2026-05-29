# Texture Cache Key Fix

작성일: 2026-05-29

## 배경

`rust-survivors`에서 상대 경로로 이미지를 로드한 뒤 `Handle<ImageAsset>` 기반 렌더링을 사용하면 이미지가 흰색 사각형으로 보일 수 있었다.

원인은 키 정책 불일치였다.

- `App::load_image("relative/path.png")`는 GPU 업로드 대기열에 원래 상대 경로 문자열을 넣었다.
- 네이티브 `AssetServer::load_image(...)`는 존재하는 파일을 canonical absolute path로 정규화해 `Handle<ImageAsset>::path()`에 저장했다.
- 스프라이트/UI/아틀라스 렌더링은 핸들이 있으면 `handle.path()`를 텍스처 키로 우선 사용했다.
- `SpriteRenderer::texture_cache`는 상대 경로 키만 가지고 있어 canonical handle 키 조회가 실패했고, white fallback texture가 사용됐다.

## 엔진 변경

- `asset_key(...)`를 crate 내부에서 재사용 가능하게 열어 렌더러도 `AssetServer`와 같은 canonical key 정책을 따른다.
- `SpriteRenderer::load_texture(...)`는 파일 텍스처를 한 번만 GPU에 올리고, 같은 `Arc<Texture>`를 아래 alias 키들에 등록한다.
  - 원래 요청 경로 문자열
  - 파일이 존재할 때 canonical absolute path
- 이미 어느 alias로 텍스처가 캐시되어 있으면, 새로 로드하지 않고 누락된 alias만 backfill한다.
- `SpriteRenderer::reload_texture(...)`는 변경 파일과 같은 canonical key로 매칭되는 기존 alias들을 모두 새 텍스처로 갱신한다.
- `rt_cache` render target 키와 비동기 이미지 완료 경로(`load_texture_from_image`)는 기존 정책을 유지한다.
- `App::load_image()` rustdoc의 GPU 업로드 시점 설명을 실제 동작에 맞게 정리했다.

## 영향

공개 API 변경은 없다.

아래 경로들이 같은 이미지 파일을 가리키는 경우 더 이상 키 불일치로 white fallback에 빠지지 않는다.

- `Sprite::textured("relative/path.png")`
- `Sprite::textured_with_handle("relative/path.png", Some(handle))`
- `DrawImage::textured_with_handle("relative/path.png", Some(handle))`
- `AtlasSprite`의 `TextureAtlas::texture_path()`

WASM과 존재하지 않는 경로는 기존처럼 입력 문자열을 키로 유지한다.

## 테스트

추가/확인한 테스트:

- existing relative path가 원 요청 경로와 canonical 경로 alias를 모두 만든다.
- missing/synthetic texture key는 원 문자열 하나만 유지한다.
- `DrawImage::with_handle(...)` / `DrawImage::textured_with_handle(...)`가 handle key를 우선 사용한다.
- `TextureAtlas::texture_path()`가 image handle path를 반환한다.

검증:

```text
cargo fmt
cargo test
```

결과:

```text
unit tests: 225 passed
doc tests: 31 passed, 19 ignored
```

## 게임 개발팀 전달사항

`rust-survivors` 쪽 임시 workaround를 제거하고 엔진 의존성을 이번 수정이 포함된 커밋으로 갱신해야 한다.

전달용 작업 프롬프트는 `docs/RUST_SURVIVORS_TEXTURE_CACHE_KEY_PROMPT.md`에 정리했다.
