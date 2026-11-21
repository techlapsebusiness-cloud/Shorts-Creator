# Decisions — Shorts Studio

Dated choices for anything that is easy to regret later. Exact crate and npm versions are filled in when that dependency is added, after reading the docs for the installed version. Entries above a pin say "unpinned" on purpose.

Observed on 2026-11-19, not yet depended on: `screencapturekit` 1.5.2 docs show `with_excluding_windows`; `whisper-rs` 0.16.0 on crates.io exposes Metal and token timestamps; `specta` / `tauri-specta` docs still show the 2.0.0-rc line; `ashpd` `CursorMode::METADATA` matches portal cursor mode 4.

---

### 2026-11-19 — TypeScript types from the Rust schema

**Decision.** `ts-rs` derives on the `core` schema. Export runs with the Rust tests into `apps/desktop/src/bindings/`. Those files are committed. `zod` schemas validate project JSON and AI JSON at runtime. A round-trip test fails if the generated types and the serde schema disagree.

**Options.** `ts-rs`. `specta` with `tauri-specta`.

**Why.** The schema lives in `core`, which must compile without Tauri. `ts-rs` is a derive on those structs and writes plain `.ts` files during `cargo test`, so drift shows up in the test gate. `specta` is the stronger fit for generated command clients, and its own docs still describe a 2.0 release candidate with exact version pins. Command wrappers stay small and handwritten until a stable `tauri-specta` is worth a migration.

**Library and version.** `ts-rs` 12.0.1. See the 2026-11-20 pin.

**Links.** <https://github.com/Aleph-Alpha/ts-rs> · <https://specta.dev/docs/tauri-specta> · <https://docs.rs/specta/latest/specta/>

---

### 2026-11-19 — Styling

**Decision.** CSS modules. Design tokens from the prompt are CSS variables on `:root`. Space Grotesk and JetBrains Mono are vendored (OFL). No font CDN.

**Options.** CSS modules. Tailwind.

**Why.** The UI is a fixed token set and seven screens. Component CSS sits next to the component, and the visual snapshots in M02 diff real rules. Accent switching writes `--accent`.

**Library and version.** CSS modules. Font files are pinned in the 2026-11-20 entry.

---

### 2026-11-19 — SQLite

**Decision.** `rusqlite` with the `bundled` feature. SQL migrations are versioned files in `crates/store`.

**Options.** `rusqlite`. `sqlx` with SQLite.

**Why.** The database is a local library: projects, topic queue, usage, uploads, originality hashes. Calls are short and can run on a blocking thread inside Tauri commands. Bundled SQLite means Omarchy and the Mini share one SQLite build.

**Library and version.** `rusqlite` 0.43.0 with `bundled`. See the 2026-11-20 pin.

**Links.** <https://docs.rs/rusqlite>

---

### 2026-11-19 — Linux screen capture

**Decision.** `ashpd` for the ScreenCast portal, then the `pipewire` crate to read frames. Request `CursorMode::METADATA` when `AvailableCursorModes` includes it, else `HIDDEN`. App-sampled `cursor.jsonl` at about 120 Hz is the editor's cursor source either way.

**Options.** `pipewire` crate. GStreamer `pipewiresrc`.

**Why.** Decode, composite, and encode already go through ffmpeg and Skia. A second media graph is extra surface area. `ashpd`'s docs show a PipeWire example and a GStreamer example; PipeWire is the first path. If DMA-BUF negotiation cannot produce frames ffmpeg can write, M06 switches to `pipewiresrc` and adds an entry here.

**Library and version.** `ashpd`, `pipewire`, unpinned. Pin in M06 after reading the installed docs.

**Links.** <https://docs.rs/ashpd/latest/ashpd/desktop/screencast/index.html> · <https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.ScreenCast.html>

---

### 2026-11-19 — macOS screen capture

