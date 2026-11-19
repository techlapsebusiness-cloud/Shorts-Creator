import { invoke } from "@tauri-apps/api/core";
import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("App", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockResolvedValue({
      name: "shorts-studio",
      version: "0.1.0",
    });
  });

  it("boots the window shell", async () => {
    render(<App />);
    expect(
      screen.getByRole("heading", { name: "Shorts Studio" }),
    ).toBeInTheDocument();
    expect(await screen.findByTestId("health")).toHaveTextContent(
      "shorts-studio",
    );
    expect(invoke).toHaveBeenCalledWith("health");
  });
});
