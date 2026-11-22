export const COVER_W = 1080;
export const COVER_H = 1920;
export const TAGLINE = "BUILD. LEARN. REPEAT.";

export type EndBeat = "chevron" | "next" | "follow" | "tagline";

export function endBeat(time: number): EndBeat {
  if (time < 0.45) {
    return "chevron";
  }
  if (time < 1.05) {
    return "next";
  }
  if (time < 1.8) {
    return "follow";
  }
  return "tagline";
}

export function fitLines(text: string, maxChars: number): string[] {
  const words = text.trim().split(/\s+/).filter(Boolean);
  const lines: string[] = [];
  let current = "";
  for (const word of words) {
    const next = current ? `${current} ${word}` : word;
    if (next.length <= maxChars) {
      current = next;
    } else {
      if (current) {
        lines.push(current);
      }
      current = word.length > maxChars ? word.slice(0, maxChars) : word;
    }
  }
  if (current) {
    lines.push(current);
  }
  return lines.length > 0 ? lines : [""];
}

export function hookFits(text: string): boolean {
  const lines = fitLines(text, 28);
  return lines.length > 0 && lines.every((line) => line.length <= 28);
}

export function nextTitle(queue: string[], current: string): string | null {
  return queue.find((topic) => topic !== current) ?? null;
}