**Decision.** The `screencapturekit` crate. Exclude the prompter with `SCContentFilter::create().with_display(...).with_excluding_windows(...)`. Match the Tauri window to `SCWindow` by owner pid and title. Also set the window sharing type to none. A Swift sidecar is only built if the crate cannot address that window.

**Options.** `screencapturekit`. A Swift helper from day one.

**Why.** Current crate docs already expose excluding windows by `SCWindow`. One language in-process is the smaller path. The magenta test is the proof, not the sharing-type flag.

**Library and version.** `screencapturekit`, unpinned. Latest docs observed: 1.5.2. Pin in M05. This crate is macOS-only and is not compiled on Omarchy.

**Links.** <https://docs.rs/screencapturekit/latest/screencapturekit/stream/content_filter/struct.SCContentFilterBuilder.html>

---

### 2026-11-19 — Motion library shared by Rust and TypeScript

**Decision.** Implement cubic-bezier, critically damped spring, and 60 fps sampling in `crates/core`. Port the same functions to `packages/motion`. Check in `tests/vectors/motion.json`. Both test suites load it and must agree sample for sample.

**Options.** One Rust library compiled to WASM for the UI. Two ports locked by vectors.

**Why.** Export determinism is a Rust property: the renderer samples `core`. The preview needs the same samples without a WASM toolchain in M00. The vector file is the contract. A WASM build of `core` can replace the port later if the two diverge.

**Library and version.** No third-party easing crate.

---

### 2026-11-19 — OpenAI wire API

**Decision.** Deferred to the start of M03. Read the current OpenAI docs, then pick the streaming endpoint that accepts a JSON schema. Preference: Responses API. Fallback: Chat Completions. Model ids are never constants in source; defaults live in config, and the settings screen lists whatever the models endpoint returns.

**Options.** Responses API. Chat Completions.

**Why.** The prompt says to check current docs at implementation time. Choosing the URL now would skip that check.

**Library and version.** Pinned in the 2026-11-20 M03 entry below.

---

### 2026-11-19 — Preview compositor

**Decision.** Scene JSON is the shared description. Preview plays HTML video and draws overlays with CanvasKit at preview resolution. Export draws the same scene with `skia-safe`. If CanvasKit init cost or preview frame rate is unacceptable, stop using it, keep the scene JSON, and write the visual difference here. The SSIM ≥ 0.95 preview-vs-export test still applies to a preview-resolution Rust render.

**Options.** CanvasKit in the webview. Rust-only render for both, with the webview showing a video of that render.

**Why.** The prompt asks for CanvasKit first so overlays match Skia. The scene JSON keeps that promise even if the wasm path is dropped.

**Library and version.** `skia-safe`, unpinned, pin in M11. CanvasKit build pin in M09 or M11, whichever lands the preview first.

---

### 2026-11-19 — Whisper default

**Decision.** Model file is a setting. Initial default: `ggml-large-v3-turbo` (multilingual), downloaded once into the app data directory. Enable token timestamps. Use `set_split_on_word` only if the installed `whisper-rs` docs still mark it as the word-timestamp path; otherwise derive words from token times. Metal feature on macOS. On Linux, pick Vulkan if the installed crate builds it here, else CPU, and record which.

**Options.** `large-v3-turbo`. `medium`. `small`.

**Why.** Daily scripts are Hindi/English code-switching, so an `.en` model is ineligible. Turbo is the quality/speed default that still fits a 24 GB Mini. A smaller file stays one settings change away.

**Library and version.** `whisper-rs` 0.16.0 observed on crates.io (updated 2026-03-12). Pin the exact version in M07, including the `metal` feature on macOS.

**Links.** <https://docs.rs/whisper-rs/latest/whisper_rs/> · <https://crates.io/crates/whisper-rs>

---

### 2026-11-19 — Defaults that are settings

