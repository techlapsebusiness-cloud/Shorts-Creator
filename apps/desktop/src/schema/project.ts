import { z } from "zod";
import type { Project } from "../bindings/Project";

const trackId = z.enum(["V3", "V2", "V1", "TXT", "A1", "A2", "A3"]);
const easing = z.enum(["linear", "easeInOut", "spring"]);
const keyframeProperty = z.enum(["zoom", "x", "y", "opacity"]);
const layoutPreset = z.enum(["screenCam", "camOnly", "screenOnly"]);
const camPosition = z.enum([
  "topLeft",
  "topCenter",
  "topRight",
  "middleLeft",
  "middleCenter",
  "middleRight",
  "bottomLeft",
  "bottomCenter",
  "bottomRight",
]);
const camSize = z.enum(["S", "M", "L"]);
const camShape = z.enum(["circle", "rounded", "square"]);
const coverTemplate = z.enum(["hookFace", "errorHook", "beforeAfter"]);
const endCardStyle = z.enum(["rise", "glitch", "slide", "terminal"]);
const videoCodec = z.enum(["h264", "hevc"]);
const beatKind = z.enum(["hook", "concept", "demo", "fix", "cta"]);

const wordStamp = z
  .object({
    text: z.string(),
    start: z.number(),
    end: z.number(),
  })
  .strict();

const keyframe = z
  .object({
    time: z.number(),
    property: keyframeProperty,
    value: z.number(),
    easing,
  })
  .strict();

const mediaBody = {
  source: z.string(),
  sourceIn: z.number(),
  sourceOut: z.number(),
  speed: z.number(),
};

const clipBody = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("video"), ...mediaBody }).strict(),
  z.object({ kind: z.literal("audio"), ...mediaBody }).strict(),
  z
    .object({
      kind: z.literal("caption"),
      text: z.string(),
      words: z.array(wordStamp),
    })
    .strict(),
  z.object({ kind: z.literal("overlay"), label: z.string() }).strict(),
]);

const clip = z
  .object({
    id: z.string(),
    timelineStart: z.number(),
    duration: z.number(),
    body: clipBody,
    keyframes: z.array(keyframe),
  })
  .strict();

const track = z
  .object({
    id: trackId,
    clips: z.array(clip),
    muted: z.boolean().default(false),
    solo: z.boolean().default(false),
  })
  .strict();

const audioStage = z
  .object({
    enabled: z.boolean(),
    amount: z.number(),
  })
  .strict();

export const projectSchema: z.ZodType<Project> = z
  .object({
    version: z.number(),
    id: z.string(),
    number: z.number(),
    name: z.string(),
    slug: z.string(),
    tracks: z.array(track),
    layout: z
      .object({
        preset: layoutPreset,
        camPosition,
        camSize,
        camShape,
        safeZone: z.boolean(),
        accent: z.string(),
        transition: z.string().default("cut"),
        markersSecs: z.array(z.number()).default([]),
      })
      .strict(),
    audioChain: z
      .object({
        noise: audioStage,
        eq: audioStage,
        compressor: audioStage,
        deesser: audioStage,
        loudness: z
          .object({
            enabled: z.boolean(),
            integratedLufs: z.number(),
            truePeakDb: z.number(),
          })
          .strict(),
        ducking: z
          .object({
            enabled: z.boolean(),
            attackMs: z.number(),
            releaseMs: z.number(),
            reductionDb: z.number(),
          })
          .strict(),
      })
      .strict(),
    assets: z
      .object({
        coverTemplate,
        endCardStyle,
        addEndCard: z.boolean(),
        burnCover: z.boolean(),
      })
      .strict(),
    export: z
      .object({
        width: z.number(),
        height: z.number(),
        fps: z.number(),
        codec: videoCodec,
        videoBitrateKbps: z.number(),
        audioBitrateKbps: z.number(),
        easing,
        animateAt60: z.boolean(),
        motionBlur: z.boolean(),
        smoothCursor: z.boolean(),
        frameBlend: z.boolean(),
      })
      .strict(),
    ai: z
      .object({
        provider: z.string(),
        model: z.string(),
        topic: z.string(),
        language: z.string(),
        rawDraft: z.string(),
        beats: z.array(
          z
            .object({
              kind: beatKind,
              start: z.number(),
              text: z.string(),
              cues: z.array(z.string()),
            })
            .strict(),
        ),
      })
      .strict(),
    sync: z
      .object({
        screenOffsetMs: z.number(),
        camOffsetMs: z.number(),
        micOffsetMs: z.number(),
      })
      .strict(),
  })
  .strict();
