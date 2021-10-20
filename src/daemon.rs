use daemonize::Daemonize;
use std::path::Path;
use std::fs::{create_dir, read_to_string, write, File, OpenOptions};
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::process::exit;
use crate::database;

const PID_FILE: &str = "/var/tmp/productivity-timer/timer.pid";
const WORKING_DIRECTORY: &str = "/var/tmp/productivity-timer";
const IN_FILE: &str = "/var/tmp/productivity-timer/in";
const OUT_FILE: &str = "/var/tmp/productivity-timer/out";
const ERR_FILE: &str = "/var/tmp/productivity-timer/err";
const TIME_GAINED_FILE: &str = "/var/tmp/productivity-timer/time-gained";

pub fn init() {
    let (tmp_file_out, tmp_file_err) = create_tmp_files();

    let daemonize = Daemonize::new()
        .pid_file(PID_FILE)
        .working_directory(WORKING_DIRECTORY)
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

fn create_tmp_files () -> (File, File) {
    create_tmp_productivity_timer_dir();
    let tmp_file_out = create_tmp_file(OUT_FILE, false /*append*/);
    let tmp_file_err = create_tmp_file(ERR_FILE, false /*append*/);

    // We only need this created, not passed back. We won't use File for
    // the in-file below, but rather the &str constant IN_FILE
    create_tmp_file(IN_FILE, false /*append*/);
    create_tmp_file(TIME_GAINED_FILE, false /*append*/);
    // TODO: decide if I should clean outfile
    reset_in_file();

    (tmp_file_out, tmp_file_err)
}

fn create_tmp_productivity_timer_dir () {
    if !Path::new("/var/tmp/productivity-timer").exists() {
        match create_dir(WORKING_DIRECTORY) {
            Ok(_) => (),
            Err(e) => eprintln!("Error, {}", e),
        }
    }
}

fn create_tmp_file (file_name: &str, append: bool) -> File {
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

fn listen_for_durations () {
    let mut durations: Vec<Instant> = Vec::new();

    let half_second = Duration::from_millis(500);

    loop {
        sleep(half_second);

        let input = read_to_string(IN_FILE).expect("Reading from tmp in failed");
        match input.trim() {
            "e" => {
                exit(0)
            }
            "k" =>  {
                durations.push(Instant::now())
            }
            "p" =>  {
                let gained_time = report_time_gained(durations.clone());
                println!("gained time: {:?}", gained_time);
            },
            _ => ()
        }

        // treating `checked` as a convention for this fn requiring a condition be fulfilled (i.e.,
        // an even number of durations) to actually write to the file
        checked_write_time_gained_to_file(durations.clone());
        write(IN_FILE, "").expect("Error writing to tmp in");
    }
}

fn checked_write_time_gained_to_file (mut durations: Vec<Instant>) {
    // TODO: make a fn for checking even/odd
    if durations.len() % 2 != 0 {
        durations.push(Instant::now())
    }

    let current_duration_gained = report_time_gained(durations);
    let seconds = current_duration_gained.as_secs() % 60;
    let minutes = (current_duration_gained.as_secs() / 60) % 60;
    let hours = (current_duration_gained.as_secs() / 60) / 60;

    write(TIME_GAINED_FILE, format!("{}:{}:{}", hours, minutes, seconds)).expect("Error writing to time gained file");
}

fn report_time_gained (durations: Vec<Instant>) -> Duration {
    get_duration_from_vec_of_tupled_instants(
        convert_vec_to_vec_of_tuples(
            durations
        )
    )
}

fn reset_in_file () {
    write(IN_FILE, "").expect("Error writing to tmp in");
}

fn reset_time_gained_file () {
    write(TIME_GAINED_FILE, "").expect("Error writing to tmp in");
}

pub fn complete_session() {
    //total_time               TEXT NOT NULL,
    //number_of_durations      INTEGER,
    //duration_avg             INTEGER
    let time_gained = get_time_gained();
    database::save_time_gained(
        time_gained,
        number_of_durations,
        avg_duration
    ).unwrap();
    reset_in_file();
    reset_time_gained_file();
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

fn get_duration_from_vec_of_tupled_instants(tupled_vec: Vec<(Instant, Instant)>) -> Duration {
    let durations_from_tuples: Vec<Duration> = tupled_vec.iter().map(|tuple| {
        match tuple.1.checked_duration_since(tuple.0) {
            Some(v) => v,
            None => {
                panic!("TODO: something serious would have gone wrong")
            }
        }
    })
    .collect();

    durations_from_tuples
        .iter()
        .sum()
}

pub fn trigger_time() {
    write(IN_FILE, "k").expect("Error writing to tmp in");
}

pub fn print_saved_times() {
    let times = database::get_times();
    for time in times {
        println!("gained time: {:?}", time);
    }
}

pub fn get_time_gained() -> String {
    read_to_string(TIME_GAINED_FILE).expect("Reading from tmp in failed")
}