**Decision.** Monthly spend cap USD 20, alert at 80%. Hinglish duration estimate 140 words per minute. YouTube Data API upload hidden behind a toggle, default off. Project directories `~/Movies/ShortsStudio/<NNN-slug>/` with `NNN` zero-padded to 3. Package and binary name `shorts-studio`. This repo directory is unchanged.

**Why.** Each value is reversible from Settings or from config. They need a default so M03, M04, and M14 have a fixture.

---

### 2026-11-19 — Background removal

**Decision.** M09 starts from a simple portrait-segmentation model only if it can land inside that module. If the model download, runtime, or quality slips, ship shapes with no background removal and record the reason here.

**Why.** Camera shape, safe zone, and captions are the daily path. A matting model is optional in the prompt.

---

### 2026-11-19 — Git hooks

**Decision.** `scripts/hooks/pre-commit` runs lint and fast unit tests. `just setup` copies it to `.git/hooks/pre-commit`.

**Why.** The hook has to exist without changing git config.

---

### 2026-11-19 — M00 toolchain pins

**Decision.** Pin the scaffold to the versions installed and compiled on Omarchy. `create-tauri-app` 4.7.5 generated the React template; the workspace layout replaces that single package.

**Versions.**

| Tool | Version |
|---|---|
| Rust | 1.99.1 (edition 2024, `rust-toolchain.toml`) |
| just | 1.59.0 |
| pnpm | 12.21.0 |
| Tauri | 2.12.7 |
| tauri-build | 2.7.6 |
| tauri-plugin-opener | 2.7.3 |
| @tauri-apps/api, @tauri-apps/cli | 2.12.5 |
| serde | 1.0.229 |
| React, React DOM | 19.3.1 |
| Vite | 8.3.8 |
| @vitejs/plugin-react | 6.3.0 |
| TypeScript | 6.0.3 |
| Vitest | 5.1.1 |
| Playwright | 1.65.1 |
| ESLint | 10.15.0 |
| typescript-eslint | 8.73.0 |
| Prettier | 3.10.1 |
| FFmpeg on this machine | n9.0.2 |

**Links.** <https://v2.tauri.app/start/frontend/vite/> · <https://v2.tauri.app/develop/tests/mocking/> · <https://docs.rs/tauri/2.12.7>

---

### 2026-11-19 — Public Tauri commands live in a submodule

**Decision.** `#[tauri::command]` functions in `crates/app` are `pub` items inside `commands`, re-exported from the crate root. `generate_handler![app::health]` stays in `apps/desktop/src-tauri`.

**Options.** Public command in `lib.rs`. Private command only inside the Tauri crate.

**Why.** tauri-macros 2.7.6 emits `#[macro_export]` plus `pub use` of `__cmd__<name>` for a public command. On Rust 1.99 that pair is defined twice when the function sits in the crate root (`E0255`). A submodule keeps the export, which is what another crate's `generate_handler!` calls.

**Library and version.** tauri-macros 2.7.6, pulled by Tauri 2.12.7.

---

### 2026-11-19 — Where Mac-only proof runs

**Decision.** Develop and run the automatic gate on Omarchy. macOS ScreenCaptureKit, VideoToolbox, permission probes, and the magenta capture are compiled with `cfg(target_os = "macos")` and marked MANUAL in `progress.md` until they run on the Mini.

**Why.** This checkout is the secondary machine. The prompt's primary proof stays on the M4, and Linux still has to record, edit, and export.

---

### 2026-11-20 — Project schema v1 and the library

**Decision.** `project.json` is schema version 1. Times in the file are seconds (`f64`). Clip ordering uses integer milliseconds on `TimelineSpan`, separate from a clip's duration, so the property tests stay exact. The document has no nulls: an empty string means unset. Clip bodies are an internally tagged `kind` enum. `#[serde(rename_all_fields = "camelCase")]` renames variant fields, so `source_in` is `sourceIn`. Export defaults are 2160×3840, 30 fps, h264, 40000 kbps video, and 384 kbps audio.

