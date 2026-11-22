import { describe, expect, it } from "vitest";
import { etaSeconds, presetById, sizeLabel } from "./settings";

describe("export settings", () => {
  it("keeps the 4K bitrate and estimates the queue", () => {
    expect(presetById("4K")).toMatchObject({
      width: 2160,
      height: 3840,
      bitrateKbps: 40000,
    });
    expect(presetById("1080p").height).toBe(1920);
    expect(etaSeconds(10, 40, 2)).toBeCloseTo(6);
    expect(etaSeconds(0, 40, 2)).toBe(0);
    expect(sizeLabel(40000, 60)).toBe("about 300 MB");
  });
});
