import { describe, expect, it } from "vitest";
import { advance, explainFault, pausedStep } from "./steps";

describe("daily steps", () => {
  it("pauses on review and record, then finishes the auto steps", () => {
    const review = advance([]);
    expect(review.at(-1)).toBe("review");
    expect(pausedStep(review)?.id).toBe("review");
    const record = advance(review);
    expect(record.at(-1)).toBe("record");
    const done = advance(record);
    expect(done.at(-1)).toBe("kit");
    expect(pausedStep(done)).toBeNull();
    expect(explainFault("ffmpeg not found")).toMatch(/ffmpeg/);
    expect(explainFault("permission denied")).toMatch(/permission/);
    expect(explainFault("no space left")).toMatch(/disk/);
  });
});
