import { describe, expect, it } from "vitest";
import { applyAccent } from "./accents";

describe("applyAccent", () => {
  it("writes the accent custom property", () => {
    applyAccent("mint");
    expect(document.documentElement.style.getPropertyValue("--accent")).toBe(
      "#4FE3C1",
    );
    applyAccent("rust");
    expect(document.documentElement.style.getPropertyValue("--accent")).toBe(
      "#FF7A3D",
    );
  });
});
