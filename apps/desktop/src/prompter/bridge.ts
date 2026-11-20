import { emit } from "@tauri-apps/api/event";

export type PrompterLine = {
  text: string;
  cue: boolean;
};

export type PrompterSettings = {
  lines: PrompterLine[];
  wordsPerMinute: number;
  textSize: number;
  opacity: number;
  mirror: boolean;
  underWebcam: boolean;
  paused: boolean;
};

export async function publishPrompter(detail: PrompterSettings) {
  window.dispatchEvent(new CustomEvent("prompter-settings", { detail }));
  try {
    await emit("prompter-settings", detail);
  } catch {
    // The Vite preview has no Tauri event bridge.
  }
}

export async function armPrompter() {
  window.dispatchEvent(new Event("prompter-arm"));
  try {
    await emit("prompter-arm");
  } catch {
    // The Vite preview has no Tauri event bridge.
  }
}
