import { produce } from "immer";

export type Span = {
  id: string;
  startMs: number;
  durationMs: number;
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
