// Mirrors crates/core/src/layout.rs. Keep the numbers together.

export const MAX_CROP_STEP_PX = 48;
export const CROP_OMEGA = 6;
export const CAPTION_LINE_CHARS = 28;

export type Rect = { x: number; y: number; width: number; height: number };
export type CropFollow = { x: number; y: number; vx: number; vy: number };
export type CaptionPage = {
  lines: string[][];
  activeLine: number;
  activeIndex: number;
};

const POSITIONS = [
  "Top left",
  "Top",
  "Top right",
  "Left",
  "Center",
  "Right",
  "Bottom left",
  "Bottom",
  "Bottom right",
] as const;

export type CamSlot = (typeof POSITIONS)[number];

export function shortsInsets(frameW: number, frameH: number) {
  return {
    top: frameH * 0.08,
    right: frameW * 0.16,
    bottom: frameH * 0.18,
    left: frameW * 0.04,
  };
}

export function safeRect(frameW: number, frameH: number): Rect {
  const insets = shortsInsets(frameW, frameH);
  return {
    x: insets.left,
    y: insets.top,
    width: Math.max(1, frameW - insets.left - insets.right),
    height: Math.max(1, frameH - insets.top - insets.bottom),
  };
}

function frameMargin(frameW: number, frameH: number): Rect {
  const margin = Math.max(8, frameW * 0.02);
  return {
    x: margin,
    y: margin,
    width: Math.max(1, frameW - margin * 2),
    height: Math.max(1, frameH - margin * 2),
  };
}

export function camSide(size: "S" | "M" | "L", frameW: number) {
  const px = size === "S" ? 140 : size === "M" ? 200 : 280;
  return (px * frameW) / 1080;
}

function anchor(slot: CamSlot): [number, number] {
  const column = POSITIONS.indexOf(slot) % 3;
  const row = Math.floor(POSITIONS.indexOf(slot) / 3);
  return [column / 2, row / 2];
}

export function camRect(
  frameW: number,
  frameH: number,
  slot: CamSlot,
  size: "S" | "M" | "L",
  safe: boolean,
): Rect {
  const bounds = safe ? safeRect(frameW, frameH) : frameMargin(frameW, frameH);
  const side = Math.min(camSide(size, frameW), bounds.width, bounds.height);
  const [ax, ay] = anchor(slot);
  return {
    x: bounds.x + ax * (bounds.width - side),
    y: bounds.y + ay * (bounds.height - side),
    width: side,
    height: side,
  };
}

export function captionRect(
  frameW: number,
  frameH: number,
  lineCount: number,
  safe: boolean,
): Rect {
  const bounds = safe ? safeRect(frameW, frameH) : frameMargin(frameW, frameH);
  const lines = Math.min(2, Math.max(1, lineCount));
  const height = Math.min(frameH * 0.045 * lines, bounds.height);
  return {
    x: bounds.x,
    y: bounds.y + bounds.height - height,
    width: bounds.width,
    height,
  };
}

export function cropSize(sourceW: number, sourceH: number, zoom: number) {
  const scale = Math.max(1, zoom);
  let height = sourceH / scale;
  let width = (height * 9) / 16;
  if (width > sourceW) {
    width = sourceW / scale;
    height = (width * 16) / 9;
  }
  return { width: Math.max(1, width), height: Math.max(1, height) };
}

function clampRect(rect: Rect, sourceW: number, sourceH: number): Rect {
  const width = Math.min(Math.max(1, rect.width), sourceW);
  const height = Math.min(Math.max(1, rect.height), sourceH);
  return {
    x: Math.min(Math.max(0, rect.x), Math.max(0, sourceW - width)),
    y: Math.min(Math.max(0, rect.y), Math.max(0, sourceH - height)),
    width,
    height,
  };
}

