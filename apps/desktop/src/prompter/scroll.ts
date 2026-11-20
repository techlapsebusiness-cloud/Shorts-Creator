export const COUNTDOWN_MS = 3000;
export const NUDGE_PX = 48;

export function countdownLabel(elapsedMs: number): number | null {
  if (!Number.isFinite(elapsedMs) || elapsedMs < 0) {
    return 3;
  }
  if (elapsedMs >= COUNTDOWN_MS) {
    return null;
  }
  return 3 - Math.floor(elapsedMs / 1000);
}

export function scrollOffsetPx(input: {
  nowMs: number;
  startMs: number;
  pausedMs: number;
  wordsPerMinute: number;
  textSize: number;
  nudgePx: number;
}): number {
  if (!Number.isFinite(input.wordsPerMinute) || input.wordsPerMinute <= 0) {
    return 0;
  }
  if (!Number.isFinite(input.textSize) || input.textSize <= 0) {
    return 0;
  }
  if (input.nowMs < input.startMs) {
    return 0;
  }
  const playing = Math.max(0, input.nowMs - input.startMs - input.pausedMs);
  const words = (playing / 1000) * (input.wordsPerMinute / 60);
  return words * input.textSize * 0.5 + input.nudgePx;
}