`#[ts(export)]` on `Project` writes `apps/desktop/src/bindings/` during `cargo test`. `.cargo/config.toml` sets `TS_RS_EXPORT_DIR` there and `TS_RS_LARGE_INT=number`. Those generated files are committed and ignored by ESLint and Prettier. `zod` 4 parses the same JSON, and the schema is annotated `z.ZodType<Project>` so a field rename fails `tsc`.

SQLite schema lives in `crates/store/migrations/001_library.sql`. Opening the library applies it and records version 1 in `schema_migrations`. Autosave waits 5 seconds from the first dirty mark. A later edit does not reset that timer. The save writes `project.json.tmp` in the same directory, `sync_all`, then renames over `project.json`. `load_project` reads only `project.json`.

**Versions.**

| Crate / package | Version |
|---|---|
| ts-rs | 12.0.1 |
| proptest | 1.12.0 |
| rusqlite | 0.43.0 (`bundled`; default features compiled on Omarchy) |
| zod | 4.6.7 |

**Why.** `rename_all` on an enum renames variants and leaves struct-variant fields alone. `rename_all_fields` is what serde 1.0.229 and ts-rs 12 both apply to those fields, which keeps the fixture, the generated types, and zod on one spelling. Bundled SQLite compiled here with its default features, including the modern SQLite build, so those features stay on.

**Links.** <https://docs.rs/ts-rs/12.0.1/ts_rs/> · <https://docs.rs/rusqlite/0.43.0/rusqlite/> · <https://zod.dev/>

---

### 2026-11-20 — Shell fonts, accent, and screen checks

**Decision.** Space Grotesk and JetBrains Mono are committed as woff2 files under `apps/desktop/src/fonts/`, with the SIL Open Font License beside each family. The shell does not load a font CDN. Switching accent writes `--accent` on the document element. The seven screens are static placeholder UI. Route and accent live in React state. Visual snapshots are Playwright captures at 1440×900, and axe must report no serious or critical violations.

**Options.** `@fontsource` imported from `node_modules`. Fonts committed in the repo. Zustand for the route and accent.

**Why.** A committed file still renders when the machine is offline and the install cache is empty. `@fontsource` 5.3.0 is only the source of the woff2 subsets and the license text. Two pieces of UI state do not need a store yet.

**Versions.**

| Asset | Version |
|---|---|
| Space Grotesk | v22, latin woff2 from `@fontsource/space-grotesk` 5.3.0 |
| JetBrains Mono | v24, latin woff2 from `@fontsource/jetbrains-mono` 5.3.0 |
| @axe-core/playwright | 4.14.0 |

**Links.** <https://github.com/floriankarsten/space-grotesk> · <https://github.com/JetBrains/JetBrainsMono> · <https://github.com/dequelabs/axe-core-npm/tree/develop/packages/playwright>

---

### 2026-11-20 — AI providers

**Decision.** Both providers implement `LlmProvider`. Claude calls the Anthropic Messages API, `POST /v1/messages`, with header `anthropic-version: 2026-10-21` and `stream: true`. ChatGPT calls the OpenAI Responses API, `POST /v1/responses`, with `stream: true`. A machine-readable call sends OpenAI `text.format` as `{ type: "json_schema", strict: true, name, schema }`. Claude receives the schema as a system instruction. Both replies are parsed as JSON. One failure retries once, with the serde error included in the next user message. A second failure returns the error.

The clients parse the SSE stream into ordered text deltas on `Completion`. The settings Test button calls `GET /v1/models` and fills the select from `data[].id`. It does not send a completion. Token-by-token rendering waits for the script screen.

Model ids are not constants in source. `config/ai.toml` holds the routing table, the USD 20 cap, the 0.8 alert ratio, empty default model strings, and token rates. Rates are USD per million tokens and ship at zero, so the repo does not invent a price. `cost_usd` reads that table. The cap tests pass an explicit cost into the meter, which proves allow, alert, and block while the shipped rates stay zero. Spend at the cap blocks before the provider is called. Spend at 80 percent alerts and still calls.

