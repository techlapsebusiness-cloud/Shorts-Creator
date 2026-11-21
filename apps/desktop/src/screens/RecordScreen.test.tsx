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
});
