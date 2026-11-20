import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { RecordScreen, type GeneratedScript } from "./RecordScreen";

const generated: GeneratedScript = {
  rawDraft: "ORIGINAL",
  beats: [
    {
      kind: "hook",
      start: 0,
      text: "Naya hook.",
      cues: ["CUE: point at the folder"],
    },
    { kind: "concept", start: 3, text: "Idea.", cues: [] },
    { kind: "demo", start: 8, text: "Show it.", cues: [] },
    { kind: "fix", start: 15, text: "Fix it.", cues: [] },
    { kind: "cta", start: 22, text: "Do it.", cues: [] },
  ],
};

describe("RecordScreen", () => {
  it("shows the pace and asks for a model before generating", () => {
    render(<RecordScreen />);
    expect(
      screen.getByText("27 words · 11.6 s at 140 wpm"),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Generate" }));
    expect(screen.getByRole("status")).toHaveTextContent(
      "Pick a model in Settings.",
    );
  });

  it("replaces one beat and keeps the raw draft", async () => {
    const onGenerate = vi.fn(async () => generated);
    const onRegenerate = vi.fn(async () => ({
      kind: "hook",
      start: 0,
      text: "Edited hook.",
      cues: ["CUE: point at the folder"],
    }));
    render(
      <RecordScreen
        claudeModel="from-the-caller"
        onGenerate={onGenerate}
        onRegenerate={onRegenerate}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Generate" }));
    await waitFor(() => {
      expect(screen.getByLabelText("HOOK beat")).toHaveValue("Naya hook.");
    });
    expect(screen.getByText("CUE: point at the folder")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Regenerate HOOK" }));
    await waitFor(() => {
      expect(screen.getByLabelText("HOOK beat")).toHaveValue("Edited hook.");
    });
    expect(screen.getByText(/words ·/).getAttribute("data-raw-draft")).toBe(
      "ORIGINAL",
    );
    expect(onRegenerate).toHaveBeenCalledWith(
      expect.objectContaining({ model: "from-the-caller", provider: "claude" }),
      0,
      generated.beats,
    );
  });
});
