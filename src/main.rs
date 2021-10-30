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
    let completing_session = matches.is_present("complete");

    if completing_session {
        daemon::trigger_session_completion();
        daemon::print_saved_times();
    }

    if printing {
        match matches.value_of("print").unwrap() {
            "tmp" => {
                let time_gained = daemon::get_time_gained();
                println!("gained time: {:?}", time_gained);
            }
            "db" => daemon::print_saved_times(),
            _ => println!("Unrecognized command"),
        }
    }

    if triggering {
        daemon::trigger_time();
    }

    if daemonizing {
        daemon::init();
    }

    if interface {
        interface::hello_world().unwrap();
    }
}