export function cropTarget(
  cursorX: number,
  cursorY: number,
  sourceW: number,
  sourceH: number,
  zoom: number,
): Rect {
  const size = cropSize(sourceW, sourceH, zoom);
  return clampRect(
    {
      x: cursorX - size.width / 2,
      y: cursorY - size.height / 2,
      width: size.width,
      height: size.height,
    },
    sourceW,
    sourceH,
  );
}

function damp(position: number, velocity: number, target: number, dt: number) {
  const accel =
    CROP_OMEGA * CROP_OMEGA * (target - position) - 2 * CROP_OMEGA * velocity;
  const nextVelocity = velocity + accel * dt;
  return { position: position + nextVelocity * dt, velocity: nextVelocity };
}

export function stepCrop(
  state: CropFollow,
  target: Rect,
  sourceW: number,
  sourceH: number,
  dt: number,
): { follow: CropFollow; rect: Rect } {
  const step = Math.max(dt, 1 / 240);
  const x = damp(state.x, state.vx, target.x, step);
  const y = damp(state.y, state.vy, target.y, step);
  const dx = x.position - state.x;
  const dy = y.position - state.y;
  const distance = Math.hypot(dx, dy);
  const limited =
    distance > MAX_CROP_STEP_PX
      ? {
          x: state.x + (dx * MAX_CROP_STEP_PX) / distance,
          y: state.y + (dy * MAX_CROP_STEP_PX) / distance,
        }
      : { x: x.position, y: y.position };
  const rect = clampRect(
    {
      x: limited.x,
      y: limited.y,
      width: target.width,
      height: target.height,
    },
    sourceW,
    sourceH,
  );
  return {
    follow: {
      x: rect.x,
      y: rect.y,
      vx: (rect.x - state.x) / step,
      vy: (rect.y - state.y) / step,
    },
    rect,
  };
}

function packLines(words: string[], maxChars: number) {
  const limit = Math.max(1, maxChars);
  const lines: string[][] = [];
  let current: string[] = [];
  let used = 0;
  for (const word of words) {
    const len = Math.max(1, [...word].length);
    if (current.length === 0 || used + 1 + len <= limit) {
      if (current.length > 0) {
        used += 1;
      }
      used += len;
      current.push(word);
    } else {
      lines.push(current);
      current = [word];
      used = len;
    }
  }
  if (current.length > 0) {
    lines.push(current);
  }
  return lines;
}

export function captionPage(
  words: string[],
  active: number,
  maxChars = CAPTION_LINE_CHARS,
): CaptionPage {
  if (words.length === 0) {
    return { lines: [], activeLine: 0, activeIndex: 0 };
  }
  const index = Math.min(active, words.length - 1);
  let start = index;
  while (
    start > 0 &&
    packLines(words.slice(start - 1, index + 1), maxChars).length <= 2
  ) {
    start -= 1;
  }
  let end = index;
  while (
    end + 1 < words.length &&
    packLines(words.slice(start, end + 2), maxChars).length <= 2
  ) {
    end += 1;
  }
  const lines = packLines(words.slice(start, end + 1), maxChars);
  let seen = start;
  let activeLine = 0;
  let activeIndex = 0;
  lines.forEach((line, lineIndex) => {
    line.forEach((_, wordIndex) => {
      if (seen === index) {
        activeLine = lineIndex;
        activeIndex = wordIndex;
      }
      seen += 1;
    });
  });
  return { lines, activeLine, activeIndex };
}

export function zoomAt(keys: Array<[number, number]>, time: number) {
  const first = keys[0];
  if (!first) {
    return 1;
  }
  if (time <= first[0]) {
    return Math.max(1, first[1]);
  }
  for (let i = 0; i < keys.length - 1; i += 1) {
    const [t0, z0] = keys[i];
    const [t1, z1] = keys[i + 1];
    if (time <= t1) {
      const span = t1 - t0;
      const t = span === 0 ? 1 : (time - t0) / span;
      return Math.max(1, z0 + (z1 - z0) * t);
    }
  }
  return Math.max(1, keys[keys.length - 1][1]);
}
