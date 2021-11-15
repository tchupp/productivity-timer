use clap::{App, Arg};

mod daemon;
mod database;
mod interface;

fn main() {
    let matches = App::new("Productivity Timer")
        .author("Aaron Arinder <aaronarinder@protonmail.com>")
        .version("0.2.0")
        .about("Productivity Timer is a CLI and Daemon for recording quality time gained on projects. Quality time is time spent reading, writing, or thinking. Anything absent-minded (builds, deploys, [most] meetings, and so on) doesn't count. Consistently spending quality time on problems you care about will eventually solve those problems; so, get to it!")
        .arg(
            Arg::with_name("daemonize")
                .short("d")
                .long("daemonize")
                .help("Initializes the daemon, which is used for recording durations and interacting with the host system asynchronously to the CLI.")
        )
        .arg(
            Arg::with_name("trigger")
                .short("t")
                .long("trigger")
                .takes_value(true)
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
            Arg::with_name("interface")
                .short("i")
                .long("interface")
                .help("Opens a terminal interface.")
        )
        .arg(
            Arg::with_name("add")
                .short("a")
                .long("add")
                .takes_value(true)
                .help("Add an arbitrary number of minutes to count as one duration for time gained. Example: pt -a 10, which adds 10 minutes to your time gained.")
        )
        .arg(
            Arg::with_name("complete")
                .short("c")
                .long("complete")
                .help("Completes a session of recording quality time.")
        )
        .get_matches();

    let triggering = matches.is_present("trigger");
    let daemonizing = matches.is_present("daemonize");
    let printing = matches.is_present("print");
    let interface = matches.is_present("interface");
    let adding_minutes = matches.is_present("add");
    let completing_session = matches.is_present("complete");

    if completing_session {
        daemon::trigger_session_completion().unwrap();
        daemon::print_saved_times();
    }

    if printing {
        match matches.value_of("print").unwrap() {
            "tmp" => {
                let time_gained = daemon::get_time_gained().unwrap();
                println!("{:?}", time_gained);
            }
            "db" => daemon::print_saved_times(),
            _ => println!("Unrecognized command"),
        }
    }

    if adding_minutes {
        // handle regex in supporting fn
        let minutes_to_add = matches.value_of("add").unwrap().to_string();
        daemon::add_minutes(minutes_to_add).unwrap();
    }

    if triggering {
        println!("wtf?");
        match matches.value_of("trigger") {
            Some(tag) => {
                daemon::trigger_time(Some(tag.to_string())).unwrap();
            }
            None => {
                daemon::trigger_time(None).unwrap();
            }
        }
    }

    if daemonizing {
        daemon::init();
    }

    if interface {
        interface::hello_world().unwrap();
    }
}
