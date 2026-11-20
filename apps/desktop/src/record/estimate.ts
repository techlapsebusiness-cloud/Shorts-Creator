export const VIDEO_BITRATE_KBPS = 40_000;
export const AUDIO_BITRATE_KBPS = 384;

export function estimateMegabytes(
  durationSeconds: number,
  videoKbps: number,
  audioKbps: number,
  videoTracks: number,
): number {
  if (!Number.isFinite(durationSeconds) || durationSeconds <= 0) {
    return 0;
  }
  const kbps = videoKbps * videoTracks + audioKbps;
  const bytes = (kbps * 1000 * durationSeconds) / 8;
  return bytes / 1_000_000;
}

export function micLevel(samples: number[]): number {
  let peak = 0;
  for (const sample of samples) {
    peak = Math.max(peak, Math.abs(sample));
  }
  return Math.min(1, peak / 32767);
}
