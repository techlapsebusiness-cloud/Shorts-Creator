use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Autosave {
    interval: Duration,
    dirty_since: Option<Instant>,
}

impl Autosave {
    pub const INTERVAL: Duration = Duration::from_secs(5);

    pub fn new() -> Self {
        Self {
            interval: Self::INTERVAL,
            dirty_since: None,
        }
    }

    pub fn mark_dirty(&mut self, now: Instant) {
        if self.dirty_since.is_none() {
            self.dirty_since = Some(now);
        }
    }

    pub fn due(&self, now: Instant) -> bool {
        self.dirty_since
            .is_some_and(|since| now.saturating_duration_since(since) >= self.interval)
    }

    pub fn saved(&mut self) {
        self.dirty_since = None;
    }
}

impl Default for Autosave {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Autosave;
    use std::time::{Duration, Instant};

    #[test]
    fn saves_five_seconds_after_the_first_edit() {
        let start = Instant::now();
        let mut autosave = Autosave::new();
        assert!(!autosave.due(start));
        autosave.mark_dirty(start);
        autosave.mark_dirty(start + Duration::from_secs(1));
        assert!(!autosave.due(start + Duration::from_secs(4)));
        assert!(autosave.due(start + Duration::from_secs(5)));
        autosave.saved();
        assert!(!autosave.due(start + Duration::from_secs(9)));
    }
}
