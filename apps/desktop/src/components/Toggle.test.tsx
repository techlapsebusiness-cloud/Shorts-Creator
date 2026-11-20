import { fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it } from "vitest";
import { Toggle } from "./Toggle";

function Harness() {
  const [checked, setChecked] = useState(false);
  return <Toggle label="Mirror" checked={checked} onChange={setChecked} />;
}

describe("Toggle", () => {
  it("flips aria-checked", () => {
    render(<Harness />);
    const toggle = screen.getByRole("switch", { name: "Mirror" });
    expect(toggle).toHaveAttribute("aria-checked", "false");
    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute("aria-checked", "true");
  });

  it("stays on when it is locked", () => {
    render(<Toggle label="Hide from recording" checked disabled />);
    const toggle = screen.getByRole("switch", { name: "Hide from recording" });
    expect(toggle).toBeDisabled();
    expect(toggle).toHaveAttribute("aria-checked", "true");
  });
});
