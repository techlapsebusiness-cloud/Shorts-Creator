# Cursor Agent Prompt — Shorts Studio (local YouTube Shorts recorder, editor and 4K exporter)

> Paste this whole file into Cursor as the first message, in an empty repo named `shorts-studio`.
> Keep a copy in the repo as `docs/PROMPT.md` and re-read it at the start of every session.

---

## 0. Who you are and how we work

You are the lead engineer building **Shorts Studio**, a desktop app that runs entirely on my own machine. With it I can:

1. Generate a Short's script with **Claude or ChatGPT (via their APIs)**.
2. Read the script from a **teleprompter overlay that is never recorded**.
3. Record **screen + face cam + voice as three separate tracks**.
4. Edit it on a full **video and audio timeline**.
5. Get a **cover/thumbnail and an animated end card**, generated automatically.
6. **Export 4K vertical (2160×3840)** with smooth animation.
7. Get a complete **YouTube upload kit** (title, description, hashtags and every Studio setting).

I post one Short every day, so speed and a single-app flow matter more than features nobody uses.

### Working rules (follow strictly)

- **Work one module at a time**, in the order in §5. When a module is finished, its tests pass and `progress.md` is updated, **stop and wait for me to say "continue"**. Do not start the next module on your own.
- **First action, before any code:** create `plan.md`, `progress.md` and `DECISIONS.md` exactly as described in §6. Then stop and ask me to review `plan.md`.
- **Never mark a module done if any test fails.** Run the full gate in §7. If something can't be tested automatically (real camera, OS permission dialogs), write a short manual check in `progress.md` and mark that item `MANUAL`.
- **Verify every library API you use** against its current docs or source in the version you install. Don't trust memory: crate and npm APIs change. Record each non-obvious library choice and its version in `DECISIONS.md`.
- **Don't hardcode AI model names.** Fetch the available models from each provider's models endpoint and let me pick in Settings. Only the defaults go in config.
- **Make small commits**, one or more per module, with clear messages, on a branch per module (`m03-ai-providers` and so on). Do not push unless I ask.
- **Ask me** whenever a choice is irreversible or truly ambiguous. Otherwise decide, record it in `DECISIONS.md`, and continue.

---

## 1. Target machines

| Machine | OS | Role |
|---|---|---|
| Mac Mini M4, 24 GB | macOS (Apple Silicon) | **Primary**. Everything must work here. |
| x86 desktop | Omarchy Linux (Arch, Hyprland, Wayland, PipeWire) | **Secondary**. Recording, editing and export must work. Prompter exclusion follows the Linux rule in M05. |
| (later) | Ubuntu 24.04 GNOME, Windows | Don't block these, but don't build for them yet. |

---

## 2. Tech stack (use this unless a module's research proves something is broken; then record why in `DECISIONS.md`)

### App shell
- **Tauri 2** (Rust backend + system webview), multi-window (main window + prompter window).
- **Frontend:** React 19 + TypeScript (strict) + Vite, **pnpm**.
- **State:** Zustand, plus Immer for timeline edits. Undo/redo uses a command pattern on top.
- **Schemas:** `zod` on the frontend and `serde` on the backend for the same project schema. Generate TS types from Rust with `ts-rs` or `specta` (pick one and record it), so the two never drift apart.
- **Styling:** plain CSS modules or Tailwind (pick one) using the design tokens in §3. Bundle the fonts locally (Space Grotesk, JetBrains Mono); no Google Fonts CDN, because the app must work offline.

### Media
- **FFmpeg + ffprobe** (system install for local use: `brew install ffmpeg` / `pacman -S ffmpeg`). Called as child processes from Rust: no FFI bindings in v1.
- **Screen capture:**
  - macOS: **ScreenCaptureKit** via a Rust binding (for example the `screencapturekit` crate) or a small Swift sidecar if the binding can't do it. It must support **excluding specific windows** (the prompter) and **hiding the system cursor**.
  - Linux: **xdg-desktop-portal ScreenCast** (the `ashpd` crate) + **PipeWire** stream (`pipewire` crate or GStreamer `pipewiresrc`; pick one and record it). Request cursor mode "metadata" where the portal supports it.