Keys go to the OS keychain through `keyring` 4.2.0. The service is `shorts-studio`. The accounts are the provider ids `claude` and `openai`. Tests use an in-memory store and do not open the Secret Service. Log lines and Test results pass through `redact`, which removes the saved secret and tokens shaped like `sk-` or `sk-ant-`.

Live tests compile and return immediately unless `LIVE_AI=1`. Listing models also needs `LIVE_AI_ANTHROPIC_KEY` and `LIVE_AI_OPENAI_KEY`. A streamed completion also needs `LIVE_AI_ANTHROPIC_MODEL` and `LIVE_AI_OPENAI_MODEL`. That early return is the M03 skip specified by the prompt.

**Options.** OpenAI Responses API. OpenAI Chat Completions. `reqwest` with rustls. `reqwest` with native-tls.

**Why.** The current Responses guide still accepts `text.format.type = json_schema` with `strict: true`. rustls avoids an OpenSSL header dependency. reqwest 0.13 names that feature `rustls` (the 0.12 name was `rustls-tls`) and pulls aws-lc-rs.

**Versions.**

| Crate | Version |
|---|---|
| reqwest | 0.13.5 (`blocking`, `json`, `rustls`; default features off) |
| keyring | 4.2.0 (default `v1`: Secret Service on Linux, Keychain on macOS, Credential Manager on Windows) |
| toml | 1.2.1 |

**Links.** <https://developers.openai.com/api/docs/guides/streaming-responses> · <https://developers.openai.com/api/docs/guides/structured-outputs?api-mode=responses> · <https://platform.claude.com/docs/en/build-with-claude/streaming> · <https://docs.rs/keyring/4.2.0/keyring/>

---

### 2026-11-20 — M06 recording uses FFmpeg children first

**Decision.** The M06 recording session spawns one FFmpeg process per track (screen, cam, mic) plus a synthetic `cursor.jsonl` writer at 120 Hz. Automated tests use the `lavfi` backend (`testsrc2` / `testsrc` / `sine`) with a fixed `-t` duration and assert ffprobe durations within 50 ms. Real Mac devices use `avfoundation` device ids from `ffmpeg -f avfoundation -list_devices`. Linux portal/`ashpd` + PipeWire screen capture and ScreenCaptureKit window exclusion stay deferred: SCK exclusion needs the M05 prompter window id; portal capture is refused with a clear error until that path lands. Lavfi fixture takes encode with `libx264`; real takes prefer `h264_videotoolbox` when doctor sees it, else `libx264`. Pause/resume uses `kill -STOP` / `kill -CONT` on the child pids.

**Options.** ScreenCaptureKit `SCRecordingOutput` for screen on day one. FFmpeg for all three tracks. ashpd + pipewire for Linux screen in the same module.

**Why.** The prompt's acceptance test is lavfi-shaped and must pass on Omarchy without screen-recording TCC. One FFmpeg path covers fixture, Mac, and cam/mic on Linux. SCK remains the planned exclusion path once M05 exists; pinning it now would not make the lavfi gate greener.

**Library and version.** System FFmpeg (n9.0.2 on this machine). No new Rust capture crate pinned in M06.

**Links.** <https://ffmpeg.org/ffmpeg-devices.html#avfoundation> · <https://docs.rs/screencapturekit/latest/screencapturekit/>

---

### 2026-11-20 — Real Mac screen uses ScreenCaptureKit

