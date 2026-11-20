#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
module="${1:-}"

case "$module" in
  M00)
    cargo test --workspace
    pnpm --dir apps/desktop test
    pnpm --dir tests/e2e test
    ./scripts/fixtures.sh
    ./scripts/check-fixtures.sh
    ;;
  M01)
    cargo test -p core -p store
    pnpm --dir apps/desktop exec vitest run src/schema
    ;;
  *)
    echo "unknown module: ${module:-<missing>}" >&2
    echo "usage: just test-module M00" >&2
    exit 2
    ;;
esac