- **Camera + mic:** FFmpeg input devices (`avfoundation` on macOS; `v4l2` + `pulse`/PipeWire on Linux), each written as its own file. If FFmpeg device capture is unreliable, fall back to `nokhwa` (camera) + `cpal` (mic) and record the reason.
- **Cursor telemetry:** record cursor position and clicks at about 120 Hz to `cursor.jsonl` during screen capture, so the editor can draw a **smoothed cursor** and do **auto-zoom**.
- **Transcription:** **whisper.cpp** through `whisper-rs` (Metal on macOS), with word-level timestamps. Model is configurable; the default is a multilingual model that handles Hindi/English code-switching. The model is downloaded once at first run into the app data folder.
- **Audio processing:** FFmpeg filters: `arnndn` or `afftdn` (noise), `equalizer`/`highpass`, `acompressor`, `deesser`, `sidechaincompress` (music ducking), two-pass `loudnorm` (I=-14, TP=-1.0).
- **Compositor / renderer:** **Rust + `skia-safe`**. FFmpeg processes decode screen and cam into raw frames. Rust composites each frame (9:16 crop and zoom, cam mask and border, captions, overlays, smoothed cursor, cover/end card) with Skia, then pipes raw RGBA into an FFmpeg encode process.
  - **Encoders:** `h264_videotoolbox` / `hevc_videotoolbox` on macOS; on Linux `h264_vaapi`/`hevc_vaapi` (or NVENC) if available, else `libx264`/`libx265`. Detect what's available at startup.
  - **Preview** uses the same scene description. Draw overlays in the webview with **CanvasKit** (Skia WASM) on top of HTML `<video>` elements at preview resolution, so preview and export look the same. If CanvasKit makes this too costly, keep the scene JSON shared and accept small differences, but record that.
- **Animation:** a small keyframe/easing library shared by Rust and TS: cubic-bezier easing, springs, and 60 fps sampling. It must be deterministic, so the same input always gives the same frames.

### AI
- **Claude:** Anthropic Messages API. **ChatGPT:** OpenAI API (Responses or Chat Completions; check current docs).
- Calls are made **from Rust** (`reqwest`) and streamed to the UI. API keys are stored in the **OS keychain** (`keyring` crate), never in project files or logs.
- Every response that has to be machine-readable (script beats, upload kit) asks for **JSON matching a schema**. Validate it; on failure, retry once with the validation error included.
- Track token usage per call into SQLite to show the spend shown on the settings screen and enforce the monthly cap.
- Both providers sit behind one `LlmProvider` trait, with a **mock provider** for tests that replays fixtures.

### Storage
- One folder per Short: `~/Movies/ShortsStudio/<NNN-slug>/` containing `project.json`, `media/` (raw takes), `cache/` (proxies, waveforms, transcripts) and `renders/`.
- **SQLite** (`rusqlite`, or `sqlx` with SQLite) in the app data folder for the library, topic queue, AI usage, upload history and the hashes used by the originality check.

### Testing
- **Rust:** `cargo test`, `insta` snapshots, `proptest` where useful (timeline maths).
- **Frontend:** Vitest + React Testing Library.
- **End-to-end:** Playwright against the Vite dev server, with Tauri IPC mocked (`@tauri-apps/api/mocks`), plus a small set of real Tauri smoke tests via `tauri-driver`/WebDriver where they work (Linux; on macOS mark as MANUAL if unsupported).
- **Media golden tests:** generate fixture media with FFmpeg (`testsrc2`, `sine`, a synthetic "speech" clip), render short clips, and assert with **ffprobe** (resolution, fps, codec, duration, audio sample rate) and **FFmpeg `ssim`/`psnr`** against golden frames (SSIM ≥ 0.98). Check loudness with `ebur128`.
- **Single command:** `just test` (or `pnpm test:all`) runs lint, typecheck, Rust tests, frontend tests, e2e and media goldens. `just test-module MXX` runs that module's tests.

