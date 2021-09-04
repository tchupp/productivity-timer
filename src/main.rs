extern crate daemonize;

use std::fs::{create_dir, read_to_string, write, File, OpenOptions};
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::path::Path;
use std::process::exit;
use daemonize::Daemonize;

const WORKING_DIRECTORY: &str = "/var/tmp/productivity-timer";
const IN_FILE: &str = "/var/tmp/productivity-timer/in";
const OUT_FILE: &str = "/var/tmp/productivity-timer/out";
const ERR_FILE: &str = "/var/tmp/productivity-timer/err";
const PID_FILE: &str = "/var/tmp/productivity-timer/timer.pid";


fn main() {
    daemonize();
}


fn create_tmp_files () -> (File, File) {
    create_tmp_productivity_timer_dir();
    let tmp_file_out = create_tmp_file(OUT_FILE, false /*append*/);
    let tmp_file_err = create_tmp_file(ERR_FILE, false /*append*/);

    // We only need this created, not passed back. We won't use File for
    // the in-file below, but rather the &str constant IN_FILE
    create_tmp_file(IN_FILE, false /*append*/);
    // TODO: decide if I should clean outfile
    clean_files_on_startup();

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
                let gained_time = get_duration_from_vec_of_tupled_instants(
                    convert_vec_to_vec_of_tuples(
                        durations.clone()
                    )
                );
                println!("gained time: {:?}", gained_time);
            },
            _ => ()
        }

        write(IN_FILE, "").expect("Error writing to tmp in");
    }
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

fn clean_files_on_startup () {
    write(IN_FILE, "").expect("Error writing to tmp in");
}
