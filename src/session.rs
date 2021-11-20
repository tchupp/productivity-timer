use crate::pt_duration::PTDuration;
use crate::daemon::format_instant_to_hhmmss;
use crate::database;
use core::time::Duration;
use crate::analytics::Analytics;
use std::convert::TryInto;

// TODO: only private data fields, add getters/setters
#[derive(Debug)]
pub struct Session {
    id: u64,
    durations: Vec<PTDuration>,
    additions: Vec<PTDuration>,
    pub active: bool,
    pub analytics: Analytics,
}

impl Session {
    pub fn new() -> Session {
        // save new database session
        // get id from database session
        Session {
            id: 1234, // get id from database saving
            durations: Vec::new(),
            additions: Vec::new(),
            active: false,
            analytics: Analytics::new(),
        }
    }

    // TODO: DRY up record_time and record_additions
    pub fn record_time(&mut self, tag: Option<String>) {
        self.durations.push(PTDuration::new(tag));
        self.active = true;
    }

    pub fn record_addition(&mut self, minutes_to_add: u64) {
        // TODO: support tags
        let mut pt_duration = PTDuration::new(None);
        let addition = Duration::new(minutes_to_add * 60, 0);
        pt_duration.update_time_gained(addition);

        self.additions.push(pt_duration);
    }

    pub fn pause(&mut self) {
        let active_duration = self.durations.last_mut().unwrap();
        active_duration.end();
        active_duration.time_gained = active_duration
            .end
            .unwrap()
            .checked_duration_since(active_duration.begin);

        self.analytics.update_duration_count();
        self.analytics.update_duration_avg();
        self.active = false;
    }

    pub fn update_time_gained(&mut self) {
        if self.durations.len() != 0 {
            self.analytics
                .update_time_gained(&self.durations, &self.additions);
        }
    }

    pub fn save_session(self) {
        let formatted_time_gained = match self.analytics.time_gained {
            Some(v) => format_instant_to_hhmmss(v),
            None => "00:00:00".to_string(),
        };

        let duration_avg = match self.analytics.duration_avg {
            Some(v) => v.to_string(),
            None => 0.to_string(),
        };

        database::save_time_gained(
            formatted_time_gained,
            self.analytics.duration_count.unwrap().try_into().unwrap(),
            duration_avg,
        )
        .unwrap();
    }
}