### Local run
- `just dev`: Tauri dev with hot reload.
- `just doctor`: checks that ffmpeg/ffprobe exist (with versions), lists available hardware encoders, reports whether the whisper model is present, OS permissions (macOS screen/camera/mic), and on Linux the portal and PipeWire.
- `just build`: local release build (no signing or notarization needed for personal use).

---

## 3. Design (match this exactly; it's my YouTube slides theme)

**Tokens**

```
--bg:        #07090B   app background
--bar:       #090C0F   top bar / timeline background
--panel:     #0E1317   cards, inputs
--panel-2:   #0B0F12   inset surfaces, previews
--select:    #141A20   selected item background
--line:      #1F262E   borders
--line-2:    #2A323B   stronger borders
--text:      #EDF1F4
--text-mid:  #A9B4BF
--text-dim:  #8A96A3
--accent:    #FF7A3D   rust orange (user can switch: #4FE3C1 mint, #F5B26B amber, #8AB4FF blue)
--mint:      #4FE3C1   voice / success
--amber:     #F5B26B   music / warnings / safe-zone guides
--blue:      #8AB4FF   screen track / FX
--red:       #FF5A5A   recording, errors, cut marks
Fonts: Space Grotesk (UI), JetBrains Mono (labels, numbers, code, key hints)
Radius 8–16 px; borders 1 px; no gradients except subtle radial behind the preview; no emoji in the UI.
```

**Top bar on every screen:** `>>` chevron in the accent colour, then `sudoakshay/shorts`, project name, step tabs `01 Record · 02 Edit · 03 Audio · 04 Auto assets · 05 Export · 06 Upload`, an AI settings link, and a context action on the right.

**Screens**

1. **Record.**
   - Left: AI script panel. Model toggle (Claude / ChatGPT), topic input, language (Hinglish), length, chips (hook in 3 s, screen cues, fact-check notes, CTA), a Generate button, and an editable list of beats (HOOK / CONCEPT / DEMO / FIX / CTA with timestamps).
   - Centre: a "What you see" preview with the floating prompter (header `● PROMPTER · NOT RECORDED`) next to a "What gets recorded" 9:16 clean preview; layout presets; Start/Stop recording; Pause prompter.
   - Right: prompter settings (follows voice / fixed speed, speed, text size, mirror, position under webcam, show screen cues; "Hide from recording" is always on).
2. **Edit.**
   - Left: media bin and auto chapters from the transcript.
   - Centre: 9:16 preview (342×608 at fit) with the screen crop, captions, face cam and a Shorts safe-zone toggle.
   - Right: inspector (Layout / Captions / Motion). Cam position on a 3×3 grid, size S/M/L, shape circle/rounded/square, background removal, auto-zoom on cursor, word-by-word captions, caption language.
   - Bottom: timeline with tracks V3 FX, V2 CAM, V1 SCREEN, TXT, A1 VOICE, A2 MUSIC, A3 SFX. Tools: Select (V), Split (S), Trim (T), Ripple delete, Speed (R), Zoom (Z), Marker (M), Transition (⇧T), snapping, zoom slider.
