// Matches config/ai.toml script.words_per_minute.
export const WORDS_PER_MINUTE = 140;

export function wordCount(text: string): number {
  const trimmed = text.trim();
  if (trimmed === "") {
    return 0;
  }
  return trimmed.split(/\s+/).length;
}

export function estimateSeconds(text: string, wordsPerMinute: number): number {
  if (!Number.isFinite(wordsPerMinute) || wordsPerMinute <= 0) {
    return 0;
  }
  return (wordCount(text) / wordsPerMinute) * 60;
}

export function spokenText(lines: string[]): string {
  return lines.join(" ");
}
