//! Clip ordering on one track. Times are integer milliseconds so the checks are exact.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineSpan {
    pub id: String,
    pub start_ms: i64,
    pub duration_ms: u64,
}

pub fn normalize_spans(mut spans: Vec<TimelineSpan>) -> Vec<TimelineSpan> {
    spans.sort_by_key(|span| span.start_ms);
    let mut cursor: Option<i64> = None;
    for span in &mut spans {
        if let Some(occupied) = cursor
            && span.start_ms < occupied
        {
            span.start_ms = occupied;
        }
        let duration = i64::try_from(span.duration_ms).unwrap_or(i64::MAX);
        cursor = Some(span.start_ms.saturating_add(duration));
    }
    spans
}

pub fn spans_are_ordered(spans: &[TimelineSpan]) -> bool {
    spans.windows(2).all(|pair| {
        let duration = i64::try_from(pair[0].duration_ms).unwrap_or(i64::MAX);
        pair[0].start_ms <= pair[1].start_ms
            && pair[1].start_ms >= pair[0].start_ms.saturating_add(duration)
    })
}

#[cfg(test)]
mod tests {
    use super::{TimelineSpan, normalize_spans, spans_are_ordered};
    use proptest::prelude::*;

    fn span(id: &str, start_ms: i64, duration_ms: u64) -> TimelineSpan {
        TimelineSpan {
            id: id.to_string(),
            start_ms,
            duration_ms,
        }
    }

    fn signature(spans: &[TimelineSpan]) -> Vec<(String, u64)> {
        let mut items: Vec<_> = spans
            .iter()
            .map(|span| (span.id.clone(), span.duration_ms))
            .collect();
        items.sort();
        items
    }

    #[test]
    fn overlap_is_pushed_forward_without_reordering_equal_starts() {
        let ordered = normalize_spans(vec![span("b", 0, 10), span("a", 0, 10), span("c", 50, 10)]);
        assert_eq!(ordered[0].id, "b");
        assert_eq!(ordered[0].start_ms, 0);
        assert_eq!(ordered[1].id, "a");
        assert_eq!(ordered[1].start_ms, 10);
        assert_eq!(ordered[2].id, "c");
        assert_eq!(ordered[2].start_ms, 50);
        assert!(spans_are_ordered(&ordered));
    }

    #[test]
    fn a_gap_is_left_alone() {
        let ordered = normalize_spans(vec![span("a", 0, 10), span("b", 40, 5)]);
        assert_eq!(ordered[1].start_ms, 40);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]
        #[test]
        fn normalize_keeps_durations_and_stays_ordered(
            raw in prop::collection::vec(span_strategy(), 0..16)
        ) {
            let before = signature(&raw);
            let ordered = normalize_spans(raw);
            prop_assert_eq!(signature(&ordered), before);
            prop_assert!(spans_are_ordered(&ordered));
            prop_assert_eq!(ordered.clone(), normalize_spans(ordered));
        }
    }

    fn span_strategy() -> impl Strategy<Value = TimelineSpan> {
        ("[a-z]{1,4}", 0i64..20_000, 1u64..5_000).prop_map(|(id, start_ms, duration_ms)| {
            TimelineSpan {
                id,
                start_ms,
                duration_ms,
            }
        })
    }
}