3. **Audio.** Voice waveform with filler and silence marks, a "Remove N cuts" button, editable transcript (strike a word to cut it from video and audio), a one-click voice chain (noise removal, voice EQ, compressor, de-esser, loudness −14 LUFS) with bypass toggles, and a Voice/Music/SFX mixer with auto-ducking.
4. **Auto assets.** Three cover variants (A: hook + face cutout, B: error hook, C: before/after), a live looping end-card preview (chevron → "Next" card → Follow pill pulse → BUILD. LEARN. REPEAT.), motion styles (Rise, Glitch, Slide, Terminal type), and "add to every new Short".
5. **Export.** Resolution cards (4K 2160×3840 / 1440p / 1080p), frame rate, codec, bitrate and estimated size, **motion smoothness** options (easing presets, animate at 60 fps, motion blur on fast moves, smooth cursor path, frame blending on speed-ups), also-export (cover PNG, .srt, metadata .txt), Shorts pre-flight checks, and a render queue with progress.
6. **Upload kit.** Title, description and tags each with a copy button; a settings grid (audience, altered/synthetic content, category, video language, related video, custom thumbnail, comments plus a pinned question, remixing, visibility/schedule, length); files to upload; an originality check; an "after 24–48 h" reminder; Open YouTube Studio; optional direct upload via the YouTube API.
7. **Settings.** Claude and OpenAI key fields with a Test button, model pickers fed from the models endpoints, a "which AI does what" table, monthly spend cap, and **Daily Short mode** (topic queue plus an 8-step pipeline marking each step AI / YOU / AUTO).

---

## 4. Key product rules (these are the hard parts; tests must prove them)

1. **The prompter is never recorded.**
   - The prompter is its own Tauri window (always on top, transparent, click-through except its controls).
   - **macOS:** screen capture uses an `SCContentFilter` that **excludes the prompter window by window ID**. Also set the window's sharing type to none as a second safeguard, but don't rely on it alone.
   - **Linux (Wayland):** the portal can't exclude one window from a monitor capture. So the app must either (a) put the prompter on a **different monitor** from the one being captured, or (b) capture a **window or region** that doesn't overlap the prompter. If neither holds, **refuse to start recording** and show why.
   - **Automated proof:** an integration test records 3 s with the prompter showing a bright magenta test pattern, then checks that **no frame contains the magenta pattern** (pixel histogram check). On macOS this runs as a real test when screen-recording permission is granted; otherwise it is skipped with a clear message and listed as MANUAL.
2. **Three separate tracks** (screen, cam, mic) are recorded with a shared monotonic start clock. The offset between them is stored in `project.json` and checked after each take. As a fallback, a clap can be detected from the audio spike for manual re-sync.
3. **Non-destructive editing.** Raw takes are never modified; every edit lives in `project.json`.
4. **Preview matches export.** The same scene JSON feeds both; a golden test compares a preview-resolution render with a downscaled export frame (SSIM ≥ 0.95).
5. **Deterministic export.** The same project gives the same frames (golden hashes on a fixture project).
6. **Shorts safe zone.** Guides cover the right-hand button column and the bottom title area. When "safe zone" is on, the cam and captions snap inside the safe area.
7. **The YouTube policy pack is data, not code.** Upload-kit rules (max length, hashtag guidance, required fields, the disclosure explanation) live in `policy/youtube-shorts.json`, each with a `source` URL and `checked_on` date, so I can update them without code changes. The app shows the date of the pack it is using.
8. **Originality check.** Before export, compare the new Short with the last 10: topic and title similarity (embedding or trigram), script edit distance from the raw AI draft (warn if under 20% changed), and layout/template repeats. Show warnings; never block.
9. **Secrets.** API keys go only in the OS keychain. Logs redact anything that looks like a key. No telemetry.

---

## 5. Modules (build in this order; each needs passing tests before I say continue)

For each module: **Goal → Deliverables → Acceptance tests (automated) → Manual checks**.

### M00 — Repo scaffold and tooling
- Tauri 2 + React + TS + Vite + pnpm; Rust workspace with crates `core` (project model, timeline maths), `media` (ffmpeg, capture, render), `ai`, `store`, and `app` (Tauri commands).
- `justfile` with `dev`, `test`, `test-module`, `lint`, `doctor`, `build`.
- ESLint + Prettier, `cargo fmt` + `clippy -D warnings`, TS strict.
- `scripts/fixtures.sh` generates fixture media with FFmpeg into `tests/fixtures/` (gitignored, regenerated on demand).
- **Tests:** an empty app window boots in an e2e smoke test; `just doctor` passes on my Mac; CI-style `just test` passes.

