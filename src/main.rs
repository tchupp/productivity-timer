use std::io;
use std::time::{Duration, Instant};

fn main() {
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

    }
}

fn convert_vec_to_vec_of_tuples(untupled_vec: Vec<Instant>) -> Vec<(Instant, Instant)> {
    // TODO: test for uneven number (meaning timer still going)
    let mut tupled_vec = Vec::new();
    for (idx, instant) in untupled_vec.iter().enumerate() {
        if idx % 2 == 0 {
            tupled_vec.push((*instant, untupled_vec[idx + 1]));
        }
    }
    println!("tupled_vec {:?}", tupled_vec);
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
