import { describe, expect, it } from "vitest";
import { readHealthName } from "./health";

describe("readHealthName", () => {
  it("reads a health payload", () => {
    expect(readHealthName({ name: "shorts-studio", version: "0.1.0" })).toBe(
      "shorts-studio",
    );
  });

  it("rejects an empty payload", () => {
    expect(readHealthName({})).toBeNull();
    expect(readHealthName(null)).toBeNull();
  });
});
