#!/usr/bin/env bash
# WASM 릴리즈 번들 빌드 스크립트
#
# 사용법:
#   ./scripts/build_wasm.sh              # release 프로필
#   ./scripts/build_wasm.sh --dev        # dev 프로필 (빠른 반복)
#
# 출력: dist/
#   engine_bg.wasm   — 최적화된 WASM 모듈
#   engine.js        — wasm-bindgen JS 글루 코드
#   index.html       — 기본 HTML 엔트리 포인트
#
# 요구 사항: wasm-bindgen-cli, wasm-pack (선택)
#   cargo install wasm-bindgen-cli

set -euo pipefail

PROFILE="release-wasm"
TARGET_DIR="target/wasm32-unknown-unknown"

if [[ "${1:-}" == "--dev" ]]; then
    PROFILE="dev"
    echo "[build_wasm] dev 모드"
else
    echo "[build_wasm] release-wasm 모드"
fi

DIST="dist"
mkdir -p "$DIST"

# ── 1. 컴파일 ─────────────────────────────────────────────────────────────────
if [[ "$PROFILE" == "dev" ]]; then
    cargo build --target wasm32-unknown-unknown
    WASM_FILE="$TARGET_DIR/debug/engine.wasm"
else
    cargo build --target wasm32-unknown-unknown --profile release-wasm
    WASM_FILE="$TARGET_DIR/release-wasm/engine.wasm"
fi

# ── 2. wasm-bindgen 바인딩 생성 ────────────────────────────────────────────────
wasm-bindgen \
    --target web \
    --out-dir "$DIST" \
    --no-typescript \
    "$WASM_FILE"

echo "[build_wasm] wasm-bindgen 완료 → $DIST/"

# ── 3. 기본 index.html 생성 (없으면) ──────────────────────────────────────────
INDEX="$DIST/index.html"
if [[ ! -f "$INDEX" ]]; then
cat > "$INDEX" <<'HTML'
<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>rust-2d-engine</title>
  <style>
    body { margin: 0; background: #0a0a0f; display: flex; justify-content: center; align-items: center; height: 100vh; }
    canvas { display: block; }
  </style>
</head>
<body>
  <canvas id="canvas"></canvas>
  <script type="module">
    import init, { run_demo } from './engine.js';
    await init();
    run_demo();
  </script>
</body>
</html>
HTML
    echo "[build_wasm] index.html 생성 완료"
fi

echo "[build_wasm] 빌드 완료 → $DIST/"
echo "  로컬 서버 실행: python3 -m http.server --directory $DIST 8080"
