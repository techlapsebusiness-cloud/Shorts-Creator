import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { PrompterWindow } from "./PrompterWindow";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async () => null),
}));

vi.mock("@tauri-apps/api/event", () => ({
  emit: vi.fn(async () => null),
  listen: vi.fn(async () => () => undefined),
}));

describe("PrompterWindow", () => {
  it("styles cue lines and pauses from the keyboard", () => {
    render(<PrompterWindow />);
    expect(screen.getByText("PROMPTER · NOT RECORDED")).toBeInTheDocument();
    expect(screen.getByText("CUE: point at the folder")).toHaveAttribute(
      "data-cue",
      "true",
    );
    expect(screen.getByRole("status")).toHaveTextContent("Ready");
    fireEvent.keyDown(window, { code: "Space" });
    expect(screen.getByRole("status")).toHaveTextContent("Paused");
    fireEvent.keyDown(window, { code: "KeyP", shiftKey: true, metaKey: true });
    expect(screen.getByRole("status")).toHaveTextContent("Prompter hidden");
    expect(
      screen.queryByText("CUE: point at the folder"),
    ).not.toBeInTheDocument();
  });

  it("starts the countdown when recording is armed", () => {
    render(<PrompterWindow />);
    fireEvent(window, new Event("prompter-arm"));
    expect(screen.getByRole("status")).toHaveTextContent("3");
  });
});
