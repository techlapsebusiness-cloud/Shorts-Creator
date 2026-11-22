import { describe, expect, it } from "vitest";
import { endBeat, hookFits, nextTitle } from "./card";

describe("cover and end card", () => {
  it("keeps both hooks inside a line and hits the four end-card beats", () => {
    expect(hookFits("This file survives")).toBe(true);
    expect(hookFits("rename failed ".repeat(12))).toBe(true);
    expect(endBeat(0)).toBe("chevron");
    expect(endBeat(0.6)).toBe("next");
    expect(endBeat(1.3)).toBe("follow");
    expect(endBeat(2)).toBe("tagline");
    expect(nextTitle(["Rename", "Exposure"], "Rename")).toBe("Exposure");
  });
});
