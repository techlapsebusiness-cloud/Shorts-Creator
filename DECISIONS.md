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

**Library and version.** `reqwest`, unpinned. Pin in M03.

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
