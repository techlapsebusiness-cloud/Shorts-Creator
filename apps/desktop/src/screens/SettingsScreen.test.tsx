import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import { SettingsScreen, type ProviderTestResult } from "./SettingsScreen";

const key = "sk-ant-ui-example";

function Harness({
  onTest,
  onSpend,
  onCap,
}: {
  onTest: (provider: string, key: string) => Promise<ProviderTestResult>;
  onSpend?: () => Promise<{
    spentUsd: number;
    capUsd: number;
    decision: string;
  }>;
  onCap?: (capUsd: number) => void;
}) {
  const [accent, setAccent] = useState<"rust" | "mint" | "amber" | "blue">(
    "rust",
  );
  return (
    <SettingsScreen
      accent={accent}
      onAccent={setAccent}
      onTest={onTest}
      onSpend={onSpend}
      onCap={onCap}
    />
  );
}

describe("SettingsScreen", () => {
  it("stores nothing in the page and shows models from the test result", async () => {
    const onTest = vi.fn(async (): Promise<ProviderTestResult> => ({
      ok: true,
      message: "Key works",
      models: ["model-from-endpoint"],
      defaultModel: "",
    }));
    render(<Harness onTest={onTest} />);
    fireEvent.change(screen.getByLabelText("Claude key"), {
      target: { value: key },
    });
    fireEvent.click(screen.getByRole("button", { name: "Test Claude" }));
    await waitFor(() => {
      expect(screen.getByRole("status")).toHaveTextContent("Key works");
    });
    expect(onTest).toHaveBeenCalledWith("claude", key);
    expect(screen.getByRole("status").textContent ?? "").not.toContain(key);
    expect(screen.getByLabelText("Claude key")).toHaveValue("");
    expect(screen.getByRole("combobox", { name: "Claude model" })).toHaveValue(
      "model-from-endpoint",
    );
  });

  it("shows the cap alert from the spend status", async () => {
    const onCap = vi.fn();
    render(
      <Harness
        onTest={vi.fn()}
        onSpend={async () => ({
          spentUsd: 16,
          capUsd: 20,
          decision: "alert",
        })}
        onCap={onCap}
      />,
    );
    await waitFor(() => {
      expect(screen.getByTestId("spend")).toHaveTextContent("Cap alert.");
    });
    fireEvent.change(screen.getByLabelText("Monthly cap USD"), {
      target: { value: "25" },
    });
    fireEvent.blur(screen.getByLabelText("Monthly cap USD"));
    expect(onCap).toHaveBeenCalledWith(25);
  });
});
