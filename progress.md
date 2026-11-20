# Progress — Shorts Studio

Last updated: 2026-11-20 21:23 +0530 · Current module: M06 · Branch: m06-recording

| Module | Status | Tests (pass/total) | Last full gate | Notes |
|---|---|---|---|---|
| M00 Scaffold | ✅ Done | 14/14 | 2026-11-19 22:16 | doctor passed on Omarchy; Mac permissions MANUAL |
| M01 Project model | ✅ Done | 29/29 | 2026-11-20 19:54 | schema v1, SQLite library, crash-safe autosave |
| M02 UI shell | ✅ Done | 48/48 | 2026-11-20 20:14 | seven screens, snapshots at 1440×900, axe clean |
| M03 AI providers | ✅ Done | 75/75 | 2026-11-20 20:40 | Claude and OpenAI behind one trait; live calls return immediately unless LIVE_AI=1 |
| M04 Script generator | ✅ Done | 87/87 | 2026-11-20 20:52 | versioned prompts, editable beats, raw draft kept |
| M05 Prompter | ✅ Done | 105/105 | 2026-11-20 21:09 | separate window; magenta capture MANUAL |
| M06 Recording | ✅ Done | 111/111 | 2026-11-20 21:23 | lavfi take; real devices MANUAL |
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

**Next step:** completed. M03 is below.

## M03 — AI providers

- [x] `LlmProvider` for Claude (Anthropic Messages, streaming) and OpenAI (Responses, streaming)
- [x] Model lists come from each provider's models endpoint. No model id constants in source
- [x] Keys go to the OS keychain. Tests use an in-memory store
- [x] JSON validation retries once with the parser error, then fails
- [x] Log redaction strips the saved secret and `sk-` / `sk-ant-` tokens
- [x] Monthly cap USD 20, alert at 80 percent, block at 100 percent, recorded in SQLite
- [x] Routing table in `config/ai.toml`. Settings Test button and spend line

**Tests:** `just test` at 2026-11-20 20:40. Lint clean. Rust 49 passed (ai 22, app 6, core 11, media 5, store 5). Vitest 18 passed. Playwright 8 passed. axe reported no serious or critical violations. Settings snapshot updated. Fixture probe ok. `just test-module M03` passed. Two live tests compile and return immediately unless `LIVE_AI=1`. That return is the skip the prompt specifies. The gate does not call the network.

**Failing / blocked:** none.

**Manual checks for Akshay:** paste a real key, press Test, and confirm the key is absent from `project.json` and from logs. That check is not required for this done mark. The M00 doctor and native `just dev` checks are still open.

**Next step:** completed. M04 is below.

## M04 — Script generator

- [x] Versioned prompts in `prompts/script.md` and `prompts/regenerate-beat.md`
- [x] Fixture JSON validates as five beats: hook, concept, demo, fix, CTA, hook inside 3 s
- [x] Word count and duration at 140 words per minute
- [x] Regenerate one beat
- [x] Raw draft stays when a beat is edited or regenerated
- [x] An edited beat survives save and reload through the project writer

**Tests:** `just test` at 2026-11-20 20:52. Lint clean. Rust 58 passed (ai 26, app 8, core 13, media 5, store 6). Vitest 21 passed. Playwright 8 passed. axe reported no serious or critical violations. Record snapshot updated. Fixture probe ok. `just test-module M04` passed. Nothing skipped inside the gate.

The record screen keeps the draft and the beats in memory. The save-and-reload proof writes the edited beat with the crash-safe project writer, because the shell does not open a project folder yet.

**Failing / blocked:** none.

**Manual checks for Akshay:** generate one real script with `LIVE_AI=1` and read it. That check is not required for this done mark. The M00 doctor, native `just dev`, and M03 key-paste checks are still open.

**Next step:** completed. M05 is below.

## M05 — Teleprompter window

- [x] Separate always-on-top window, transparent, click-through except its controls
- [x] Scroll from `requestAnimationFrame` using elapsed time, after a 3 second countdown
- [x] Speed, text size, mirror, opacity, under the webcam, and distinct `CUE:` lines
- [x] Hotkeys: Space pauses, ↑↓ nudge, ⌘⇧P or Ctrl⇧P hides the text
- [x] Voice-follow decision: 2–3 second chunks, Hinglish fuzzy match, pause on silence, fixed speed when confidence is low
- [x] Linux refusal when a monitor capture includes the prompter, or a window/region overlaps it
- [x] macOS exclusion plan: match the window by pid and title, then sharing type none

**Tests:** `just test` at 2026-11-20 21:09. Lint clean. Rust 71 passed (ai 26, app 9, core 19, media 11, store 6). Vitest 25 passed. Playwright 9 passed. axe reported no serious or critical violations. Record snapshot updated for the opacity field. Fixture probe ok. `just test-module M05` passed. The 3 second magenta capture returns immediately with a MANUAL reason. On Linux that reason names the Wayland portal. On macOS it names the Mini permission check. The histogram checker itself runs in the gate.

The prompter keeps the fixed-speed clock until a transcript chunk arrives. `whisper-rs` stays in M07. A missing transcript takes the low-confidence path, which is fixed speed.

