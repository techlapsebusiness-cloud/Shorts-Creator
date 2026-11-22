export const MAX_DURATION_SECS = 60;
export const POLICY_CHECKED_ON = "2026-11-22";
export const TRIGRAM_WARN = 0.4;
export const CHANGE_WARN = 0.2;

export type CheckState = "pass" | "fail" | "warn";

export type Check = {
  id: string;
  label: string;
  state: CheckState;
  detail: string;
};

export type PastShort = {
  topic: string;
  title: string;
  layoutKey: string;
};

export type ShotCheck = {
  width: number;
  height: number;
  durationSecs: number;
  lufs: number | null;
  truePeakDb: number | null;
  safeZone: boolean;
  musicLicensed: boolean;
  topic: string;
  title: string;
  script: string;
  rawDraft: string;
  layoutKey: string;
  history: PastShort[];
};

export function exampleShot(musicLicensed: boolean): ShotCheck {
  return {
    width: 1080,
    height: 1920,
    durationSecs: 42,
    lufs: -14,
    truePeakDb: -1.4,
    safeZone: true,
    musicLicensed,
    topic: "atomic rename",
    title: "Rename without losing the file",
    script: "The temp name is the trick and the real file appears last.",
    rawDraft: "Just rename the file quickly and hope.",
    layoutKey: "screen-cam|hook",
    history: [
      {
        topic: "camera exposure",
        title: "Expose for the face",
        layoutKey: "cam-only|error",
      },
    ],
  };
}

export function preflight(shot: ShotCheck): Check[] {
  const ratio = shot.height / Math.max(shot.width, 1);
  const checks: Check[] = [
    mark(
      "aspect",
      "Vertical 9:16",
      Math.abs(ratio - 16 / 9) < 0.02,
      `${shot.width}×${shot.height}`,
    ),
    mark(
      "length",
      `Under ${MAX_DURATION_SECS} seconds`,
      shot.durationSecs <= MAX_DURATION_SECS,
      `${Math.round(shot.durationSecs)}s`,
    ),
    loudness(shot),
    mark(
      "safe",
      "Captions and camera inside the safe zone",
      shot.safeZone,
      shot.safeZone ? "Safe zone on" : "Safe zone off",
    ),
    shot.musicLicensed
      ? mark("licence", "Music licence", true, "Ticked")
      : {
          id: "licence",
          label: "Music licence",
          state: "warn",
          detail: "Tick this. It does not block export.",
        },
  ];
  checks.push(...originality(shot));
  return checks;
}

export function blocks(): boolean {
  return false;
}

function loudness(shot: ShotCheck): Check {
  if (shot.lufs === null || shot.truePeakDb === null) {
    return {
      id: "loudness",
      label: "Loudness -14 LUFS",
      state: "warn",
      detail: "Not measured yet",
    };
  }
  const ok = Math.abs(shot.lufs - -14) <= 0.5 && shot.truePeakDb <= -1;
  return mark(
    "loudness",
    "Loudness -14 LUFS",
    ok,
    `${shot.lufs.toFixed(1)} LUFS`,
  );
}

function originality(shot: ShotCheck): Check[] {
  const recent = shot.history.slice(-10);
  const similar = recent.some(
    (past) =>
      jaccard(shot.topic, past.topic) >= TRIGRAM_WARN ||
      jaccard(shot.title, past.title) >= TRIGRAM_WARN,
  );
  const changed = changeRatio(shot.rawDraft, shot.script);
  const repeated = recent.some((past) => past.layoutKey === shot.layoutKey);
  return [
    similar
      ? {
          id: "similar",
          label: "Topic and title",
          state: "warn",
          detail: "Close to one of the last 10 Shorts.",
        }
      : mark("similar", "Topic and title", true, "Distinct from the last 10"),
    changed < CHANGE_WARN
      ? {
          id: "draft",
          label: "Script vs the raw draft",
          state: "warn",
          detail: `${Math.round(changed * 100)}% changed. Under 20% is a warning.`,
        }
      : mark(
          "draft",
          "Script vs the raw draft",
          true,
          `${Math.round(changed * 100)}% changed`,
        ),
    repeated
      ? {
          id: "layout",
          label: "Layout and template",
          state: "warn",
          detail: "Same layout as a recent Short.",
        }
      : mark("layout", "Layout and template", true, "Not a repeat"),
  ];
}

function mark(id: string, label: string, ok: boolean, detail: string): Check {
  return { id, label, state: ok ? "pass" : "fail", detail };
}

function trigrams(text: string): Set<string> {
  const compact = text
    .toLowerCase()
    .replace(/[^a-z0-9\s]/g, "")
    .split(/\s+/)
    .filter(Boolean)
    .join(" ");
  if (compact.length < 3) {
    return new Set([compact]);
  }
  const out = new Set<string>();
  for (let index = 0; index <= compact.length - 3; index += 1) {
    out.add(compact.slice(index, index + 3));
  }
  return out;
}

function jaccard(left: string, right: string): number {
  const a = trigrams(left);
  const b = trigrams(right);
  let inter = 0;
  for (const item of a) {
    if (b.has(item)) {
      inter += 1;
    }
  }
  const union = new Set([...a, ...b]).size;
  return union === 0 ? 0 : inter / union;
}

function changeRatio(draft: string, script: string): number {
  const left = draft.toLowerCase().split(/\s+/).filter(Boolean);
  const right = script.toLowerCase().split(/\s+/).filter(Boolean);
  const denom = Math.max(left.length, right.length, 1);
  return levenshtein(left, right) / denom;
}

function levenshtein(left: string[], right: string[]): number {
  const previous = Array.from(
    { length: right.length + 1 },
    (_, index) => index,
  );
  const current = new Array<number>(right.length + 1).fill(0);
  for (let row = 0; row < left.length; row += 1) {
    current[0] = row + 1;
    for (let col = 0; col < right.length; col += 1) {
      const cost = left[row] === right[col] ? 0 : 1;
      current[col + 1] = Math.min(
        current[col] + 1,
        previous[col + 1] + 1,
        previous[col] + cost,
      );
    }
    for (let index = 0; index < previous.length; index += 1) {
      previous[index] = current[index];
    }
  }
  return previous[right.length];
}
