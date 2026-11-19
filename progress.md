# Progress — Shorts Studio

Last updated: 2026-11-19 22:17 +0530 · Current module: M00 · Branch: m00-scaffold

| Module | Status | Tests (pass/total) | Last full gate | Notes |
|---|---|---|---|---|
| M00 Scaffold | ✅ Done | 14/14 | 2026-11-19 22:16 | doctor passed on Omarchy; Mac permissions MANUAL |
| M01 Project model | ⬜ Not started | — | — | |
| M02 UI shell | ⬜ Not started | — | — | |
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

**Next step:** wait for "continue", then branch `m01-project-model`.

## Session log

### 2026-11-19 21:53 +0530

Read the prompt. Checked current docs for `screencapturekit` window exclusion, `ashpd` cursor mode `METADATA`, `whisper-rs` 0.16 timestamps, and `ts-rs` vs `specta`. Wrote `plan.md`, `progress.md`, `DECISIONS.md`, and `docs/PROMPT.md`. No application code. Waiting for review of `plan.md`.

### 2026-11-19 22:17 +0530

"continue" accepted the recorded defaults (USD 20 cap, Mac checks stay MANUAL, Linux refusal rule unchanged). Scaffolded M00 from `create-tauri-app` 4.7.5 and reshaped it into the workspace. `just test`, `just test-module M00`, and `just doctor` passed on Omarchy. Whisper model absent, which doctor reports and does not fail. Installed `pnpm` 12.21.0 and `just` 1.59.0 on this machine.
