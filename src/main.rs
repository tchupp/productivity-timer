use std::io;
use std::time::SystemTime;

fn main() {
    let mut durations: Vec<SystemTime> = Vec::new();
    loop {

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Something went wrong with reading stdin");

        match input.trim() {
            "k" =>  durations.push(SystemTime::now()),
            "p" =>  println!("{:?}", durations),
            _ => println!("no default behavior yet")
        }

    }
}
