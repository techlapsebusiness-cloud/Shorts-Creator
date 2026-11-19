set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
    @just --list

# Tauri dev with Vite hot reload.
dev:
    pnpm --dir apps/desktop tauri dev

# ESLint, Prettier, rustfmt, clippy, and tsc.
lint:
    pnpm --dir apps/desktop lint
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    pnpm --dir apps/desktop exec tsc --noEmit -p tsconfig.json
    pnpm --dir tests/e2e exec tsc --noEmit -p tsconfig.json

# Tests for one module, for example `just test-module M00`.
test-module module:
    ./scripts/test-module.sh {{module}}

# Full gate: lint, Rust, Vitest, Playwright, and media fixtures.
test:
    just lint
    cargo test --workspace
    pnpm --dir apps/desktop test
    pnpm --dir tests/e2e test
    ./scripts/fixtures.sh
    ./scripts/check-fixtures.sh

# ffmpeg, encoders, whisper model, portal, and PipeWire.
doctor:
    cargo run -p media --bin doctor

# Local release build. No signing.
build:
    pnpm --dir apps/desktop tauri build

# Copy the pre-commit hook into .git/hooks.
setup:
    cp scripts/hooks/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit scripts/fixtures.sh scripts/check-fixtures.sh scripts/test-module.sh
