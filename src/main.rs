use clap::{App, Arg};
use daemonize::Daemonize;

use std::fs::{create_dir, read_to_string, write, File, OpenOptions};
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::path::Path;
use std::process::exit;

mod database;

const WORKING_DIRECTORY: &str = "/var/tmp/productivity-timer";
const IN_FILE: &str = "/var/tmp/productivity-timer/in";
const OUT_FILE: &str = "/var/tmp/productivity-timer/out";
const ERR_FILE: &str = "/var/tmp/productivity-timer/err";
const PID_FILE: &str = "/var/tmp/productivity-timer/timer.pid";
const TIME_GAINED_FILE: &str = "/var/tmp/productivity-timer/time-gained";


fn main() {
    let matches = App::new("Productivity Timer")
        .author("Aaron Arinder <aaronarinder@protonmail.com>")
        .version("0.2.0")
        .about("Productivity Timer is a CLI and Daemon for recording quality time gained on projects. Quality time is time spent reading, writing, or thinking. Anything absent-minded (builds, deploys, [most] meetings, and so on) doesn't count. Consistently spending quality time on problems you care about will eventually solve those problems; so, get to it!")
        .arg(
            Arg::with_name("trigger")
                .short("t")
                .long("trigger")
                .help("Records a moment in time, either the beginning or end of a duration.")
        )
        .arg(
            Arg::with_name("print")
                .short("p")
                .long("print")
                .takes_value(true)
                .help("Prints from two places, either `db` for what's been saved or `tmp` for what's in /var/tmp/productivity-timer/time-gained.")
        )
        .arg(
            Arg::with_name("daemonize")
                .short("d")
                .long("daemonize")
                .help("Initializes the daemon, which is used for recording durations and interacting with the host system asynchronously to the CLI.")
        )
        .get_matches();

    let triggering = matches.is_present("trigger");
    let daemonizing = matches.is_present("daemonize");
    let printing = matches.is_present("print");
    let completing_session = matches.is_present("print");

    if completing_session {
        complete_session();
        print_saved_times();
    }

    if printing {
        match matches.value_of("print").unwrap() {
            "tmp" => {
                let time_gained = get_time_gained();
                println!("gained time: {:?}", time_gained);
            }
            "db" => print_saved_times(),
            _ => println!("Unrecognized command")
        }
    }

    if triggering {
        write(IN_FILE, "k").expect("Error writing to tmp in");
    }

    if daemonizing {
        daemonize();
    }
}

fn print_saved_times() {
    let times = database::get_times();
    for time in times {
        println!("gained time: {:?}", time);
    }
}

fn complete_session() {
    let time_gained = get_time_gained();
    database::save_time_gained(time_gained).unwrap()
}

fn get_time_gained() -> String {
    read_to_string(TIME_GAINED_FILE).expect("Reading from tmp in failed")
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

fn daemonize () {
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

fn listen_for_durations () {
    let mut durations: Vec<Instant> = Vec::new();

    let half_second = Duration::from_millis(500);

    loop {
        sleep(half_second);

        let input = read_to_string(IN_FILE).expect("Reading from tmp in failed");
        match input.trim() {
            "e" => {
                println!("println exiting");
                exit(0)
            }
            "k" =>  {
                println!("println k");
                durations.push(Instant::now())
            }
            "p" =>  {
                println!("println p");
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

fn reset_in_file () {
    write(IN_FILE, "").expect("Error writing to tmp in");
}
