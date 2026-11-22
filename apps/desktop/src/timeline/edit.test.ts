import { describe, expect, it } from "vitest";
import {
  clampSpeed,
  historyOf,
  nextTransition,
  proxyFileName,
  pushHistory,
  redoHistory,
  rippleDelete,
  setSpeed,
  snapTime,
  splitSpan,
  trimSpan,
  undoHistory,
  waveformPeaks,
  type Span,
} from "./edit";

const screen: Span = { id: "screen", startMs: 0, durationMs: 10_000 };

describe("timeline edits", () => {
  it("splits, ripple-deletes, and undoes back to the split", () => {
    let history = historyOf([screen]);
    history = pushHistory(history, splitSpan(history.present, "screen", 5_000));
    expect(history.present).toHaveLength(2);
    expect(
      history.present.reduce((sum, span) => sum + span.durationMs, 0),
    ).toBe(10_000);
    const right = history.present[1];
    history = pushHistory(history, rippleDelete(history.present, right.id));
    expect(history.present).toEqual([
      { id: "screen-a", startMs: 0, durationMs: 5_000 },
    ]);
    history = undoHistory(history);
    expect(history.present).toHaveLength(2);
    history = redoHistory(history);
    expect(history.present).toHaveLength(1);
  });

  it("trims, snaps, clamps speed, and names the 720p proxy", () => {
    const trimmed = trimSpan([screen], "screen", 4_000);
    expect(trimmed[0]?.durationMs).toBe(4_000);
    expect(snapTime(980, [0, 1_000], 40)).toBe(1_000);
    expect(clampSpeed(9)).toBe(4);
    expect(clampSpeed(0.1)).toBe(0.25);
    expect(setSpeed([screen], "screen", 2)[0]?.speed).toBe(2);
    expect(waveformPeaks([0.2, -0.9, 0.4], 2)).toEqual([0.9, 0.4]);
    expect(proxyFileName("screen.mov")).toBe("screen.720p.mp4");
    expect(nextTransition("glitch")).toBe("cut");
  });
});
