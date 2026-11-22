import { describe, expect, it } from "vitest";
import {
  CAPTION_LINE_CHARS,
  MAX_CROP_STEP_PX,
  camRect,
  captionPage,
  cropTarget,
  safeRect,
  stepCrop,
  zoomAt,
} from "./frame";

describe("layout frame", () => {
  it("keeps a jumped crop inside the source and under the step cap", () => {
    const sourceW = 1920;
    const sourceH = 1080;
    const start = cropTarget(200, 200, sourceW, sourceH, 1);
    let follow = { x: start.x, y: start.y, vx: 0, vy: 0 };
    const jump = cropTarget(1800, 900, sourceW, sourceH, 1);
    const stepped = stepCrop(follow, jump, sourceW, sourceH, 1 / 30);
    const moved = Math.hypot(
      stepped.follow.x - follow.x,
      stepped.follow.y - follow.y,
    );
    expect(moved).toBeLessThanOrEqual(MAX_CROP_STEP_PX + 1e-6);
    expect(stepped.rect.x).toBeGreaterThanOrEqual(0);
    expect(stepped.rect.y).toBeGreaterThanOrEqual(0);
    expect(stepped.rect.x + stepped.rect.width).toBeLessThanOrEqual(
      sourceW + 1,
    );
    follow = stepped.follow;
    for (let frame = 0; frame < 180; frame += 1) {
      const next = stepCrop(follow, jump, sourceW, sourceH, 1 / 30);
      expect(
        Math.hypot(next.follow.x - follow.x, next.follow.y - follow.y),
      ).toBeLessThanOrEqual(MAX_CROP_STEP_PX + 1e-6);
      follow = next.follow;
    }
    expect(Math.abs(follow.x - jump.x)).toBeLessThan(1);
    expect(Math.abs(follow.y - jump.y)).toBeLessThan(1);
  });

  it("snaps the camera inside the safe zone and blends zoom", () => {
    expect(
      zoomAt(
        [
          [0, 1],
          [1, 2],
        ],
        0.5,
      ),
    ).toBeCloseTo(1.5);
    const safe = safeRect(1080, 1920);
    const cam = camRect(1080, 1920, "Bottom right", "L", true);
    expect(cam.x).toBeGreaterThanOrEqual(safe.x - 0.01);
    expect(cam.y).toBeGreaterThanOrEqual(safe.y - 0.01);
    expect(cam.x + cam.width).toBeLessThanOrEqual(safe.x + safe.width + 0.01);
    expect(cam.y + cam.height).toBeLessThanOrEqual(safe.y + safe.height + 0.01);
    const loose = camRect(1080, 1920, "Bottom right", "L", false);
    expect(loose.x + loose.width).toBeGreaterThan(safe.x + safe.width);
  });

  it("breaks a long Hinglish line into two caption lines", () => {
    const words = [
      "Yeh",
      "file",
      "crash",
      "ke",
      "baad",
      "bhi",
      "survive",
      "karti",
      "hai",
      "kyunki",
      "rename",
      "atomic",
      "hota",
      "hai",
    ];
    const page = captionPage(words, 9, CAPTION_LINE_CHARS);
    expect(page.lines).toHaveLength(2);
    expect(page.lines[page.activeLine][page.activeIndex]).toBe("kyunki");
    for (const line of page.lines) {
      if (line.length > 1) {
        expect(line.join(" ").length).toBeLessThanOrEqual(CAPTION_LINE_CHARS);
      }
    }
  });
});
