import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SETTINGS, STEPS } from "./routes";
import { TopBar } from "./TopBar";

describe("TopBar", () => {
  it("shows the identity, steps, settings, and context action", () => {
    const onNavigate = vi.fn();
    render(
      <TopBar
        projectName="Example short"
        route={STEPS[0]}
        onNavigate={onNavigate}
      />,
    );
    expect(
      screen.getByRole("heading", { name: "sudoakshay/shorts" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Example short")).toBeInTheDocument();
    for (const step of STEPS) {
      expect(
        screen.getByRole("button", { name: step.label }),
      ).toBeInTheDocument();
    }
    expect(screen.getByRole("button", { name: "01 Record" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    fireEvent.click(screen.getByRole("button", { name: "02 Edit" }));
    expect(onNavigate).toHaveBeenCalledWith("/edit");
    fireEvent.click(screen.getByRole("button", { name: SETTINGS.label }));
    expect(onNavigate).toHaveBeenCalledWith("/settings");
    expect(
      screen.getByRole("button", { name: "Start recording" }),
    ).toBeInTheDocument();
  });
});
