import { describe, expect, it } from "vitest";
import {
  buildKit,
  directUploadAllowed,
  keywordTitle,
  kitDescription,
  plainText,
} from "./kit";

describe("upload kit", () => {
  it("puts the keyword first and strips banned hashtags", () => {
    const kit = buildKit(
      "rename",
      "This file survives a crash",
      "https://example.com/long",
      "exposure",
    );
    expect(kit.title.toLowerCase().startsWith("rename")).toBe(true);
    expect(kit.title.length).toBeLessThanOrEqual(100);
    expect(kit.description).toContain("#Shorts");
    expect(kit.description.toLowerCase()).not.toContain("viral");
    expect(kit.checkedOn).toBe("2026-11-22");
    const messy = kitDescription("Hook", "https://example.com", [
      "#Viral",
      "#fyp",
      "#howto",
    ]);
    expect(messy).toContain("#howto");
    expect(messy.toLowerCase()).not.toContain("viral");
    expect(plainText(kit)).toContain("Audience: Not made for kids");
    expect(keywordTitle("rename", "word ".repeat(40)).length).toBe(100);
    expect(directUploadAllowed(true, "")).toBe(false);
    expect(directUploadAllowed(true, "client")).toBe(true);
    expect(buildKit("ffmpeg", "Hook", "https://example.com", "next").tags).toBe(
      "ffmpeg, shorts",
    );
  });
});
