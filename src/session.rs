use crate::analytics::Analytics;
use crate::daemon::format_instant_to_hhmmss;
use crate::database;
use crate::pt_duration::PTDuration;
use core::time::Duration;
use std::convert::TryInto;
use std::io::Error;
use std::time::Instant;

// TODO: only private data fields, add getters/setters
#[derive(Debug)]
pub struct Session {
    // TODO only pub to debug daemon
    pub id: u64,
    durations: Vec<PTDuration>,
    additions: Vec<PTDuration>,
    subtractions: Vec<PTDuration>,
    pub active: bool,
    pub analytics: Analytics,
    pub tag: Option<String>,
}

impl Session {
    pub fn new() -> Session {
        let id = database::new_session().unwrap();
        println!("id from new session: {}", id);

        Session {
            id,
            durations: Vec::new(),
            additions: Vec::new(),
            subtractions: Vec::new(),
            active: false,
            analytics: Analytics::new(),
            tag: None,
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

    pub fn record_subtraction(&mut self, minutes_to_subtract: u64) {
        // TODO: support tags
        let mut pt_duration = PTDuration::new(None);
        let subtraction = Duration::new(minutes_to_subtract * 60, 0);
        pt_duration.update_time_gained(subtraction);

        self.subtractions.push(pt_duration);
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
                .update_time_gained(&self.durations, &self.additions, &self.subtractions);
        }
    }

    pub fn save_session(self) {
        // This includes additions and subtractions via analytics
        let formatted_time_gained = match self.analytics.time_gained {
            Some(v) => format_instant_to_hhmmss(v),
            None => "00:00:00".to_string(),
        };

        let duration_avg = match self.analytics.duration_avg {
            Some(v) => v.to_string(),
            None => 0.to_string(),
        };

        database::save_session(
            formatted_time_gained,
            self.analytics.duration_count.unwrap().try_into().unwrap(),
            duration_avg,
            self.id,
            self.tag.unwrap(),
        )
        .unwrap();

        // TODO: this only accounts for 'natural' durations, not additions or subtractions, but it
        // makes sense to take tags for, them, too. Expand this to cover them, which will require
        // supporting tags for adds/subs
        for duration in self.durations {
            database::save_tag(
                self.id,
                duration.tag.unwrap(),
                format_instant_to_hhmmss(duration.time_gained.unwrap()),
            )
            .expect("Error saving tag");
        }
    }

    pub fn get_tag_time_gained(&self, tag: String) -> Result<String, Error> {
        let time_gained_for_tag: Duration = self
            .durations
            .iter()
            .map(|duration| {
                if duration.tag.as_ref().unwrap().to_string() == tag {
                    match duration.time_gained {
                        Some(time_gained) => time_gained,
                        None => Instant::now()
                            .checked_duration_since(duration.begin)
                            .unwrap(),
                    }
                } else {
                    Duration::new(0, 0)
                }
            })
            .sum();

        Ok(format_instant_to_hhmmss(time_gained_for_tag))
    }
}