### M01 — Project model and storage
- `project.json` schema v1 (tracks, clips, keyframes, captions, layout, audio chain, assets, export settings, AI metadata) shared via generated types; migration hook for future versions.
- SQLite library: projects, topics queue, ai_usage, uploads, originality hashes.
- New / open / save / autosave every 5 s; crash-safe writes (write to a temp file, then rename).
- **Tests:** schema round-trip (Rust ⇄ TS), property tests for clip ordering, autosave recovery after a simulated crash, a migration test from a v0 fixture.

### M02 — UI shell and design system
- Tokens, fonts, buttons, pills, toggles, cards, inputs, top bar with step tabs, routing between the 7 screens, accent switcher.
- Static versions of every screen from §3 with placeholder data.
- **Tests:** component tests for each primitive; Playwright visits every screen; axe accessibility check with no serious violations; visual snapshot of each screen at 1440×900.

### M03 — AI providers
- `LlmProvider` trait; Claude and OpenAI implementations with streaming; model listing; keychain storage; a Test button; usage and spend tracking; monthly cap with an 80% alert; a mock provider with fixtures; task routing table (script / upload kit / cover text / fact-check → provider).
- **Tests:** mock-provider unit tests; schema-validation retry; key redaction in logs; spend-cap enforcement; live tests behind `LIVE_AI=1` (skipped by default).

### M04 — Script generator
- Prompt templates (stored in `prompts/*.md`, versioned) for a Hinglish Short of a given length: hook within 3 s, beats with timestamps, `[CUE]` lines, CTA, optional fact-check notes.
- JSON output → editable beats UI; word count and estimated duration (configurable words per minute); a "regenerate this beat" action; the raw draft is stored for the originality check.
- **Tests:** schema validation of fixture outputs; duration estimator; edits are saved to the project.

### M05 — Teleprompter window
- Separate always-on-top window. Smooth scrolling at 60 fps using `requestAnimationFrame` with time-based movement (no CSS jank); speed, text size, mirror, opacity, position presets ("under webcam"); `[CUE]` lines styled differently. Hotkeys: Space pause, ↑↓ nudge, ⌘⇧P toggle. Starts with the recording countdown.
- **Voice-follow mode:** stream mic audio in 2–3 s chunks to whisper, fuzzy-match against the script and scroll to the matched line, pausing while I'm silent. Fall back to fixed speed if confidence is low.
- **Capture exclusion** exactly as in §4.1, including the Linux refusal rule.
- **Tests:** scroll-position maths (time → offset) unit tests; fuzzy matcher tests on Hinglish fixtures; the **magenta exclusion test** (§4.1); Linux overlap-detection unit tests.

### M06 — Recording engine
- Sources: display / window / region; camera; mic, each with device pickers and a live mic meter. Options: highlight clicks, show keystrokes (as overlay data, not baked in), hide desktop icons (macOS best effort).
- Writes `screen.mov` (hardware codec, high bitrate), `cam.mov`, `mic.wav` (48 kHz), and `cursor.jsonl`; shared start clock; 3 s countdown; pause/resume; disk-space estimate.
- Layout presets: Screen + cam ↘, Cam only, Screen only.
- **Tests:** with fixture "devices" (FFmpeg lavfi sources standing in for real devices), check that all three files exist and ffprobe durations match within 50 ms; cursor jsonl parser; MANUAL check with real devices on Mac and Linux.

### M07 — Transcription and analysis
- whisper-rs with word timestamps, run in the background after each take; filler detection (configurable list: "umm", "uh", "matlab", "basically", "you know"…); silences over 0.6 s; auto chapters from script beats matched to the transcript.
- **Tests:** a fixture clip with known words gives the expected words within ±150 ms; filler and silence detection on synthetic audio.

