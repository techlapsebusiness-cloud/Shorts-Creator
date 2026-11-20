import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Pill } from "./Pill";

describe("Pill", () => {
  it("exposes the selected state", () => {
    const onClick = vi.fn();
    render(
      <Pill selected onClick={onClick}>
        CTA
      </Pill>,
    );
    const pill = screen.getByRole("button", { name: "CTA" });
    expect(pill).toHaveAttribute("aria-pressed", "true");
    fireEvent.click(pill);
    expect(onClick).toHaveBeenCalledOnce();
  });
});
