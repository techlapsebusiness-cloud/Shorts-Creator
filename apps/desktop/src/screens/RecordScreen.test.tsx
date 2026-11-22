import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { RecordScreen } from "./RecordScreen";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("RecordScreen", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_capture_devices") {
        return {
          screens: [],
          cameras: [],
          microphones: [],
          backend: "avfoundation",
        };
      }
      if (cmd === "list_takes") {
        return [];
      }
      if (cmd === "disk_estimate") {
        return {
          freeBytes: 10_000_000_000,
          bytesPerMinute: 180_000_000,
          minutesRemaining: 55,
        };
      }
      if (cmd === "prepare_take") {
        return "/tmp/take";
      }
      if (cmd === "start_recording") {
        return null;
      }
      if (cmd === "stop_recording") {
        return {
          screen: "media/screen.mov",
          cam: "media/cam.mov",
          mic: "media/mic.wav",
          cursor: "media/cursor.jsonl",
          sync: { screenOffsetMs: 0, camOffsetMs: 1, micOffsetMs: 2 },
        };
      }
      throw new Error(cmd);
    });
  });

  it("lists devices and starts a lavfi take", async () => {
    render(<RecordScreen countdownSecs={0} lavfiHoldMs={0} />);
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("list_capture_devices");
    });
    expect(screen.getByLabelText("Camera")).toHaveDisplayValue(
      "No camera found",
    );
    expect(screen.getByLabelText("Microphone")).toHaveDisplayValue(
      "No microphone found",
    );
    fireEvent.click(screen.getByRole("button", { name: "Start recording" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("prepare_take");
    });
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith(
        "start_recording",
        expect.objectContaining({
          args: expect.objectContaining({
            backend: "lavfi",
            layout: "screenCam",
          }),
        }),
      );
    });
    await waitFor(() => {
      expect(screen.getByTestId("last-take")).toHaveTextContent(
        "media/screen.mov",
      );
    });
  });

  it("opens a listed take in the editor", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_capture_devices") {
        return {
          screens: [{ id: "1", label: "Capture screen 0", kind: "screen" }],
          cameras: [{ id: "0", label: "FaceTime HD Camera", kind: "camera" }],
          microphones: [
            { id: "0", label: "MacBook Air Microphone", kind: "mic" },
          ],
          backend: "avfoundation",
        };
      }
      if (cmd === "list_takes") {
        return [
          {
            id: "1000",
            dir: "/tmp/takes/1000",
            createdMs: 1000,
            durationSecs: 3,
            screen: true,
            cam: true,
            mic: true,
          },
        ];
      }
      if (cmd === "open_take") {
        return {
          dir: "/tmp/movies/1000-take",
          project: { id: "1000", name: "Take 1000", tracks: [] },
        };
      }
      if (cmd === "list_capture_windows") {
        return [];
      }
      if (cmd === "desktop_icon_support") {
        return { supported: false };
      }
      if (cmd === "disk_estimate") {
        return {
          freeBytes: 10_000_000_000,
          bytesPerMinute: 180_000_000,
          minutesRemaining: 55,
        };
      }
      return null;
    });
    const onOpened = vi.fn();
    render(<RecordScreen countdownSecs={0} onOpened={onOpened} />);
    await waitFor(() => {
      expect(screen.getByLabelText("Camera")).toHaveDisplayValue(
        "FaceTime HD Camera",
      );
    });
    expect(screen.getByLabelText("Microphone")).toHaveDisplayValue(
      "MacBook Air Microphone",
    );
    fireEvent.click(screen.getByRole("button", { name: "Open in editor" }));
    await waitFor(() => {
      expect(onOpened).toHaveBeenCalledWith(
        expect.objectContaining({ dir: "/tmp/movies/1000-take" }),
      );
    });
  });
});
