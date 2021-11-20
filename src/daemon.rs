use crate::database;
use crate::session::Session;
use daemonize::Daemonize;
use dirs::home_dir;
use regex::Regex;
use std::fs::{create_dir, read_to_string, write, File, OpenOptions};
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

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

pub fn format_instant_to_hhmmss(time_gained: Duration) -> String {
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

fn listen_for_durations() {
    let mut session = Session::new();
    let half_second = Duration::from_millis(500);

    loop {
        sleep(half_second);

        let input = read_from_in_file().unwrap();
        match input.trim() {
            "e" => exit(0),
            // TODO: find a better way to reset durations on session completions
            "c" => {
                session.save_session();
                session = Session::new();
            }
            "t" => {
                match session.active {
                    // TODO: figure out best way to take in flags for stuff like tags
                    true => session.pause(),
                    false => {
                        // TODO: DRY/refactor this logic (in -a, too)
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
                let time_gained = session.analytics.get_time_gained_formatted();
                println!("gained time: {:?}", time_gained);
            }
            "a" => {
                let minutes_to_add: u64 = get_misc().unwrap().parse().unwrap();

                // TODO: support tags
                session.record_addition(minutes_to_add);

                reset_misc().unwrap();
            }
            _ => (),
        }

        session.update_time_gained();
        // TODO: figure out best strategy for updating time gained: file? -p running every few
        // seconds? Cf i3bar/zsh and see what feels best
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

fn reset_in_file() -> Result<(), Error> {
    let in_filepath = get_filepath("in")?;
    write(in_filepath, "").expect("Error writing to tmp in");
    Ok(())
}

//fn zero_out_time_gained_file() -> Result<(), Error> {
//    let time_gained_filepath = get_filepath("time-gained")?;
//    // TODO: consider writing 00:00:00
//    write(time_gained_filepath, "").expect("Error writing to time-gained");
//    Ok(())
//}

// TODO: consolidate session completion fns and figure out a better way to do it
pub fn trigger_session_completion() -> Result<(), Error> {
    let in_filepath = get_filepath("in")?;
    write(in_filepath, "c").expect("Error writing to /in");
    Ok(())
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