### M08 — Timeline editor
- Tracks from §3; clips with in/out points; split, trim, ripple delete, move, snap, speed (0.25–4×), markers, zoom keyframes, transitions (cut, fade, wipe, glitch); multi-select; undo/redo (100 steps); keyboard shortcuts as in the design; playhead scrubbing; waveform rendering from cached peaks; proxy files (720p) for smooth preview.
- **Tests:** property tests for split/trim/ripple invariants (no overlaps, total duration correct); undo/redo round-trip; Playwright test that does split → ripple delete → undo.

### M09 — Layout, cam and captions
- 9:16 screen crop that **follows the cursor** with smoothing (critically damped spring) plus manual zoom keyframes; cam position on the 3×3 grid, size S/M/L, shape, border in the accent colour, optional background removal (start with a simple portrait-segmentation model, or skip and record why).
- Safe-zone guides and snapping (§4.6).
- Word-by-word captions from the transcript: active word highlighted in the accent colour, max 2 lines, editable text, Hinglish.
- **Tests:** crop-follow maths (cursor path → crop rect stays inside the frame, never jumps more than N px per frame); safe-zone snap rules; caption line-breaking; golden frames for the 3 cam shapes.

### M10 — Audio processing
- Voice chain with bypass toggles and amounts; music ducking with sidechain (attack 80 ms, release 400 ms, −12 dB default); per-track gain and mute/solo; A/B preview; transcript-based cutting (cut a word → video and audio cut together, with 20 ms crossfades to avoid clicks).
- Final mix gets two-pass `loudnorm` to −14 LUFS integrated, −1.0 dBTP.
- **Tests:** loudness of the rendered fixture mix within ±0.5 LU of target; true peak ≤ −1.0 dBTP; ducking reduces music by about the set amount during voice; no clicks at cut points (sample-discontinuity check).

### M11 — Render engine
- Rust compositor (§2): decode → composite with Skia → encode. 2160×3840 at 30 or 60 fps; also 1440p/1080p; H.264 or HEVC; bitrate presets; AAC 48 kHz 384 kbps; `+faststart`.
- **Motion smoothness:** easing presets (smooth ease-in-out, snappy spring, linear); animate overlays at 60 fps; motion blur for fast moves (sub-frame accumulation, 4–8 samples); smoothed cursor path (from `cursor.jsonl`, drawn by the renderer); frame blending for speed-ups.
- Render queue with progress, ETA, cancel, and the output folder.
- **Tests:** ffprobe asserts resolution, fps, codec, duration ±1 frame; golden frame SSIM; deterministic hash test; motion-blur test (a moving square produces a blurred edge); performance budget logged (target: a 60 s 4K30 export in under 3 min on an M4; record actual numbers in `progress.md`).

### M12 — Auto assets
- Cover generator: three templates (Hook + face, Error hook, Before/after) filled from the hook line, the best face frame (largest face with an open expression; simple heuristic first) and the key code line; exported as a 1080×1920 PNG and optionally burned into the first 0.5 s.
- End card: 3 s animated sequence (chevron → Next card → Follow pill double-pulse → tagline) in 4 motion styles, auto-appended; "Next" title taken from the topic queue.
- **Tests:** PNG size and dimensions; text fits (no overflow) for long and short hooks; end-card golden frames at 0.0, 0.6, 1.3 and 2.0 s.

### M13 — Export screen and pre-flight
- UI from §3 wired to M11; pre-flight checks: 9:16, length ≤ the policy pack max, loudness, captions and cam inside the safe zone, music licence confirmation (manual tick), originality warnings (§4.8).
- **Tests:** each pre-flight rule against good and bad fixture projects.

