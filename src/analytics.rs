use crate::daemon::format_instant_to_hhmmss;
use crate::pt_duration::PTDuration;
use std::time::{Duration, Instant};

// TODO: make data fields private, add getters/setters
#[derive(Debug)]
pub struct Analytics {
    pub time_gained: Option<Duration>,
    // TODO: probably should be a Duration, but going for the visual stuff before the true analytic
    // stuff
    pub duration_avg: Option<String>,
    pub duration_count: Option<u64>,
}

impl Analytics {
    pub fn new() -> Analytics {
        Analytics {
            time_gained: None,
            duration_avg: None,
            duration_count: None,
        }
    }

    pub fn update_time_gained(
        &mut self,
        durations: &Vec<PTDuration>,
        additions: &Vec<PTDuration>,
        subtractions: &Vec<PTDuration>,
    ) {
        let mut durations_time_gained: Vec<Duration> = durations
            .iter()
            .map(|duration| match duration.time_gained {
                Some(time_gained) => time_gained,
                None => Instant::now()
                    .checked_duration_since(duration.begin)
                    .unwrap(),
            })
            .collect();

        let additions_time_gained: Vec<Duration> = additions
            .iter()
            .map(|duration| match duration.time_gained {
                Some(time_gained) => time_gained,
                None => panic!("additions.time_gained failed to find a time_gained"),
            })
            .collect();

        // what happens if no subtractions or additions? empty array?
        let subtractions_time_gained: Vec<Duration> = subtractions
            .iter()
            .map(|duration| match duration.time_gained {
                Some(time_gained) => time_gained,
                None => panic!("subtractions.time_gained failed to find a time_gained"),
            })
            .collect();

        let subtractions = subtractions_time_gained.iter().sum();

        durations_time_gained.extend(additions_time_gained);
        let time_gained: Duration = durations_time_gained.iter().sum();
        let time_gained = time_gained.checked_sub(subtractions).unwrap();

        self.time_gained = Some(time_gained);
    }

    pub fn get_time_gained_formatted(&self) -> String {
        match self.time_gained {
            Some(v) => format_instant_to_hhmmss(v),
            None => "00:00:00".to_string(),
        }
    }

    pub fn update_duration_count(&mut self) {
        let duration_count = match self.duration_count {
            Some(v) => v,
            None => 0,
        };

        self.duration_count = Some(duration_count + 1);
    }

    pub fn update_duration_avg(&mut self) {
        if self.time_gained == None {
            return;
        }

        let avg_seconds_raw =
            (self.time_gained.unwrap().as_secs() / self.duration_count.unwrap()) % 60;
        let avg_minutes_raw =
            ((self.time_gained.unwrap().as_secs() / self.duration_count.unwrap()) / 60) % 60;
        let avg_hours_raw =
            ((self.time_gained.unwrap().as_secs() / self.duration_count.unwrap()) / 60) / 60;

        let avg_seconds: String;

        if avg_seconds_raw < 10 {
            avg_seconds = "0".to_owned() + &avg_seconds_raw.to_string();
        } else {
            avg_seconds = avg_seconds_raw.to_string();
        }

        let avg_minutes: String;
        if avg_minutes_raw < 10 {
            avg_minutes = "0".to_owned() + &avg_minutes_raw.to_string();
        } else {
            avg_minutes = avg_minutes_raw.to_string();
        }

        let avg_hours: String;
        if avg_hours_raw < 10 {
            avg_hours = "0".to_owned() + &avg_hours_raw.to_string();
        } else {
            avg_hours = avg_hours_raw.to_string();
        }

        self.duration_avg = Some(format!("{}:{}:{}", avg_hours, avg_minutes, avg_seconds));
    }
}
