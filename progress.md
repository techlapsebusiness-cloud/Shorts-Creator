# Progress — Shorts Studio

Last updated: 2026-11-22 14:40 +0530 · Current module: M15 · Branch: m15-daily-mode

| Module | Status | Tests (pass/total) | Last full gate | Notes |
|---|---|---|---|---|
| M00 Scaffold | ✅ Done | 14/14 | 2026-11-19 22:16 | doctor passed on Omarchy; Mac permissions MANUAL |
| M01 Project model | ✅ Done | 29/29 | 2026-11-20 19:54 | schema v1, SQLite library, crash-safe autosave |
| M02 UI shell | ✅ Done | 48/48 | 2026-11-20 20:14 | seven screens, snapshots at 1440×900, axe clean |
| M03 AI providers | ✅ Done | 75/75 | 2026-11-20 20:40 | Claude and OpenAI behind one trait; live calls return immediately unless LIVE_AI=1 |
| M04 Script generator | ✅ Done | 9/9 | 2026-11-21 22:53 | prompt v1, beat schema, 140 wpm estimate, raw draft kept. Live generation stays MANUAL |
| M05 Prompter | ✅ Done | 6/6 | 2026-11-21 23:05 | separate window, scroll, Hinglish match, Linux overlap refusal. Live magenta capture, Omarchy click-through, and mic chunks stay MANUAL |
| M06 Recording | ✅ Done | 27 passed, 1 ignored | 2026-11-21 23:41 | lavfi durations within 50 ms. Screen capture is capped at a 1920 long edge and a 2 GB take. Real Mac and Omarchy takes stay MANUAL |
| M07 Transcription | ✅ Done | 4/4 | 2026-11-21 22:48 | whisper-rs 0.16; fillers, silences, chapters. Hinglish accuracy and model download stay MANUAL |
| M08 Timeline | ✅ Done | 8/8 | 2026-11-21 22:49 | split, trim, ripple, speed, markers, transitions, waveform peaks, 100-step undo. Real-take scrub stays MANUAL |
| M09 Layout, cam, captions | ✅ Done | 7/7 | 2026-11-22 00:09 | crop spring, safe-zone snap, Hinglish line break, three cam masks. Background removal stays off. Real-frame safe zone stays MANUAL |
| M10 Audio | ✅ Done | 5/5 | 2026-11-22 00:15 | voice chain, −12 dB duck, 20 ms crossfade, two-pass loudnorm. Bypass each stage on a real take stays MANUAL |
| M11 Render | ✅ Done | 11/11 | 2026-11-22 01:35 | Skia composite, libx264 fixture, faststart, blur, queue. 4 frames of 4K took 0.615 s on this M1. The 60 s M4 budget and a watched export stay MANUAL |
| M12 Auto assets | ✅ Done | 5/5 | 2026-11-22 01:49 | three covers, open-face pick, 1080×1920 PNG, end-card beats. Reading the covers at phone size stays MANUAL |
| M13 Export | ✅ Done | 3/3 | 2026-11-22 01:49 | pre-flight on the M11 queue. Warnings do not block. A real-project pass stays MANUAL |
| M14 Upload kit | ✅ Done | 4/4 | 2026-11-22 02:40 | keyword-first title, banned tags stripped, policy date 2026-11-22, mock OAuth stays private. Paste into Studio, and a real OAuth client, stay MANUAL |
| M15 Daily Short mode | ✅ Done | 4/4 | 2026-11-22 02:40 | pauses on review and record. 4K fixture is 4 frames, 0.511 s on this M1. A timed real Short and a cold start stay MANUAL |

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

**Next step:** wait for "continue", then branch `m04-script-generator`.

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

### 2026-11-20 21:53 +0530

