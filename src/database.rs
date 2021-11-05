use dirs::home_dir;
use rusqlite::{params, Connection, Result};

const DATABASE_NAME: &str = "time_gained";

#[derive(Debug)]
pub struct TimeGained {
    id: i32,
    total_time: String,
    durations_count: i32,
    durations_avg: String,
}

pub fn save_time_gained(
    time_gained: String,
    durations_count: u32,
    durations_avg: String,
) -> Result<()> {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";
    let database = working_directory + "/" + &DATABASE_NAME.to_string();

    // TODO: switch back to ? rather than matches
    // creates database if it doesn't already exist
    let conn = match Connection::open(database) {
        Ok(conn) => conn,
        Err(e) => {
            println!("error opening db");
            return Err(e);
        }
    };

    // TODO: interpolate the database name to make it dynamic
    // SQLite doesnt have a storage class set aside for storing dates and/or times. We can
    // use TEXT and the time fns will work with it (supposedly)
    match conn.execute(
        "CREATE TABLE IF NOT EXISTS time_gained (
            id                          INTEGER PRIMARY KEY,
            total_time                  TEXT NOT NULL,
            durations_count             INTEGER NOT NULL,
            durations_avg               TEXT NOT NULL
        )",
        [],
    ) {
        // TODO: figure out what .. actually does
        Ok(..) => (),
        Err(e) => {
            println!("error creating table: {:?}", e);
            return Err(e);
        }
    };

    // TODO: interpolate the database name to make it dynamic
    match conn.execute(
        "INSERT INTO time_gained (total_time, durations_count, durations_avg) VALUES (?1, ?2, ?3)",
        params![time_gained, durations_count, durations_avg],
    ) {
        Ok(..) => (),
        Err(e) => {
            println!("error inserting into db: {:?}", e);
            return Err(e);
        }
    };

    Ok(())
}

// TODO: better typing
#[derive(Debug)]
pub struct TimeGainedModified {
    id: i32,
    pub total_time: i32,
    durations_count: i32,
    durations_avg: String,
}
pub fn get_times() -> Result<Vec<TimeGainedModified>> {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";
    let database = working_directory + "/" + &DATABASE_NAME.to_string();
    let conn = Connection::open(database)?;

    // TODO: interpolate the database name to make it dynamic
    // Use sqlite3's datetime fns to get total seconds
    let mut stmt =
        conn.prepare("SELECT id, strftime('%s', total_time) - strftime('%s', '00:00:00'), durations_count, durations_avg FROM time_gained")?;

    let times: Vec<TimeGainedModified> = stmt
        .query_map([], |row| {
            Ok(TimeGainedModified {
                id: row.get(0)?,
                total_time: row.get(1)?,
                durations_count: row.get(2)?,
                durations_avg: row.get(3)?,
            })
        })?
        .map(Result::unwrap)
        .collect();

    Ok(times)
}
