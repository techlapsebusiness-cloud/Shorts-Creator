import { describe, expect, it } from "vitest";
import {
  AUDIO_BITRATE_KBPS,
  VIDEO_BITRATE_KBPS,
  estimateMegabytes,
  micLevel,
} from "./estimate";

describe("take estimate", () => {
  it("matches the bitrate table and the mic peak", () => {
    expect(
      estimateMegabytes(30, VIDEO_BITRATE_KBPS, AUDIO_BITRATE_KBPS, 2),
    ).toBeCloseTo(301.44, 2);
    expect(
      estimateMegabytes(0, VIDEO_BITRATE_KBPS, AUDIO_BITRATE_KBPS, 2),
    ).toBe(0);
    expect(micLevel([])).toBe(0);
    expect(micLevel([0, 32767, -1000])).toBeCloseTo(1, 5);
  });
});
