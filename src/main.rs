extern crate daemonize;

use std::io;
use std::time::{Duration, Instant};
use std::fs::File;
use daemonize::Daemonize;

fn main() {
    let tmp_daemon_out = File::create("/tmp/daemon.out").unwrap();
    let tmp_daemon_err = File::create("/tmp/daemon.err").unwrap();

    let daemonize = Daemonize::new()
        .pid_file("/tmp/test.pid") // Every method except `new` and `start`
        .working_directory("/tmp") // for default behaviour.
        .stdout(tmp_daemon_out)    // Redirect to `/tmp/daemon.out`.
        .stderr(tmp_daemon_err)    // Redirect to `/tmp/daemon.err`.
        .exit_action(|| println!("Executed before master process exits"));

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
        break
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
