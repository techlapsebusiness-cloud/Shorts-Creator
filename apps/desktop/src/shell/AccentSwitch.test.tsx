import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AccentSwitch } from "./AccentSwitch";

describe("AccentSwitch", () => {
  it("selects a colour", () => {
    const onChange = vi.fn();
    render(<AccentSwitch value="rust" onChange={onChange} />);
    const mint = screen.getByRole("radio", { name: "Mint" });
    expect(mint).toHaveAttribute("aria-checked", "false");
    fireEvent.click(mint);
    expect(onChange).toHaveBeenCalledWith("mint");
  });
});
