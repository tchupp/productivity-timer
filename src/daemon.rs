use crate::database;
use daemonize::Daemonize;
use dirs::home_dir;
use regex::Regex;
use std::fs::{create_dir, read_to_string, write, File, OpenOptions};
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::process::exit;
use std::thread::sleep;
// TODO: apparently chrono supports negative durations (or some representation of time); it'd
// probably be smart to pull out std::time in favor of that to make -s, --subtract easier
use std::time::{Duration, Instant};

pub fn init() {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";
    let pid_filepath = get_filepath("timer.pid").unwrap();
    let (tmp_file_out, tmp_file_err) = create_files().unwrap();

    let daemonize = Daemonize::new()
        .pid_file(pid_filepath)
        .working_directory(working_directory)
        .stdout(tmp_file_out)
        .stderr(tmp_file_err)
        .exit_action(|| println!("TODO: exiting"));

    match daemonize.start() {
        Ok(_) => listen_for_durations(),
        Err(e) => eprintln!("Error, {}", e),
    }
}

// TODO: convert to struct with a constuctor? Something like Files::new() and maybe Files::clean()
fn create_files() -> Result<(File, File), Error> {
    let in_filepath = get_filepath("in")?;
    let out_filepath = get_filepath("out")?;
    let err_filepath = get_filepath("err")?;
    let time_gained_filepath = get_filepath("time-gained")?;
    let durations_count_filepath = get_filepath("durations-count")?;
    let durations_avg_filepath = get_filepath("durations-average")?;

    create_productivity_timer_dir();
    let tmp_file_out = create_file(&out_filepath, false /*append*/);
    let tmp_file_err = create_file(&err_filepath, false /*append*/);

    // We only need this created, not passed back. We won't use File for
    // the in-file below, but rather the &str constant in_file
    create_file(&in_filepath, false /*append*/);
    create_file(&time_gained_filepath, false /*append*/);
    create_file(&durations_count_filepath, false /*append*/);
    create_file(&durations_avg_filepath, false /*append*/);
    // TODO: decide if I should clean outfile
    reset_in_file().unwrap();

    Ok((tmp_file_out, tmp_file_err))
}

fn create_productivity_timer_dir() {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";

    if !Path::new(&working_directory).exists() {
        match create_dir(working_directory) {
            Ok(_) => (),
            Err(e) => eprintln!("Error, {}", e),
        }
    }
}

fn create_file(file_name: &str, append: bool) -> File {
    if append {
        return OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(file_name)
            .unwrap();
    }

    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_name)
        .unwrap()
}

// TODO: generalize this fn to be the only reader fn (e.g., the misc reader fn)
fn read_from_in_file() -> Result<String, Error> {
    let in_filepath = get_filepath("in")?;
    read_to_string(&in_filepath)
}

#[derive(Debug)]
struct Analytics {
    time_gained: Option<Duration>,
    // TODO: probably should be a Duration, but going for the visual stuff before the true analytic
    // stuff
    duration_avg: Option<String>,
    duration_count: Option<u64>,
}

impl Analytics {
    fn new() -> Analytics {
        Analytics {
            time_gained: None,
            duration_avg: None,
            duration_count: None,
        }
    }

    fn update_time_gained(&mut self, durations: &Vec<PTDuration>) {
        let current_instants: Vec<Duration> = durations
            .iter()
            .map(|duration| match duration.time_gained {
                Some(time_gained) => time_gained,
                None => Instant::now()
                    .checked_duration_since(duration.begin)
                    .unwrap(),
            })
            .collect();

        let time_gained: Duration = current_instants.iter().sum();

        // TODO: handle additions
        self.time_gained = Some(time_gained);
    }

    fn get_time_gained_formatted(&self) -> String {
        match self.time_gained {
            Some(v) => format_instant_to_hhmmss(v),
            None => "00:00:00".to_string(),
        }
    }

    fn update_duration_count(&mut self) {
        // TODO: figure out if unwrapping a None for Option<i32> defaults to 0? Seems to, given no
        // warnings from the compiler? Who knows, hello future Aaron as you finally figure out this
        // is the bug you're struggling with.
        self.duration_count = Some(self.duration_count.unwrap() + 1);
    }

    fn get_duration_count(self) -> u64 {
        self.duration_count.unwrap()
    }

    fn update_duration_avg(&mut self) {
        let avg_seconds_raw =
            (self.time_gained.unwrap().as_secs() / self.duration_count.unwrap()) % 60;
        let avg_minutes_raw =
            ((self.time_gained.unwrap().as_secs() / self.duration_count.unwrap()) / 60) % 60;
        let avg_hours_raw =
            ((self.time_gained.unwrap().as_secs() / self.duration_count.unwrap()) / 60) / 60;

        // TODO: gotta be beautiful
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

    fn get_duration_avg(self) -> String {
        self.duration_avg.unwrap()
    }
}

#[derive(Debug)]
struct Session {
    id: u64,
    durations: Vec<PTDuration>,
    active: bool,
    analytics: Analytics,
}

impl Session {
    fn new() -> Session {
        // save new database session
        // get id from database session
        Session {
            id: 1234, // get id from database saving
            durations: Vec::new(),
            active: false,
            analytics: Analytics::new(),
        }
    }

