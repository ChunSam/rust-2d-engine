# rust-survivors UV Migration Prompt

아래 프롬프트를 `rust-survivors` 게임 개발 작업 세션에 전달하세요.

---

## Prompt

`skeleton-engine`에서 기본 sprite quad UV 방향이 수정되었습니다.

엔진 변경 내용:

- 기본 quad top edge UV가 `v = 0.0`, bottom edge UV가 `v = 1.0`이 되었습니다.
- 이제 `Sprite`, `DrawImage`, `AtlasSprite`, `UvRect::FULL`, `UvRect::from_grid(...)`, `UvRect::from_pixels(...)`는 top-left-origin PNG를 정상 방향으로 렌더링합니다.
- 기존에 이미지가 뒤집혀 보여서 추가했던 `.flipped_y()` 보정은 엔진 업데이트 후 double-flip을 만들 수 있으므로 제거해야 합니다.

목표:

1. `rust-survivors`의 `skeleton-engine` 의존성을 UV 수정이 포함된 엔진 커밋으로 갱신합니다.
2. 게임 코드에서 수동 UV 보정용 `.flipped_y()`를 제거합니다.
3. 테스트, 릴리즈 빌드, 시각 QA를 수행합니다.

## 수정 대상

우선 아래 실제 코드 위치를 확인하고 `.flipped_y()`를 제거하세요.

```text
/Users/jkl/Projects/rust-survivors/crates/game/src/survivor/ui_icons.rs:630
/Users/jkl/Projects/rust-survivors/crates/game/src/survivor/ui_icons.rs:636
/Users/jkl/Projects/rust-survivors/crates/game/src/survivor/ui_icons.rs:643
/Users/jkl/Projects/rust-survivors/crates/game/src/survivor/ui_icons.rs:684
/Users/jkl/Projects/rust-survivors/crates/game/src/survivor/sprites.rs:130
/Users/jkl/Projects/rust-survivors/crates/game/src/survivor/sprites.rs:358
```

변경 예:

```rust
// Before
UvRect::from_grid(col, row, ICON_COLS, ICON_ROWS).flipped_y()

// After
UvRect::from_grid(col, row, ICON_COLS, ICON_ROWS)
```

```rust
// Before
UvRect::from_pixels(x, y, w, h, texture_w, texture_h).flipped_y()

// After
UvRect::from_pixels(x, y, w, h, texture_w, texture_h)
```

```rust
// Before
UvRect::FULL.flipped_y()

// After
UvRect::FULL
```

문서에 남아 있는 migration note/checklist도 현재 방향에 맞게 갱신하세요. 검색 기준:

```sh
rg -n "flipped_y\(" /Users/jkl/Projects/rust-survivors
```

코드에서 제거해야 하는 것은 엔진 UV 방향 보정용 `.flipped_y()`입니다. 실제 게임 의도상 특정 스프라이트를 세로 미러링하려는 용도의 `.flipped_y()`가 새로 발견되면 제거하지 말고 주석으로 의도를 남기세요.

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

엔진 업데이트와 `.flipped_y()` 제거 후 다음 항목이 원본 PNG 방향 그대로 보이는지 확인하세요.

- Title backdrop
- Title menu image buttons
- HUD slot frames
- Modal panels
- Level-up card frames
- Shop row frames
- Weapon icons
- Passive icons
- Powerup icons
- Actor animation frames
- Combat effect sprites

특히 atlas/grid 기반 아이콘과 full-image UI texture가 다시 뒤집히거나 double-flip되지 않는지 확인하세요.

## 완료 기준

- `rg -n "flipped_y\(" crates/game/src` 결과에 엔진 방향 보정용 호출이 남아 있지 않습니다.
- 게임 테스트와 release build가 통과합니다.
- 위 시각 QA 항목이 정상 방향으로 렌더링됩니다.
- migration 관련 문서가 새 엔진 동작에 맞게 갱신됩니다.