**Decision.** On macOS, an `avfoundation` take writes `screen.mov` with `screencapturekit` 1.5.4 (`macos_15_0`): the first display, H.264, MOV, on a dedicated thread so `SCStream` never has to be `Send`. Windows owned by this process are excluded by pid until M05 can pass a prompter window id. Camera and microphone stay FFmpeg `avfoundation` inputs. Missing ids resolve to the first "Capture screen" display, the first non-iPhone camera, and the first non-iPhone microphone. `lavfi` stays the automated gate and still encodes with `libx264`. If ScreenCaptureKit refuses to start, the error tells the user to allow Screen Recording for the `shorts-studio` binary. Pause still only stops the FFmpeg children; the screen track keeps writing.

**Options.** Keep screen on `ffmpeg -f avfoundation` (`{id}:none`). Wait for M05 before any ScreenCaptureKit.

**Why.** `avfoundation` screen on this Mac is device index 3 (`Capture screen 0`), and the old fallback index `1` is an iPhone camera, so real takes were either a lavfi test pattern or the wrong device. ScreenCaptureKit records the display directly and can drop this app's windows without the prompter existing yet.

**Library and version.** `screencapturekit` 1.5.4, feature `macos_15_0`. The media tests and the Tauri binary add `-rpath /usr/lib/swift` so `libswift_Concurrency.dylib` resolves from the dyld shared cache. The Command Line Tools copy is left off that rpath so both Swift runtimes are not loaded together.

**Links.** <https://docs.rs/screencapturekit/1.5.4/screencapturekit/>

---

### 2026-11-20 — Whisper word timestamps

**Decision.** `whisper-rs` 0.16.0 (`whisper-rs-sys` 0.15.0) with the `metal` feature on macOS and the default CPU build elsewhere. A take queues transcription on a background thread after stop. `FullParams` enables token timestamps, `set_split_on_word(true)`, and `set_max_len(1)`, which the 0.16 docs still describe as the word-boundary split. Segment `t0`/`t1` are centiseconds and are stored as seconds on `WordStamp`. Language is left unset so the multilingual model auto-detects. The model path stays `ggml-large-v3-turbo.bin` from doctor. If that file is absent, the report status is `model-absent` and silence detection still runs on `mic.wav`. Fillers default to `umm`, `uh`, `matlab`, `basically`, `you know`. A silence is a PCM gap of at least 0.6 s below RMS 0.02. Chapters match script beat tokens onto the transcript in order.

**Options.** Derive words only from token times. Download the model inside the test gate.

**Why.** `set_split_on_word` is still on `FullParams` in 0.16, and `max_len(1)` is what makes whisper.cpp emit one segment per word. The turbo model is hundreds of megabytes and is not in the repo, so the ±150 ms test uses the committed `known-words.json` fixture. A real Hinglish take stays MANUAL until that file is downloaded.

**Library and version.** `whisper-rs` 0.16.0, `metal` on macOS. Build needs CMake (4.5.1 on this Mac).

**Links.** <https://docs.rs/whisper-rs/0.16.0/whisper_rs/struct.FullParams.html> · <https://docs.rs/whisper-rs/0.16.0/whisper_rs/struct.WhisperSegment.html>

---

### 2026-11-20 — Timeline edits stay in milliseconds

**Decision.** Split, trim, ripple delete, and move run on integer-millisecond spans in `core`, the same unit as `TimelineSpan`. Clip seconds are converted only at the `split_clips` boundary. Undo keeps 100 previous drafts and drops the oldest. The Edit screen draft uses Immer 11.2.10 for the same split and ripple operations, so Playwright can run them without the Rust process. Speed is clamped to 0.25–4. Waveform peaks are max-abs buckets. A 720p proxy is named `*.720p.mp4` at 1280×720; the encode itself waits for the preview path.

**Options.** One Rust edit engine compiled to WASM. Immer patches as the undo log.

**Why.** The property tests need exact durations, and floating clip times would make "total duration" flaky. Snapshot history is enough for a 100-step undo of a small track. Immer is what the plan names for the UI draft.

**Library and version.** `immer` 11.2.10. No new Rust crate.

**Links.** <https://immerjs.github.io/immer/>
