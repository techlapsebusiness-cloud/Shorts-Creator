import { describe, expect, it } from "vitest";
import type { Project } from "../bindings/Project";
import fixture from "../../../../crates/core/fixtures/project-v1.json";
import { projectSchema } from "./project";

describe("project schema", () => {
  it("parses the v1 fixture as the generated Project type", () => {
    const raw: unknown = fixture;
    const parsed: Project = projectSchema.parse(raw);
    expect(parsed.version).toBe(1);
    expect(parsed.slug).toBe("example-short");
    expect(parsed.tracks).toHaveLength(7);
    const screen = parsed.tracks.find((track) => track.id === "V1");
    expect(screen?.clips[0]?.body).toMatchObject({
      kind: "video",
      sourceIn: 0,
      sourceOut: 2.5,
    });
  });
});