**Failing / blocked:** none inside the gate.

**Manual checks for Akshay:** on Omarchy, open the prompter, confirm the text ignores clicks and the controls still receive them, and read the refusal when Start recording uses a monitor capture. On the Mini, record 3 seconds with the magenta pattern and confirm it is absent. The M00 doctor, native `just dev`, M03 key paste, and M04 live script checks are still open.

**Next step:** completed. M06 is below.

## M06 — Recording engine

- [x] Fixture device pickers for display, camera, and microphone, plus a mic peak meter
- [x] `screen.mov`, `cam.mov`, `mic.wav` at 48 kHz, and `cursor.jsonl` from one start instant
- [x] Session clock: 3 s countdown, pause and resume
- [x] Disk estimate for 30 s at 40 000 kbps × 2 plus 384 kbps audio
- [x] Highlight clicks and show keystrokes stay in the cursor log
- [x] Hardware encoder preference, with the fixture forced to libx264
- [x] Desktop icons stay shown here; macOS hide is best effort and MANUAL

**Tests:** `just test` at 2026-11-20 21:23. Lint clean. Rust 76 passed (ai 26, app 10, core 19, media 15, store 6). Vitest 26 passed. Playwright 9 passed. Fixture probe ok. `just test-module M06` passed. The lavfi take's three durations agree within 50 ms, and the cursor log is strictly increasing. Nothing skipped inside the gate.

**Failing / blocked:** none.

**Manual checks for Akshay:** on the Mac, `just dev`, press Start recording, allow Screen Recording, Camera, and Microphone, then press Stop. The files land in `~/Movies/ShortsStudio/takes/<unix>/`. Confirm the prompter is absent from `screen.mov`. One real take on Omarchy is still open. Portal capture and ScreenCaptureKit stay MANUAL. Hiding desktop icons is a macOS best-effort check. The M00 doctor, native `just dev`, M03 key paste, M04 live script, and M05 magenta checks are still open.

**Next step:** wait for "continue", then branch `m07-transcription`.

## Session log

### 2026-11-19 21:53 +0530

Read the prompt. Checked current docs for `screencapturekit` window exclusion, `ashpd` cursor mode `METADATA`, `whisper-rs` 0.16 timestamps, and `ts-rs` vs `specta`. Wrote `plan.md`, `progress.md`, `DECISIONS.md`, and `docs/PROMPT.md`. No application code. Waiting for review of `plan.md`.

### 2026-11-19 22:17 +0530

"continue" accepted the recorded defaults (USD 20 cap, Mac checks stay MANUAL, Linux refusal rule unchanged). Scaffolded M00 from `create-tauri-app` 4.7.5 and reshaped it into the workspace. `just test`, `just test-module M00`, and `just doctor` passed on Omarchy. Whisper model absent, which doctor reports and does not fail. Installed `pnpm` 12.21.0 and `just` 1.59.0 on this machine.

### 2026-11-20 19:54 +0530

"continue" started M01 on `m01-project-model`. Schema v1, integer-millisecond clip ordering, v0 migration, SQLite library, and crash-safe autosave are in. `ts-rs` 12.0.1 writes the bindings during `cargo test`. zod 4.6.7 parses the committed v1 fixture as the generated `Project` type. Full gate passed on Omarchy.

### 2026-11-20 20:14 +0530

"continue" started M02 on `m02-ui-shell`. The seven screens are static on the slide theme, with Space Grotesk and JetBrains Mono committed under `apps/desktop/src/fonts/`. Playwright visits every screen at 1440×900, axe is clean, and the accent switch writes `--accent`. Full gate passed on Omarchy.

### 2026-11-20 20:40 +0530

"continue" started M03 on `m03-ai-providers`. Claude streams the Messages API and OpenAI streams the Responses API, both behind `LlmProvider`, with a mock that replays fixtures. Keys stay in the keychain, spend is capped at USD 20, and the settings screen lists models only after Test. Full gate passed on Omarchy.

### 2026-11-20 20:52 +0530

"continue" started M04 on `m04-script-generator`. Versioned prompts turn a topic into five editable beats. Screen cues and fact-check notes are `CUE:` and `FACT:` lines on the existing beat. The accepted model reply is stored as the raw draft, and editing or regenerating one beat leaves that draft in place. Full gate passed on Omarchy.

### 2026-11-20 21:09 +0530

"continue" started M05 on `m05-prompter`. The prompter is its own always-on-top window. Scroll follows elapsed time after a 3 second countdown, and a Hinglish transcript can move it to the spoken line. A monitor capture that includes the prompter is refused. The magenta recording stays MANUAL. Full gate passed on Omarchy.

### 2026-11-20 21:23 +0530

"push code and continue" started M06 on `m06-recording`. A fixture take writes `screen.mov`, `cam.mov`, `mic.wav`, and `cursor.jsonl` from one clock. The record screen names the fixture devices, shows the mic meter at rest, and estimates 301.4 MB for 30 seconds. Real cameras, the portal, and ScreenCaptureKit stay MANUAL. Full gate passed on Omarchy.
