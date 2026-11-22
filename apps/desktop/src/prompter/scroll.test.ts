import { describe, expect, it } from "vitest";
import {
  FOLLOW_CONFIDENCE,
  followOffsetPx,
  fuzzyMatch,
  nudgeOffset,
  scrollMode,
  scrollOffsetPx,
} from "./scroll";

describe("prompter scroll", () => {
  it("turns elapsed time into an offset and ignores paused time", () => {
    expect(scrollOffsetPx(0, 0, 100)).toBe(0);
    expect(scrollOffsetPx(1000, 0, 120)).toBe(120);
    expect(scrollOffsetPx(2000, 500, 100)).toBe(150);
    expect(nudgeOffset(10, -40)).toBe(0);
    expect(nudgeOffset(10, 5)).toBe(15);
  });

  it("follows a Hinglish line and falls back when the match is weak", () => {
    const lines = [
      "yeh file crash ke baad bhi bachegi",
      "project.json mein edit rehti hai",
      "agla short banao",
    ];
    const hit = fuzzyMatch(lines, "file crash ke baad bachegi");
    expect(hit.index).toBe(0);
    expect(hit.confidence).toBeGreaterThanOrEqual(FOLLOW_CONFIDENCE);
    expect(scrollMode(hit.confidence, false)).toBe("follow");
    const miss = fuzzyMatch(lines, "asdf qwer zxcv");
    expect(miss.confidence).toBeLessThan(FOLLOW_CONFIDENCE);
    expect(scrollMode(miss.confidence, false)).toBe("fixed");
    expect(scrollMode(1, true)).toBe("paused");
    expect(followOffsetPx(2, 48)).toBe(96);
  });
});