    fn record_time(&mut self, tag: Option<String>) {
        self.durations.push(PTDuration::new(tag));
        self.active = true;
    }

    fn pause(&mut self) {
        let active_duration = self.durations.last_mut().unwrap();
        active_duration.end();
        active_duration.time_gained = active_duration
            .end
            .unwrap()
            .checked_duration_since(active_duration.begin);

        self.analytics.update_duration_count();

        self.active = false;
    }

    // TODO: impl
    //fn complete(&self) {}

    fn update_time_gained(&mut self) {
        if self.durations.len() != 0 {
            self.analytics.update_time_gained(&self.durations);
        }
    }
}

fn format_instant_to_hhmmss(time_gained: Duration) -> String {
    let seconds_raw = time_gained.as_secs() % 60;
    let minutes_raw = (time_gained.as_secs() / 60) % 60;
    let hours_raw = (time_gained.as_secs() / 60) / 60;

    let seconds: String;
    let minutes: String;
    let hours: String;

    if seconds_raw < 10 {
        seconds = "0".to_owned() + &seconds_raw.to_string();
    } else {
        seconds = seconds_raw.to_string();
    }

    if minutes_raw < 10 {
        minutes = "0".to_owned() + &minutes_raw.to_string();
    } else {
        minutes = minutes_raw.to_string();
    }

    if hours_raw < 10 {
        hours = "0".to_owned() + &hours_raw.to_string();
    } else {
        hours = hours_raw.to_string();
    }

    format!("{}:{}:{}", hours, minutes, seconds)
}

#[derive(Debug)]
struct PTDuration {
    tag: Option<String>,
    time_gained: Option<Duration>,
    begin: Instant,
    end: Option<Instant>,
}

impl PTDuration {
    fn new(tag: Option<String>) -> PTDuration {
        PTDuration {
            tag,
            time_gained: None,
            begin: Instant::now(),
            end: None,
        }
    }

