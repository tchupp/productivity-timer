use crate::database;
use daemonize::Daemonize;
use dirs::home_dir;
use regex::Regex;
use std::convert::TryInto;
use std::fs::{create_dir, read_to_string, write, File, OpenOptions};
use std::path::Path;
use std::process::exit;
use std::thread::sleep;
// TODO: apparently chrono supports negative durations (or some representation of time); it'd
// probably be smart to pull out std::time in favor of that to make -s, --subtract easier
use std::time::{Duration, Instant};

pub fn init() {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";

    let pid_file = working_directory.to_string() + "/timer.pid";

    let (tmp_file_out, tmp_file_err) = create_files();

    let daemonize = Daemonize::new()
        .pid_file(pid_file)
        .working_directory(working_directory)
        .stdout(tmp_file_out)
        .stderr(tmp_file_err)
        .exit_action(|| println!("TODO: exiting"));

    match daemonize.start() {
        Ok(_) => {
            println!("Success, daemonized");
            listen_for_durations()
        }
        Err(e) => eprintln!("Error, {}", e),
    }
}

// TODO Pass in struct
fn create_files() -> (File, File) {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";

    let in_file = working_directory.to_string() + "/in";
    let out_file = working_directory.to_string() + "/out";
    let err_file = working_directory.to_string() + "/err";
    let time_gained_file = working_directory.to_string() + "/time-gained";
    let durations_count_file = working_directory.to_string() + "/durations-count";
    let durations_avg_file = working_directory.to_string() + "/durations-average";

    create_tmp_productivity_timer_dir();
    let tmp_file_out = create_file(&out_file, false /*append*/);
    let tmp_file_err = create_file(&err_file, false /*append*/);

    // We only need this created, not passed back. We won't use File for
    // the in-file below, but rather the &str constant in_file
    create_file(&in_file, false /*append*/);
    create_file(&time_gained_file, false /*append*/);
    create_file(&durations_count_file, false /*append*/);
    create_file(&durations_avg_file, false /*append*/);
    // TODO: decide if I should clean outfile
    reset_in_file();

    (tmp_file_out, tmp_file_err)
}

