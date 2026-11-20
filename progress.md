# Progress — Shorts Studio

Last updated: 2026-11-20 20:14 +0530 · Current module: M02 · Branch: m02-ui-shell

| Module | Status | Tests (pass/total) | Last full gate | Notes |
|---|---|---|---|---|
| M00 Scaffold | ✅ Done | 14/14 | 2026-11-19 22:16 | doctor passed on Omarchy; Mac permissions MANUAL |
| M01 Project model | ✅ Done | 29/29 | 2026-11-20 19:54 | schema v1, SQLite library, crash-safe autosave |
| M02 UI shell | ✅ Done | 48/48 | 2026-11-20 20:14 | seven screens, snapshots at 1440×900, axe clean |
| M03 AI providers | ⬜ Not started | — | — | |
| M04 Script generator | ⬜ Not started | — | — | |
| M05 Prompter | ⬜ Not started | — | — | |
| M06 Recording | ⬜ Not started | — | — | |
| M07 Transcription | ⬜ Not started | — | — | |
| M08 Timeline | ⬜ Not started | — | — | |
| M09 Layout, cam, captions | ⬜ Not started | — | — | |
| M10 Audio | ⬜ Not started | — | — | |
| M11 Render | ⬜ Not started | — | — | |
| M12 Auto assets | ⬜ Not started | — | — | |
| M13 Export | ⬜ Not started | — | — | |
| M14 Upload kit | ⬜ Not started | — | — | |
| M15 Daily Short mode | ⬜ Not started | — | — | |

## M00 — Repo scaffold and tooling

- [x] Tauri 2 + React 19 + TypeScript + Vite + pnpm workspace
- [x] Rust crates `core`, `media`, `ai`, `store`, `app`, plus the Tauri binary
- [x] `justfile`: dev, test, test-module, lint, doctor, build, setup
- [x] ESLint, Prettier, rustfmt, clippy `-D warnings`, `tsc --noEmit`
- [x] `scripts/fixtures.sh` and ffprobe check
- [x] Playwright smoke with Tauri IPC mocked
- [x] Pre-commit hook in `scripts/hooks/pre-commit` (`just setup` copies it into `.git/hooks`)

**Tests:** `just test` at 2026-11-19 22:16. Lint clean. Rust 10 passed. Vitest 3 passed. Playwright 1 passed. Fixture probe ok (64×64 h264, 48 kHz pcm, ~1 s). Nothing skipped inside the gate.

**Failing / blocked:** none.

**Manual checks for Akshay:**

1. On the Mac Mini, `just doctor`. It should list ffmpeg and VideoToolbox, and print the Screen Recording / Camera / Microphone line. That line is MANUAL until the app is signed into TCC.
2. `just dev` on Omarchy and on the Mini, and confirm the native window shows Shorts Studio. The automated smoke boots the Vite page with IPC mocked, which is the M00 proof.

**Next step:** completed. M01 is below.

## M01 — Project model and storage

- [x] `project.json` schema v1: tracks, clips, keyframes, captions, layout, audio chain, assets, export settings, AI metadata, take sync
- [x] Migration hook, with a v0 fixture that opens as v1
- [x] SQLite library: projects, topics, ai_usage, uploads, originality hashes
- [x] New, open, save, and autosave 5 seconds after the first edit
- [x] Crash-safe write: temp file, sync, rename. A torn `project.json.tmp` does not replace the last good save
- [x] `ts-rs` bindings committed, zod schema checked against the v1 fixture

**Tests:** `just test` at 2026-11-20 19:54. Lint clean. Rust 24 passed (core 11, store 5, plus the M00 crates). Vitest 4 passed. Playwright 1 passed. Fixture probe ok. `just test-module M01` passed: core 11, store 5, schema vitest 1. Nothing skipped inside the gate.

**Failing / blocked:** none.

**Manual checks for Akshay:** none for M01. The M00 Mac doctor and native `just dev` checks are still open.

**Next step:** completed. M02 is below.

## M02 — UI shell and design system

- [x] Tokens and vendored Space Grotesk and JetBrains Mono (OFL)
- [x] Buttons, pills, toggles, cards, inputs
- [x] Top bar: chevron, `sudoakshay/shorts`, project name, six step tabs, AI settings, context action
- [x] Routes for all seven screens, with placeholder data
- [x] Accent switcher writes `--accent` (rust, mint, amber, blue)

**Tests:** `just test` at 2026-11-20 20:14. Lint clean. Rust 24 passed. Vitest 16 passed. Playwright 8 passed (boot plus one visit per screen). axe reported no serious or critical violations. Visual snapshots of each screen at 1440×900. Fixture probe ok. `just test-module M02` passed. Nothing skipped inside the gate.

**Failing / blocked:** none.

**Manual checks for Akshay:** glance at the shell on the Mini once a build exists there. Linux snapshots are the gate. The M00 doctor and native `just dev` checks are still open.

**Next step:** wait for "continue", then branch `m03-ai-providers`.

## Session log

### 2026-11-19 21:53 +0530

Read the prompt. Checked current docs for `screencapturekit` window exclusion, `ashpd` cursor mode `METADATA`, `whisper-rs` 0.16 timestamps, and `ts-rs` vs `specta`. Wrote `plan.md`, `progress.md`, `DECISIONS.md`, and `docs/PROMPT.md`. No application code. Waiting for review of `plan.md`.

### 2026-11-19 22:17 +0530

"continue" accepted the recorded defaults (USD 20 cap, Mac checks stay MANUAL, Linux refusal rule unchanged). Scaffolded M00 from `create-tauri-app` 4.7.5 and reshaped it into the workspace. `just test`, `just test-module M00`, and `just doctor` passed on Omarchy. Whisper model absent, which doctor reports and does not fail. Installed `pnpm` 12.21.0 and `just` 1.59.0 on this machine.

### 2026-11-20 19:54 +0530

"continue" started M01 on `m01-project-model`. Schema v1, integer-millisecond clip ordering, v0 migration, SQLite library, and crash-safe autosave are in. `ts-rs` 12.0.1 writes the bindings during `cargo test`. zod 4.6.7 parses the committed v1 fixture as the generated `Project` type. Full gate passed on Omarchy.

### 2026-11-20 20:14 +0530

"continue" started M02 on `m02-ui-shell`. The seven screens are static on the slide theme, with Space Grotesk and JetBrains Mono committed under `apps/desktop/src/fonts/`. Playwright visits every screen at 1440×900, axe is clean, and the accent switch writes `--accent`. Full gate passed on Omarchy.
