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
    cargo test -p core --lib script
    cargo test -p store --lib a_script_edit_survives_save_and_reload
    cargo test -p app --lib script
    ;;
  M05)
    cargo test -p core --lib prompter
    pnpm --dir apps/desktop exec vitest run src/prompter/scroll.test.ts
    ;;
  M06)
    cargo test -p media -p app
    pnpm --dir apps/desktop exec vitest run src/screens/RecordScreen.test.tsx
    ;;
  M07)
    cargo test -p core --lib transcript
    cargo test -p media --lib transcript
    ;;
  M08)
    cargo test -p core --lib edit
    pnpm --dir apps/desktop exec vitest run src/timeline/edit.test.ts
    pnpm --dir tests/e2e exec playwright test timeline.spec.ts
    ;;
  M09)
    cargo test -p core --lib layout
    pnpm --dir apps/desktop exec vitest run src/layout/frame.test.ts
    ;;
  M10)
    cargo test -p core --lib audio
    cargo test -p media --lib loudness
    ;;
  M11)
    cargo test -p core --lib render
    cargo test -p media --lib render
    pnpm --dir apps/desktop exec vitest run src/export/settings.test.ts
    ;;
  M12)
    cargo test -p core --lib assets
    cargo test -p media --lib cover
    pnpm --dir apps/desktop exec vitest run src/assets/card.test.ts
    ;;
  M13)
    cargo test -p core --lib preflight
    pnpm --dir apps/desktop exec vitest run src/export/preflight.test.ts
    ;;
  M14)
    cargo test -p core --lib upload
    cargo test -p store --lib a_kit_is_remembered_for_originality
    pnpm --dir apps/desktop exec vitest run src/upload/kit.test.ts
    ;;
  M15)
    cargo test -p core --lib daily
    cargo test -p media --lib daily
    pnpm --dir apps/desktop exec vitest run src/daily/steps.test.ts
    ;;
  *)
    echo "unknown module: ${module:-<missing>}" >&2
    echo "usage: just test-module M00" >&2
    exit 2
    ;;
esac