Started M06 on `m06-recording` (skipped ahead of M04/M05 at Akshay's request so recording can be tested). FFmpeg lavfi three-track session writes `screen.mov`, `cam.mov`, `mic.wav`, and `cursor.jsonl` with shared offsets. Record screen Start/Stop/Pause wired through Tauri. `just test-module M06` passed. Still open before ✅: live mic meter, click/keystroke overlay data, macOS icon hide, window/region pickers, Linux portal capture, ScreenCaptureKit exclusion (needs M05), real-device MANUAL takes, and the full `just test` gate.

### 2026-11-20 22:30 +0530

Started M07 on `m07-transcription`. whisper-rs 0.16 with Metal queues a transcript after each take. Fillers, 0.6 s silences, and beat chapters are in `core`. `just test-module M07` passed (fixture words within ±150 ms, synthetic fillers and silence). The ggml model is still absent, so a real Hinglish transcript is MANUAL. Full `just test` has not been run for this module.

### 2026-11-20 22:36 +0530

Started M08 on `m08-timeline`. Timeline edits use integer milliseconds. Split, trim, and ripple delete keep clips ordered and the duration sum exact under proptest. Undo stores 100 drafts. The Edit screen Split and Ripple delete buttons run the same operations through Immer, and Playwright does split → ripple delete → undo. Still open before ✅: move/snap on the timeline UI, markers, transitions, keyboard map beyond undo, waveform drawing, 720p proxy encode, and the full `just test` gate.

### 2026-11-20 22:45 +0530

Mac screen capture now uses ScreenCaptureKit 1.5.4 (`macos_15_0`) for `avfoundation` takes. The first display is written to `screen.mov` as H.264 MOV, and windows owned by this process are excluded. Camera and mic stay FFmpeg avfoundation, preferring the built-in devices over iPhone Continuity devices. The Record screen shows the take folder after stop. Lavfi remains the automated gate. An ignored probe wrote a 1440×900 H.264 `screen.mov` (~0.8 s). Still MANUAL: grant Screen Recording, Camera, and Microphone to the debug `shorts-studio` binary, then restart `just dev`.

### 2026-11-21 22:50 +0530

Closed the M07 and M08 gates. Transcript tests passed (fixture words within ±150 ms, fillers, silence, chapters). Timeline tests passed, including Playwright split → ripple delete → undo. The Edit screen now trims, sets speed, adds markers and zoom marks, cycles transitions, and draws voice peaks. A real take also writes `overlay.jsonl` for clicks and keys, a 720p proxy, and can capture one window or a region. Desktop icons can be hidden from the Record screen on macOS. M06 stays in progress because Linux portal screen capture is still not wired. Full `just test` has not been run on this machine.

### 2026-11-21 22:54 +0530

Completed M04. `prompts/script.md` is version 1. Fixture JSON validates as hook, concept, demo, fix, and cta, with the hook inside 3 seconds. Duration uses 140 words per minute. Replacing a beat keeps the raw draft, and that edit survives save and reload. The Record screen shows the word count, Generate, and Regenerate. Generate uses the fixture until a live model call is run with `LIVE_AI=1`.

### 2026-11-21 23:05 +0530

Completed M05. The prompter is a second always-on-top transparent window. Scroll uses elapsed time, Space pauses, arrows nudge, and ⌘⇧P hides the text. A Hinglish line match follows that line; a weak match stays on fixed speed; silence pauses. Linux refuses a monitor capture that shares the prompter's monitor, and a region that overlaps it. macOS marks the window content-protected and ScreenCaptureKit excludes a window titled Prompter. The 3 second magenta recording, click-through on Omarchy, and live mic chunks stay MANUAL.

### 2026-11-21 23:20 +0530

Completed M06. Lavfi still proves the three files and a monotonic cursor log. On Linux, screen capture opens the ScreenCast portal, asks for cursor metadata when the portal has it, and writes `screen.mov` with `gst-launch-1.0 pipewiresrc`. Camera and mic stay on v4l2 and pulse. The overlap refusal runs only while the prompter window is open. On this Mac, `cursor.jsonl` samples the real pointer. A real take on the Mini and one on Omarchy stay MANUAL. `just doctor` on Linux also expects `gst-launch-1.0`.

### 2026-11-21 23:41 +0530

Capped recording size. ScreenCaptureKit now writes at most a 1920 long edge, 30 fps, YUV 4:2:0. Pause closes that screen file and the next resume starts a new segment, joined on stop. A real take stops at 2 GB or when free space drops under 1 GB. The Record screen shows the live take size, and the disk estimate is the 20 Mbps screen plus 8 Mbps camera plus WAV budget (216 MB/min). The 720p proxy is capped at 4 Mbps and skipped when `screen.mov` is already over 1 GB. `cargo test -p media --lib`: 18 passed, 1 ignored. Record screen vitest passed. Lavfi stays on the fixed short duration, so the size watch does not run there.

### 2026-11-21 23:56 +0530

Stop no longer runs on the main thread, so ScreenCaptureKit can finish the file. Camera and mic are quit before the screen file is finalized, and a capture that ignores quit is killed. Closing the main window also closes the prompter and stops the take. The prompter has a Close button; click-through is off until that box is checked.

### 2026-11-22 00:09 +0530

Completed M09 on `m09-layout-captions`. The 9:16 crop follows the cursor with a critically damped spring and stays inside the source, moving at most 48 px per frame. Manual zoom keyframes blend. The camera sits on the 3×3 grid at S/M/L in circle, rounded, or square, and snaps inside the Shorts safe zone with the captions. A long Hinglish line breaks into two lines and the active word is marked. The three shape masks are distinct and stable. Background removal stays off. `cargo test -p core --lib layout` passed 4. Layout vitest passed 3. Checking the guides on a real frame stays MANUAL.

### 2026-11-22 00:15 +0530

Completed M10 on `m10-audio`. The voice chain is noise, highpass, compressor, and a treble de-esser, each with bypass and amount. Dry A/B preview is `anull`. Music ducks 12 dB over 80 ms and returns over 400 ms. Striking a word removes that span from picture and sound with a 20 ms crossfade, and the join stays under a 0.05 sample step. Solo silences the other tracks; mute is silence. The final mix is two-pass loudnorm at −14 LUFS and −1.0 dBTP. A 3 s sine measured −14.0 LUFS with true peak −10.3 dBFS. `cargo test -p core --lib audio` passed 4 (one of those is the transcript test, whose name contains "audio"). `cargo test -p media --lib loudness` passed 2. Hearing each stage on a real take stays MANUAL.

### 2026-11-22 01:35 +0530

Completed M11 on `m11-render`. Ffmpeg decodes a plate, Skia draws the square, and the RGBA frames are piped to ffmpeg with AAC at 48 kHz and 384 kbps and `+faststart`. Sizes are 2160×3840, 1440×2560, and 1080×1920. Easing is linear, smoothstep, or an overshooting spring. Overlays can be sampled at 60 fps. Motion blur averages 6 Skia subframes. A speed-up blends the neighbouring frames, and the cursor path is smoothed. The export screen can start, poll, and cancel a short fixture into `renders/`. A repeated render hashed the same first frame. The sharp square matched the fixture at SSIM ≥ 0.98. The blurred edge is gray. `cargo test -p core --lib render` passed 6. `cargo test -p media --lib render` passed 4. Export vitest passed 1. Four frames at 2160×3840 took 0.615 s on this M1. A 60 s 4K30 timing on an M4, and watching the file for dropped motion, stay MANUAL.

### 2026-11-22 01:49 +0530

Completed M12 and M13 on `m13-export`. Covers are hook + face, error hook, and before/after. The face pick is the largest box with an open mouth, and the key line is the densest code line. A short hook and a long one both stay inside 1080×1920. The cover PNG is that size. The end card is chevron, Next, a double-pulse Follow, then `BUILD. LEARN. REPEAT.`, at 0.0, 0.6, 1.3, and 2.0 s, in Rise, Glitch, Slide, or Terminal. The next title comes from the topic queue. Burn-in is the first 0.5 s when that toggle is on. Pre-flight checks 9:16, the 60 s pack limit, loudness, the safe zone, the music-licence tick, and originality against the last 10. A near copy warns. A distinct short stays quiet. Nothing in that list blocks Render fixture. `cargo test -p core --lib assets` passed 3. `cargo test -p media --lib cover` passed 1. `cargo test -p core --lib preflight` passed 2. Asset and preflight vitest passed 1 each. Reading the three covers at phone size, and running pre-flight on a real project, stay MANUAL.

### 2026-11-22 02:40 +0530

Completed M14 and M15 on `m15-daily-mode`. The upload kit puts the keyword first, keeps the title inside 100 characters, and writes a hook, the long-video link, and up to three hashtags that always include `#Shorts`. `#viral` and `#fyp` are dropped. Studio fields come from the policy pack dated 2026-11-22. Copy all is plain text. The title is stored for originality, and the upload row stays private. Direct upload needs a non-empty OAuth client id; the mock token and `videos.insert` body stay private, and the screen says an unverified Google project can only upload as private. Daily mode labels each step AI, YOU, or AUTO and pauses only for review and recording. Missing ffmpeg, a denied permission, and a full disk each have a named fault. First run asks for permissions, the whisper model, and API keys. The fixture day writes a 2160×3840 h264 file, a 1080×1920 cover, an SRT, and `upload_kit.txt`. Four frames took 0.511 s on this M1. `cargo test -p core --lib upload` passed 2. `cargo test -p store --lib a_kit_is_remembered_for_originality` passed 1. `cargo test -p core --lib daily` passed 2. `cargo test -p media --lib daily` passed 1. Upload and daily vitest passed 1 each. Pasting a kit into Studio, creating an OAuth client, one timed real Short, and a cold-start measurement stay MANUAL.

### 2026-11-22 14:40 +0530

Connected a recorded take to the editor and to export. Screen, Camera, and Microphone sit in the record column, and the page scrolls. `list_takes` lists take folders. Open in editor writes `~/Movies/ShortsStudio/<id>-take/project.json` with the screen, camera, and mic clips, and reopening keeps later edits. The editor plays those files, and split, trim, ripple, and speed change the clip in and out points. Export of an open project writes `renders/short.mp4` from the trimmed screen, with the camera overlaid and the mic loudnormed. The default size is 1080×1920. A one-second fixture exported as 1080×1920 h264 with audio. The gray plate remains only when no project is open. Playwright snapshots were not refreshed: the sandbox has no Chromium binary. A real device take on the Mini stays MANUAL.
