export const EXPORT_PRESETS = [
  { id: "4K", width: 2160, height: 3840, bitrateKbps: 40000 },
  { id: "1440p", width: 1440, height: 2560, bitrateKbps: 16000 },
  { id: "1080p", width: 1080, height: 1920, bitrateKbps: 8000 },
] as const;

export type ExportPresetId = (typeof EXPORT_PRESETS)[number]["id"];

export function presetById(id: string) {
  return EXPORT_PRESETS.find((preset) => preset.id === id) ?? EXPORT_PRESETS[0];
}

export function etaSeconds(
  done: number,
  total: number,
  elapsed: number,
): number {
  if (done <= 0 || done >= total) {
    return 0;
  }
  return (elapsed / done) * (total - done);
}

export function sizeLabel(bitrateKbps: number, seconds: number): string {
  const megabytes = (bitrateKbps * seconds) / 8 / 1000;
  return `about ${Math.round(megabytes)} MB`;
}
