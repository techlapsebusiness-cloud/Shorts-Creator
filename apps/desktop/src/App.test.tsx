import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";

const host = vi.hoisted(() => ({ mac: false }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("./record/host", () => ({
  hostRecordsDirectly: () => host.mac,
}));

describe("App", () => {
  beforeEach(() => {
    host.mac = false;
    window.history.pushState(null, "", "/");
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "health") {
        return { name: "shorts-studio", version: "0.1.0" };
      }
      if (cmd === "spend_status") {
        return { spentUsd: 0, capUsd: 20, decision: "allow" };
      }
      if (
        cmd === "show_prompter" ||
        cmd === "prompter_click_through" ||
        cmd === "stop_take"
      ) {
        return null;
      }
      if (cmd === "start_take") {
        return {
          directory: "/Users/me/Movies/ShortsStudio/takes/1",
        };
      }
      if (cmd === "recording_gate") {
        throw new Error(
          "A monitor capture includes the prompter. Move the prompter to another monitor, or capture a window or region that does not overlap it.",
        );
      }
      throw new Error(cmd);
    });
  });

  it("boots the record screen and reads health", async () => {
    render(<App />);
    expect(
      screen.getByRole("heading", { name: "sudoakshay/shorts" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Script" })).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByTestId("health")).toHaveTextContent("shorts-studio");
    });
    expect(invoke).toHaveBeenCalledWith("health");
  });

  it("opens each step and the settings screen", () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "02 Edit" }));
    expect(screen.getByRole("heading", { name: "Media" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "03 Audio" }));
    expect(screen.getByRole("heading", { name: "Voice" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "04 Auto assets" }));
    expect(screen.getByRole("heading", { name: "Covers" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "05 Export" }));
    expect(
      screen.getByRole("heading", { name: "Resolution" }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "06 Upload" }));
    expect(
      screen.getByRole("heading", { name: "Upload kit" }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "AI settings" }));
    expect(
      screen.getByRole("heading", { name: "Providers" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Daily Short mode" }),
    ).toBeInTheDocument();
  });

  it("refuses a monitor capture that includes the prompter", async () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Start recording" }));
    await waitFor(() => {
      expect(screen.getByRole("status")).toHaveTextContent("another monitor");
    });
  });

  it("starts a take on a Mac and the same button stops it", async () => {
    host.mac = true;
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Start recording" }));
    await waitFor(() => {
      expect(screen.getByRole("status")).toHaveTextContent(
        "/Users/me/Movies/ShortsStudio/takes/1",
      );
    });
    expect(invoke).toHaveBeenCalledWith("start_take");
    fireEvent.click(
      screen.getAllByRole("button", { name: "Stop recording" })[0],
    );
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("stop_take");
    });
  });
});
