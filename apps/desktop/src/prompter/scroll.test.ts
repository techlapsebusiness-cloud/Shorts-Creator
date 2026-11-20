import { describe, expect, it } from "vitest";
import { countdownLabel, scrollOffsetPx } from "./scroll";

const base = {
  startMs: 3000,
  wordsPerMinute: 60,
  textSize: 20,
};

describe("prompter scroll", () => {
  it("turns elapsed time into an offset and holds the countdown", () => {
    expect(
      scrollOffsetPx({ ...base, nowMs: 3000, pausedMs: 0, nudgePx: 0 }),
    ).toBe(0);
    expect(
      scrollOffsetPx({ ...base, nowMs: 63000, pausedMs: 0, nudgePx: 0 }),
    ).toBe(600);
    expect(
      scrollOffsetPx({ ...base, nowMs: 63000, pausedMs: 30000, nudgePx: 0 }),
    ).toBe(300);
    expect(
      scrollOffsetPx({ ...base, nowMs: 63000, pausedMs: 0, nudgePx: 40 }),
    ).toBe(640);
    expect(
      scrollOffsetPx({ ...base, nowMs: 1000, pausedMs: 0, nudgePx: 12 }),
    ).toBe(0);
    expect(countdownLabel(0)).toBe(3);
    expect(countdownLabel(1000)).toBe(2);
    expect(countdownLabel(2000)).toBe(1);
    expect(countdownLabel(3000)).toBeNull();
  });
});
