import { produce } from "immer";

export type Span = {
  id: string;
  startMs: number;
  durationMs: number;
  speed?: number;
};

export type History<T> = {
  past: T[];
  present: T;
  future: T[];
};

export const UNDO_LIMIT = 100;

export function splitSpan(spans: Span[], id: string, atMs: number): Span[] {
  const index = spans.findIndex((span) => span.id === id);
  if (index < 0) {
    return spans;
  }
  const span = spans[index];
  if (atMs <= span.startMs || atMs >= span.startMs + span.durationMs) {
    return spans;
  }
  const local = atMs - span.startMs;
  return produce(spans, (draft) => {
    draft.splice(
      index,
      1,
      { id: `${id}-a`, startMs: span.startMs, durationMs: local },
      { id: `${id}-b`, startMs: atMs, durationMs: span.durationMs - local },
    );
  });
}

export function trimSpan(
  spans: Span[],
  id: string,
  durationMs: number,
): Span[] {
  if (durationMs <= 0) {
    return spans;
  }
  const span = spans.find((item) => item.id === id);
  if (!span) {
    return spans;
  }
  const end = span.startMs + durationMs;
  const overlaps = spans.some((other) => {
    if (other.id === id) {
      return false;
    }
    const otherEnd = other.startMs + other.durationMs;
    return span.startMs < otherEnd && other.startMs < end;
  });
  if (overlaps) {
    return spans;
  }
  return produce(spans, (draft) => {
    const target = draft.find((item) => item.id === id);
    if (target) {
      target.durationMs = durationMs;
    }
  });
}

export function setSpeed(spans: Span[], id: string, speed: number): Span[] {
  const clamped = clampSpeed(speed);
  return produce(spans, (draft) => {
    const target = draft.find((item) => item.id === id);
    if (target) {
      target.speed = clamped;
    }
  });
}

export function clampSpeed(speed: number) {
  return Math.min(4, Math.max(0.25, speed));
}

export function snapTime(
  timeMs: number,
  edgesMs: number[],
  thresholdMs: number,
) {
  let best = timeMs;
  let distance = thresholdMs + 1;
  for (const edge of edgesMs) {
    const next = Math.abs(edge - timeMs);
    if (next <= thresholdMs && next < distance) {
      best = edge;
      distance = next;
    }
  }
  return best;
}

export function waveformPeaks(samples: number[], buckets: number) {
  if (buckets <= 0) {
    return [];
  }
  const peaks = Array.from({ length: buckets }, () => 0);
  if (samples.length === 0) {
    return peaks;
  }
  samples.forEach((sample, index) => {
    const bucket = Math.min(
      buckets - 1,
      Math.floor((index * buckets) / samples.length),
    );
    peaks[bucket] = Math.max(peaks[bucket], Math.abs(sample));
  });
  return peaks;
}

export function proxyFileName(source: string) {
  const dot = source.lastIndexOf(".");
  const stem = dot > 0 ? source.slice(0, dot) : source;
  return `${stem}.720p.mp4`;
}

export type Marker = { id: string; timeMs: number };

export const TRANSITIONS = ["cut", "fade", "wipe", "glitch"] as const;
export type TransitionName = (typeof TRANSITIONS)[number];

export function nextTransition(current: TransitionName): TransitionName {
  const index = TRANSITIONS.indexOf(current);
  return TRANSITIONS[(index + 1) % TRANSITIONS.length];
}

export function rippleDelete(spans: Span[], id: string): Span[] {
  const span = spans.find((item) => item.id === id);
  if (!span) {
    return spans;
  }
  const end = span.startMs + span.durationMs;
  return produce(
    spans.filter((item) => item.id !== id),
    (draft) => {
      for (const item of draft) {
        if (item.startMs >= end) {
          item.startMs -= span.durationMs;
        }
      }
    },
  );
}

export function historyOf<T>(present: T): History<T> {
  return { past: [], present, future: [] };
}

export function pushHistory<T>(
  history: History<T>,
  next: T,
  limit = UNDO_LIMIT,
): History<T> {
  return {
    past: [...history.past, history.present].slice(-limit),
    present: next,
    future: [],
  };
}

export function undoHistory<T>(history: History<T>): History<T> {
  const previous = history.past.at(-1);
  if (previous === undefined) {
    return history;
  }
  return {
    past: history.past.slice(0, -1),
    present: previous,
    future: [...history.future, history.present],
  };
}

export function redoHistory<T>(history: History<T>): History<T> {
  const next = history.future.at(-1);
  if (next === undefined) {
    return history;
  }
  return {
    past: [...history.past, history.present],
    present: next,
    future: history.future.slice(0, -1),
  };
}
