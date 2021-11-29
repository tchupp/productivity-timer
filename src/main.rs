use clap::{App, Arg};

mod analytics;
mod daemon;
mod database;
mod interface;
mod pt_duration;
mod session;

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
                .help("Prints from two places, either `db` for what's been saved or `tmp` for what's in /var/tmp/productivity-timer/time-gained.")
        )
        .arg(
            Arg::with_name("interface")
                .short("i")
                .long("interface")
                .takes_value(true)
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
            Arg::with_name("subtract")
                .short("s")
                .long("subtract")
                .takes_value(true)
                .help("Subtract an arbitrary number of minutes, counting as one duration, from a session. Example: pt -s 10, which subtracts 10 minutes from your session.")
        )
        .arg(
            Arg::with_name("complete")
                .short("c")
                .long("complete")
                .takes_value(true)
                .help("Completes a session of recording quality time.")
        )
        .arg(
            // TODO: remove kebob-case
            Arg::with_name("tag-time")
                .short("g")
                .long("tag-time")
                .takes_value(true)
                .help("Get time gained for a tag.")
        )
        .get_matches();

    let triggering = matches.is_present("trigger");
    let daemonizing = matches.is_present("daemonize");
    let printing = matches.is_present("print");
    let interface = matches.is_present("interface");
    let adding_minutes = matches.is_present("add");
    let subtracting_minutes = matches.is_present("subtract");
    let completing_session = matches.is_present("complete");
    let tag_time = matches.is_present("tag-time");

    if completing_session {
        let tag = matches.value_of("complete").unwrap().to_string();
        daemon::trigger_session_completion(tag).unwrap();
        let times = database::get_times();
        for time in times {
            println!("gained time: {:?}", time);
        }
    }

    if printing {
        let time_gained = daemon::get_time_gained().unwrap();
        println!("{:?}", time_gained);
    }

    // TODO: support tags
    if adding_minutes {
        // handle regex in supporting fn
        let minutes_to_add = matches.value_of("add").unwrap().to_string();
        daemon::add_minutes(minutes_to_add).unwrap();
    }

    if subtracting_minutes {
        // handle regex in supporting fn
        let minutes_to_subtract = matches.value_of("subtract").unwrap().to_string();
        daemon::subtract_minutes(minutes_to_subtract).unwrap();
    }

    if triggering {
        match matches.value_of("trigger") {
            Some(tag) => {
                daemon::trigger_time(Some(tag.to_string())).unwrap();
            }
            None => {
                daemon::trigger_time(None).unwrap();
            }
        }
    }

    if tag_time {
        let tag = matches.value_of("tag-time").unwrap().to_string();
        daemon::print_tags(tag);
        //let tag_time = database::get_tag_time(&tag).unwrap();
        //println!("{}: {}", tag, tag_time);
    }

    if daemonizing {
        daemon::init();
    }

    if interface {
        let tag = matches.value_of("interface").unwrap().to_string();
        interface::draw(tag).unwrap();
    }
}
