import type { Page } from "@playwright/test";

export async function mockTauri(page: Page) {
  await page.addInitScript(() => {
    const internals = {
      invoke: (cmd: string) => {
        if (cmd === "health") {
          return Promise.resolve({ name: "shorts-studio", version: "0.1.0" });
        }
        if (cmd === "spend_status") {
          return Promise.resolve({
            spentUsd: 0,
            capUsd: 20,
            decision: "allow",
          });
        }
        if (cmd === "test_provider") {
          return Promise.resolve({
            ok: true,
            message: "Key works",
            models: [],
            defaultModel: "",
          });
        }
        if (cmd === "set_spend_cap") {
          return Promise.resolve(null);
        }
        if (cmd === "list_capture_devices") {
          return Promise.resolve({
            screens: [],
            cameras: [],
            microphones: [],
            backend: "avfoundation",
          });
        }
        if (cmd === "list_takes") {
          return Promise.resolve([]);
        }
        if (cmd === "open_take" || cmd === "save_opened_project") {
          return Promise.resolve({
            dir: "/tmp/movies/1000-take",
            project: { id: "1000", name: "Take 1000", tracks: [] },
          });
        }
        if (cmd === "disk_estimate") {
          return Promise.resolve({
            freeBytes: 10_000_000_000,
            bytesPerMinute: 180_000_000,
            minutesRemaining: 55,
          });
        }
        if (cmd === "prepare_take") {
          return Promise.resolve("/tmp/take");
        }
        if (cmd === "start_recording" || cmd === "stop_recording") {
          return Promise.resolve({
            screen: "media/screen.mov",
            cam: "media/cam.mov",
            mic: "media/mic.wav",
            cursor: "media/cursor.jsonl",
            sync: { screenOffsetMs: 0, camOffsetMs: 0, micOffsetMs: 0 },
          });
        }
        if (
          cmd === "pause_recording" ||
          cmd === "resume_recording" ||
          cmd === "recording_status"
        ) {
          return Promise.resolve({ phase: "idle" });
        }
        return Promise.reject(new Error(`unmocked command ${cmd}`));
      },
      metadata: {
        currentWindow: { label: "main" },
        currentWebview: { windowLabel: "main", label: "main" },
      },
      transformCallback: () => 0,
      unregisterCallback: () => undefined,
      convertFileSrc: (filePath: string) => filePath,
    };
    Object.defineProperty(window, "__TAURI_INTERNALS__", { value: internals });
    const getUserMedia = (() =>
      Promise.resolve({
        getTracks: () => [],
        getVideoTracks: () => [],
      })) as unknown as MediaDevices["getUserMedia"];
    const media = navigator.mediaDevices;
    if (media) {
      media.getUserMedia = getUserMedia;
    } else {
      Object.defineProperty(navigator, "mediaDevices", {
        configurable: true,
        value: { getUserMedia },
      });
    }
  });
}
