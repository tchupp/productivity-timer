extern crate daemonize;

use std::io;
use std::fs::{create_dir, File, OpenOptions};
use std::time::{Duration, Instant};
use std::path::Path;
use daemonize::Daemonize;

const IN_FILE: &str = "/var/tmp/productivity-timer/out";
const OUT_FILE: &str = "/var/tmp/productivity-timer/in";
const ERR_FILE: &str = "/var/tmp/productivity-timer/err";
const PID_FILE: &str = "/var/tmp/productivity-timer/timer.pid";
const WORKING_DIRECTORY: &str = "/var/tmp/productivity-timer";

fn main() {
    daemonize();
}


fn create_tmp_productivity_timer_dir () {
    if !Path::new("/var/tmp/productivity-timer").exists() {
        match create_dir(WORKING_DIRECTORY) {
            Ok(_) => (),
            Err(e) => eprintln!("Error, {}", e),
        }
    }
}

fn create_tmp_files () -> (File, File){
    create_tmp_productivity_timer_dir();
    let tmp_daemon_out = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(OUT_FILE)
        .unwrap();

    let tmp_daemon_err = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(ERR_FILE)
        .unwrap();

    (tmp_daemon_out, tmp_daemon_err)
}

fn daemonize () {
    let (tmp_daemon_out, tmp_daemon_err) = create_tmp_files();

    let daemonize = Daemonize::new()
        .pid_file(PID_FILE)
        .working_directory(WORKING_DIRECTORY)
        .stdout(tmp_daemon_out)
        .stderr(tmp_daemon_err)
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
    loop {

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Something went wrong with reading stdin");

        match input.trim() {
            "k" =>  durations.push(Instant::now()),
            "p" =>  {
                let gained_time = get_duration_from_vec_of_tupled_instants(convert_vec_to_vec_of_tuples(durations.clone()));
                println!("gained time: {:?}", gained_time);
            },
            _ => println!("no default behavior yet")
        }

        // TODO: signals, not stdio
        // Adding break for now to demo the behavior, but `k` and `p` will
        // need to be rewritten
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
