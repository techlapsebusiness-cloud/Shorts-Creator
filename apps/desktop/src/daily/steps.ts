export type Actor = "AI" | "YOU" | "AUTO";

export type DailyStep = {
  id: string;
  label: string;
  actor: Actor;
};

export const DAILY_STEPS: DailyStep[] = [
  { id: "topic", label: "Pick a topic", actor: "AI" },
  { id: "script", label: "Generate the script", actor: "AI" },
  { id: "review", label: "Review the script", actor: "YOU" },
  { id: "record", label: "Record with the prompter", actor: "YOU" },
  { id: "edit", label: "Cut fillers, captions, zoom, ducking", actor: "AUTO" },
  { id: "assets", label: "Cover and end card", actor: "AUTO" },
  { id: "export", label: "Export 4K", actor: "AUTO" },
  { id: "kit", label: "Write the upload kit", actor: "AUTO" },
];

export function advance(done: string[]): string[] {
  const start = DAILY_STEPS.findIndex((step) => !done.includes(step.id));
  if (start < 0) {
    return done;
  }
  const next = [...done];
  for (let index = start; index < DAILY_STEPS.length; index += 1) {
    next.push(DAILY_STEPS[index].id);
    if (DAILY_STEPS[index].actor === "YOU") {
      break;
    }
  }
  return next;
}

export function pausedStep(done: string[]): DailyStep | null {
  return (
    DAILY_STEPS.find(
      (step) =>
        step.actor === "YOU" &&
        done.includes(step.id) &&
        !done.includes(nextId(step.id)),
    ) ?? null
  );
}

function nextId(id: string): string {
  const index = DAILY_STEPS.findIndex((step) => step.id === id);
  return DAILY_STEPS[index + 1]?.id ?? "";
}

export function explainFault(message: string): string | null {
  const lower = message.toLowerCase();
  if (lower.includes("ffmpeg")) {
    return "ffmpeg is not installed. Install it, then run just doctor.";
  }
  if (lower.includes("permission") || lower.includes("denied")) {
    return "Screen or microphone permission was denied.";
  }
  if (lower.includes("disk") || lower.includes("no space")) {
    return "The disk does not have enough free space for this take.";
  }
  return null;
}