fn create_tmp_productivity_timer_dir() {
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
fn read_from_in_file() -> String {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";

    let in_file = working_directory.to_string() + "/in";

    read_to_string(&in_file).expect("Reading from tmp in failed")
}

fn listen_for_durations() {
    let mut durations: Vec<Instant> = Vec::new();
    let mut additions: Vec<Duration> = Vec::new();

    let half_second = Duration::from_millis(500);

    loop {
        sleep(half_second);

        let input = read_from_in_file();
        match input.trim() {
            "e" => exit(0),
            // TODO: find a better way to reset durations on session completions
            "c" => {
                reset_in_file();
                durations = Vec::new();
                additions = Vec::new();
                // TODO: clean up the completion logic; it actually sets /time-gained
                // to an empty string and (accidentally) relies on the
                // `checked_write_time_gained_to_file` to set it to 00:00:00
                complete_session();
            }
            "k" => durations.push(Instant::now()),
            "p" => {
                // TODO: figure out whether there's a perf gain to & instead
                let gained_time = report_time_gained(durations.clone(), additions.clone());
                println!("gained time: {:?}", gained_time);
            }
            "a" => {
                let minutes_to_add: u64 = get_misc().parse().unwrap();
                let addition = Duration::new(minutes_to_add * 60, 0);
                additions.push(addition);
                reset_misc();
            }
            _ => (),
        }

        // On each loop, update time gained, duration count, average files
        // TODO: `checked` has a standard meaning in rust; revise this to be some other name
        checked_write_time_gained_to_file(durations.clone(), additions.clone());
        reset_in_file();
    }
}

fn checked_write_time_gained_to_file(mut durations: Vec<Instant>, additions: Vec<Duration>) {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";
    let time_gained_file = working_directory.to_string() + "/time-gained";
    let durations_count_file = working_directory.to_string() + "/durations-count";
    let durations_avg_file = working_directory.to_string() + "/durations-average";

    // NB: this won't include additions
    let durations_len = durations.len();
    // TODO: make a fn for checking even/odd
    if durations_len % 2 != 0 {
        durations.push(Instant::now())
    }

    let current_duration_gained = report_time_gained(durations, additions);

    let seconds_raw = current_duration_gained.as_secs() % 60;
    let minutes_raw = (current_duration_gained.as_secs() / 60) % 60;
    let hours_raw = (current_duration_gained.as_secs() / 60) / 60;

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

    let durations_count: u64 = (durations_len / 2).try_into().unwrap();

    // TODO: add something to guard against dividing by zero
    if durations_count > 0 {
        // TODO: DRY this shit up
        let avg_seconds_raw = (current_duration_gained.as_secs() / durations_count) % 60;
        let avg_minutes_raw = ((current_duration_gained.as_secs() / durations_count) / 60) % 60;
        let avg_hours_raw = ((current_duration_gained.as_secs() / durations_count) / 60) / 60;

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

        write(durations_count_file, format!("{}", durations_count))
            .expect("Error writing to durations count file");

        write(
            durations_avg_file,
            format!("{}:{}:{}", avg_hours, avg_minutes, avg_seconds),
        )
        .expect("Error writing to duration averages file");
    }

    write(
        time_gained_file,
        format!("{}:{}:{}", hours, minutes, seconds),
    )
    .expect("Error writing to time gained file");
}

fn reset_misc() {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";
    let misc_file = working_directory.to_string() + "/misc";
    write(misc_file, "").expect("Problem writing to misc file");
}

fn get_misc() -> String {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";
    let misc_file = working_directory.to_string() + "/misc";
    read_to_string(&misc_file).expect("Reading from misc file failed")
}

pub fn add_minutes(minutes_to_add: String) {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";
    let in_file = working_directory.to_string() + "/in";
    let misc_file = working_directory.to_string() + "/misc";

    let re = Regex::new(r"^\d+$").unwrap();
    assert!(re.is_match("15"));
    // TODO: better error message/handling if failed regex
    assert!(re.is_match(&minutes_to_add));

    write(in_file, "a").expect("Error writing to time gained file");
    write(misc_file, minutes_to_add).expect("Error writing to misc file");
}

fn report_time_gained(durations: Vec<Instant>, additions: Vec<Duration>) -> Duration {
    get_duration_from_vec_of_tupled_instants(convert_vec_to_vec_of_tuples(durations), additions)
}

fn reset_in_file() {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";

    let in_file = working_directory.to_string() + "/in";

    write(in_file, "").expect("Error writing to tmp in");
}

fn zero_out_time_gained_file() {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";

    let time_gained_file = working_directory.to_string() + "/time-gained";
    println!("in reset_time_gained");
    // TODO: consider writing 00:00:00
    write(time_gained_file, "").expect("Error writing to time-gained");
}

// TODO: consolidate session completion fns and figure out a better way to do it
pub fn trigger_session_completion() {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";
    let in_file = working_directory.to_string() + "/in";
    write(in_file, "c").expect("Error writing to /in");
}

fn complete_session() {
    let time_gained = get_time_gained();
    let durations_count = get_durations_count();
    let durations_avg = get_durations_avg();

    // TODO: error handling
    database::save_time_gained(time_gained, durations_count, durations_avg).unwrap();
    zero_out_time_gained_file();
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

pub fn trigger_time() {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";

    let in_file = working_directory.to_string() + "/in";
    write(in_file, "k").expect("Error writing to tmp in");
}

pub fn print_saved_times() {
    let times = database::get_times();
    for time in times {
        println!("gained time: {:?}", time);
    }
}

pub fn get_time_gained() -> String {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";
    let time_gained_file = working_directory.to_string() + "/time-gained";
    read_to_string(time_gained_file).expect("Reading from time gained file failed")
}

fn get_durations_count() -> u32 {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";
    let durations_count_file = working_directory.to_string() + "/durations-count";
    read_to_string(durations_count_file)
        .expect("Reading from duration count file failed")
        .parse::<u32>()
        .unwrap()
}

fn get_durations_avg() -> String {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";
    let durations_avg_file = working_directory.to_string() + "/durations-average";

    read_to_string(durations_avg_file).expect("Reading from duration average file failed")
}
