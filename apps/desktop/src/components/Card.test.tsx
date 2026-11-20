import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Card } from "./Card";

describe("Card", () => {
  it("shows its title and body", () => {
    render(
      <Card title="Script">
        <p>Placeholder beat</p>
      </Card>,
    );
    expect(screen.getByRole("heading", { name: "Script" })).toBeInTheDocument();
    expect(screen.getByText("Placeholder beat")).toBeInTheDocument();
  });
});
