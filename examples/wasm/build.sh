#!/usr/bin/env bash
# WASM 빌드 스크립트
# 의존성: wasm-pack (cargo install wasm-pack)
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo ">>> WASM 빌드 시작..."
cd "$PROJECT_ROOT"
wasm-pack build --target web --out-dir examples/wasm/pkg

echo ">>> 완료. 브라우저에서 열기:"
echo "    python3 -m http.server 8080 --directory examples/wasm"
echo "    open http://localhost:8080"
