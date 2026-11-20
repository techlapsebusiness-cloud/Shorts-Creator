import { describe, expect, it } from "vitest";
import { estimateSeconds, wordCount } from "./estimate";

describe("estimate", () => {
  it("counts words and turns a pace into seconds", () => {
    expect(wordCount("  ek   do teen  ")).toBe(3);
    expect(wordCount("")).toBe(0);
    expect(estimateSeconds("ek do teen", 60)).toBeCloseTo(3);
    expect(estimateSeconds("ek", 0)).toBe(0);
  });
});
