import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Button } from "./Button";

describe("Button", () => {
  it("calls onClick", () => {
    const onClick = vi.fn();
    render(<Button onClick={onClick}>Render</Button>);
    fireEvent.click(screen.getByRole("button", { name: "Render" }));
    expect(onClick).toHaveBeenCalledOnce();
  });

  it("does not call onClick when disabled", () => {
    const onClick = vi.fn();
    render(
      <Button tone="primary" disabled onClick={onClick}>
        Render
      </Button>,
    );
    fireEvent.click(screen.getByRole("button", { name: "Render" }));
    expect(onClick).not.toHaveBeenCalled();
  });
});
