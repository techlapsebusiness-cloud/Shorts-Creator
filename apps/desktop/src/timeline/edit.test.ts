import { describe, expect, it } from "vitest";
import {
  historyOf,
  pushHistory,
  redoHistory,
  rippleDelete,
  splitSpan,
  undoHistory,
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
});
