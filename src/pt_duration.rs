use std::time::{Duration, Instant};

// TODO: only private data fields; getters/setters for updating
#[derive(Debug)]
pub struct PTDuration {
    pub tag: Option<String>,
    pub time_gained: Option<Duration>,
    pub begin: Instant,
    pub end: Option<Instant>,
}

impl PTDuration {
    pub fn new(tag: Option<String>) -> PTDuration {
        PTDuration {
            tag,
            time_gained: None,
            begin: Instant::now(),
            end: None,
        }
    }

    pub fn update_time_gained(&mut self, time_gained: Duration) {
        self.time_gained = Some(time_gained);
    }

    pub fn end(&mut self) {
        self.end = Some(Instant::now());
    }
}
