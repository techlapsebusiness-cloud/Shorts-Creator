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
  M02)
    pnpm --dir apps/desktop test
    pnpm --dir tests/e2e test
    ;;
  M03)
    cargo test -p ai -p app
    pnpm --dir apps/desktop exec vitest run src/screens/SettingsScreen.test.tsx
    ;;
  M04)
    cargo test -p core -p ai -p store -p app
    pnpm --dir apps/desktop exec vitest run src/script src/screens/RecordScreen.test.tsx
    ;;
  M05)
    cargo test -p core -p media -p app
    pnpm --dir apps/desktop exec vitest run src/prompter src/screens/RecordScreen.test.tsx src/App.test.tsx
    ;;
  M06)
    cargo test -p media -p app
    pnpm --dir apps/desktop exec vitest run src/record src/screens/RecordScreen.test.tsx
    ;;
  *)
    echo "unknown module: ${module:-<missing>}" >&2
    echo "usage: just test-module M00" >&2
    exit 2
    ;;
esac