    fn end(&mut self) {
        self.end = Some(Instant::now());
    }
}

fn listen_for_durations() {
    let mut session = Session::new();
    let mut durations: Vec<Instant> = Vec::new();
    let mut additions: Vec<Duration> = Vec::new();

    let half_second = Duration::from_millis(500);

    loop {
        sleep(half_second);

        let input = read_from_in_file().unwrap();
        match input.trim() {
            "e" => exit(0),
            // TODO: find a better way to reset durations on session completions
            "c" => {
                reset_in_file().unwrap();
                durations = Vec::new();
                additions = Vec::new();
                // TODO: clean up the completion logic; it actually sets /time-gained
                // to an empty string and (accidentally) relies on the
                // `checked_write_time_gained_to_file` to set it to 00:00:00
                complete_session();
            }
            "t" => {
                //durations.push(Instant::now()),
                match session.active {
                    // TODO: figure out best way to take in flags for stuff like tags
                    true => session.pause(),
                    false => {
                        let tag = get_tag().unwrap();
                        match tag {
                            Some(tag) => {
                                session.record_time(Some(tag));
                            }
                            None => session.record_time(None),
                        }
                    }
                }
            }
            // TODO: deprecate
            "p" => {
                // TODO: figure out whether there's a perf gain to & instead
                let gained_time = report_time_gained(durations.clone(), additions.clone());
                println!("gained time: {:?}", gained_time);
            }
            "a" => {
                let minutes_to_add: u64 = get_misc().unwrap().parse().unwrap();
                let addition = Duration::new(minutes_to_add * 60, 0);
                additions.push(addition);
                reset_misc().unwrap();
            }
            _ => (),
        }

        session.update_time_gained();
        let time_gained = session.analytics.get_time_gained_formatted();
        set_time_gained(time_gained).expect("Error writing to time-gained file");
        reset_in_file().unwrap();
    }
}

// TODO: make into struct with methods for getting/resetting?
fn reset_misc() -> Result<(), Error> {
    let misc_filepath = get_filepath("misc")?;
    write(misc_filepath, "").expect("Problem writing to misc file");
    Ok(())
}

fn get_misc() -> Result<String, Error> {
    let misc_filepath = get_filepath("misc")?;
    Ok(read_to_string(&misc_filepath)?)
}

fn get_tag() -> Result<Option<String>, Error> {
    let tag_filepath = get_filepath("tag")?;
    let tag = read_to_string(&tag_filepath).unwrap();
    if tag != "" {
        Ok(Some(tag))
    } else {
        Ok(None)
    }
}

pub fn add_minutes(minutes_to_add: String) -> Result<(), Error> {
    let in_filepath = get_filepath("in")?;
    let misc_filepath = get_filepath("misc")?;

    let re = Regex::new(r"^\d+$").unwrap();
    assert!(re.is_match("15"));
    // TODO: better error message/handling if failed regex
    assert!(re.is_match(&minutes_to_add));

    write(in_filepath, "a").expect("Error writing to time gained file");
    write(misc_filepath, minutes_to_add).expect("Error writing to misc file");
    Ok(())
}

fn report_time_gained(durations: Vec<Instant>, additions: Vec<Duration>) -> Duration {
    get_duration_from_vec_of_tupled_instants(convert_vec_to_vec_of_tuples(durations), additions)
}

fn reset_in_file() -> Result<(), Error> {
    let in_filepath = get_filepath("in")?;
    write(in_filepath, "").expect("Error writing to tmp in");
    Ok(())
}

fn zero_out_time_gained_file() -> Result<(), Error> {
    let time_gained_filepath = get_filepath("time-gained")?;
    // TODO: consider writing 00:00:00
    write(time_gained_filepath, "").expect("Error writing to time-gained");
    Ok(())
}

// TODO: consolidate session completion fns and figure out a better way to do it
pub fn trigger_session_completion() -> Result<(), Error> {
    let in_filepath = get_filepath("in")?;
    write(in_filepath, "c").expect("Error writing to /in");
    Ok(())
}

fn complete_session() {
    let time_gained = get_time_gained().unwrap();
    let durations_count = get_durations_count().unwrap();
    let durations_avg = get_durations_avg().unwrap();

    // TODO: error handling
    database::save_time_gained(time_gained, durations_count, durations_avg).unwrap();
    zero_out_time_gained_file().unwrap();
}

fn convert_vec_to_vec_of_tuples(untupled_vec: Vec<Instant>) -> Vec<(Instant, Instant)> {
    if untupled_vec.len() % 2 != 0 {
        panic!("TODO: attempted to print timer before stopping it");
    }
    let mut tupled_vec = Vec::new();
    for (idx, instant) in untupled_vec.iter().enumerate() {
        if idx % 2 == 0 {
            tupled_vec.push((*instant, untupled_vec[idx + 1]));
        }
    }
    tupled_vec
}

fn get_duration_from_vec_of_tupled_instants(
    tupled_vec: Vec<(Instant, Instant)>,
    additions: Vec<Duration>,
) -> Duration {
    // mut for .extend()
    let mut durations_from_tuples: Vec<Duration> = tupled_vec
        .iter()
        .map(|tuple| match tuple.1.checked_duration_since(tuple.0) {
            Some(v) => v,
            None => {
                panic!("TODO: something serious would have gone wrong")
            }
        })
        .collect();

    durations_from_tuples.extend(additions);
    durations_from_tuples.iter().sum()
}

fn get_filepath(filename: &str) -> Result<String, Error> {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer" + "/";

    match filename {
        // TODO: figure out how to dry this up
        "in" => Ok(working_directory + filename),
        "out" => Ok(working_directory + filename),
        "err" => Ok(working_directory + filename),
        "misc" => Ok(working_directory + filename),
        "tag" => Ok(working_directory + filename),
        "timer.pid" => Ok(working_directory + filename),
        "time-gained" => Ok(working_directory + filename),
        "durations-count" => Ok(working_directory + filename),
        "durations-average" => Ok(working_directory + filename),
        _ => Err(Error::new(
            ErrorKind::InvalidInput,
            filename.to_string() + "is not a valid file name",
        )),
    }
}

pub fn trigger_time(tag: Option<String>) -> Result<(), Error> {
    let filepath = get_filepath("in")?;
    let tag_filepath = get_filepath("tag")?;
    write(filepath, "t")?;

    match tag {
        Some(v) => {
            write(tag_filepath, v).expect("Error writing to tag file");
            Ok(())
        }
        None => Ok(()),
    }
}

pub fn print_saved_times() {
    let times = database::get_times();
    for time in times {
        println!("gained time: {:?}", time);
    }
}

pub fn get_time_gained() -> Result<String, Error> {
    let filepath = get_filepath("time-gained")?;
    Ok(read_to_string(filepath)?)
}

fn set_time_gained(time_gained: String) -> Result<(), Error> {
    let filepath = get_filepath("time-gained")?;
    write(filepath, time_gained)?;
    Ok(())
}

fn get_durations_count() -> Result<u32, Error> {
    let filepath = get_filepath("durations-count")?;
    Ok(read_to_string(filepath)
        .expect("Reading from duration count file failed")
        .parse::<u32>()
        // TODO: will this actually return an error?
        .unwrap())
}

fn get_durations_avg() -> Result<String, Error> {
    let filepath = get_filepath("durations-average")?;
    Ok(read_to_string(filepath)?)
}
