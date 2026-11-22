/** Same numbers as `core::prompter`. The window scrolls from elapsed time. */

export const FOLLOW_CONFIDENCE = 0.6;

export type ScrollMode = "follow" | "fixed" | "paused";

export function scrollOffsetPx(
  elapsedMs: number,
  pausedMs: number,
  pxPerSec: number,
): number {
  const active = Math.max(0, elapsedMs - pausedMs);
  return (pxPerSec * active) / 1000;
}

export function nudgeOffset(offsetPx: number, deltaPx: number): number {
  return Math.max(0, offsetPx + deltaPx);
}

export function scrollMode(confidence: number, silent: boolean): ScrollMode {
  if (silent) {
    return "paused";
  }
  if (confidence >= FOLLOW_CONFIDENCE) {
    return "follow";
  }
  return "fixed";
}

export function followOffsetPx(
  lineIndex: number,
  lineHeightPx: number,
): number {
  return lineIndex * lineHeightPx;
}

export function fuzzyMatch(
  lines: string[],
  heard: string,
): { index: number; confidence: number } {
  const heardTokens = tokens(heard);
  if (heardTokens.length === 0 || lines.length === 0) {
    return { index: 0, confidence: 0 };
  }
  let best = { index: 0, confidence: 0 };
  lines.forEach((line, index) => {
    const lineTokens = tokens(line);
    if (lineTokens.length === 0) {
      return;
    }
    const shared = heardTokens.filter((token) =>
      lineTokens.includes(token),
    ).length;
    const confidence = shared / heardTokens.length;
    if (confidence > best.confidence) {
      best = { index, confidence };
    }
  });
  return best;
}

function tokens(text: string): string[] {
  return text
    .split(/[^0-9A-Za-z]+/)
    .filter((token) => token.length > 0)
    .map((token) => token.toLowerCase());
}
