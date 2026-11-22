import { describe, expect, it } from "vitest";
import { blocks, exampleShot, preflight } from "./preflight";

function state(checks: ReturnType<typeof preflight>, id: string) {
  return checks.find((check) => check.id === id)?.state;
}

describe("preflight", () => {
  it("passes a distinct short and warns without blocking", () => {
    const ok = preflight(exampleShot(true));
    expect(ok.every((check) => check.state === "pass")).toBe(true);
    const unticked = preflight(exampleShot(false));
    expect(state(unticked, "licence")).toBe("warn");
    expect(blocks()).toBe(false);
    const near = exampleShot(true);
    near.script = near.rawDraft;
    near.history.push({
      topic: near.topic,
      title: near.title,
      layoutKey: near.layoutKey,
    });
    const warned = preflight(near);
    expect(state(warned, "similar")).toBe("warn");
    expect(state(warned, "draft")).toBe("warn");
    expect(state(warned, "layout")).toBe("warn");
    const wide = exampleShot(true);
    wide.width = 1920;
    wide.height = 1080;
    wide.durationSecs = 90;
    const failed = preflight(wide);
    expect(state(failed, "aspect")).toBe("fail");
    expect(state(failed, "length")).toBe("fail");
  });
});