### M14 — Upload kit
- AI-generated title (keyword first, length shown against the limit), description (hook line + link to the long video + 2–3 hashtags including #Shorts; never #viral/#fyp), tags, a pinned-comment question, and every Studio setting from the policy pack, each with a copy button; "Copy all" as plain text; also saved to `renders/upload_kit.txt`.
- Opens YouTube Studio in the default browser.
- Optional **direct upload via the YouTube Data API** (OAuth with a loopback redirect; `videos.insert` + `thumbnails.set` + captions). Show clearly that uploads from an unverified Google API project are stuck as private until Google audits the project. Keep it behind a toggle.
- History saved to SQLite for the originality check and the "after 24–48 h" reminder.
- **Tests:** schema validation; banned hashtags filtered; policy-pack loading and the date shown; OAuth flow with mocks.

### M15 — Daily Short mode and polish
- One button runs the pipeline: pick topic → generate script → **I review** → record with prompter → auto-edit (cut fillers and silences, captions, cursor zoom, ducking) → cover + end card → export 4K → upload kit. It pauses only at the YOU steps.
- Performance pass (startup < 2 s, timeline at 60 fps with 10 clips), error handling (FFmpeg missing, permission denied, disk full), first-run onboarding (permissions, whisper model download, API keys).
- **Tests:** an end-to-end pipeline on fixture media with the mock AI produces a valid 4K MP4, cover PNG, SRT and upload kit; time taken is logged.

---

## 6. Files you maintain

### `plan.md` (create first; update when scope changes)
- Overview, architecture diagram (Mermaid), crate and folder layout, data flow (record → project.json → preview/render), the module list from §5 with goals and acceptance tests, risks and mitigations (prompter exclusion on Wayland, A/V sync, 4K render time, whisper accuracy on Hinglish), and open questions for me.

### `progress.md` (update at the end of every work session and every module)

```markdown
# Progress — Shorts Studio

Last updated: <date time> · Current module: M05 · Branch: m05-prompter

| Module | Status | Tests (pass/total) | Last full gate | Notes |
|---|---|---|---|---|
| M00 Scaffold | ✅ Done | 6/6 | 2026-11-20 10:12 | |
| M01 Project model | ✅ Done | 24/24 | 2026-11-20 14:40 | |
| M02 UI shell | ✅ Done | 31/31 | … | |
| M03 AI providers | ✅ Done | 18/18 (+4 live skipped) | … | |
| M04 Script generator | ✅ Done | … | … | |
| M05 Prompter | 🟡 In progress | 9/11 | — | magenta test MANUAL on Linux |
| M06 Recording | ⬜ Not started | — | — | |
| … | | | | |

## M05 — Teleprompter window
- [x] Prompter window, always on top
- [x] Time-based smooth scroll
- [ ] Voice-follow mode
- [x] macOS capture exclusion (SCContentFilter)
- [ ] Linux overlap refusal
**Tests:** `just test-module M05` → 9 passed, 2 failing (listed below)
**Failing / blocked:** …
**Manual checks for Akshay:** 1) … 2) …
**Next step:** …
```

Status icons: ⬜ Not started · 🟡 In progress · 🔴 Blocked · ✅ Done (only when the full gate passes).

### `DECISIONS.md`
Dated entries: decision, options considered, why, library and version, links to docs.

---

## 7. Automatic test gate (run before marking any module done)

1. `just lint`: eslint, prettier check, `cargo fmt --check`, `cargo clippy -- -D warnings`, `tsc --noEmit`.
2. `just test-module MXX`: the new module's tests.
3. `just test`: **the full suite**, so earlier modules don't regress.
4. Media goldens, if the module touches media.
5. Paste the summary (counts, duration, anything skipped and why) into `progress.md`.
6. If anything fails: fix it, or mark the module 🔴 Blocked with the reason. **Never** mark it ✅.

Also add a git pre-commit hook that runs lint and the fast unit tests.

---

## 8. Definition of done for the whole app
On my Mac Mini M4 I can open the app, press **Start today's Short**, approve an AI script, record with the prompter (which is absent from the recording), and press Export. I get a **2160×3840 MP4** at −14 LUFS with smooth captions, cam, cursor zoom and end card, a cover PNG, an SRT, and a complete upload kit, in **under 15 minutes of my time** for a 60 s Short.

**Begin now:** read this prompt, create `plan.md`, `progress.md` and `DECISIONS.md`, then stop and ask me to review the plan.
