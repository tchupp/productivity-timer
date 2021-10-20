use rusqlite::{params, Connection, Result};

#[derive(Debug)]
pub struct TimeGained {
    id: i32,
    total_time: String,
    number_of_durations: i32,
    duration_avg: i32,
}

pub fn save_time_gained(
    time_gained: String,
    number_of_durations: String,
    avg_duration: String,
) -> Result<()> {
    // creates database if it doesn't already exist
    let conn = Connection::open("time_gained")?;
    println!("time_gained as arg: {}", time_gained);

    conn.execute(
        "CREATE TABLE IF NOT EXISTS time_gained (
            id                       INTEGER PRIMARY KEY,
            total_time               TEXT NOT NULL,
            number_of_durations      INTEGER,
            duration_avg             INTEGER
        )",
        [],
    )?;
    println!("after first execute");
    conn.execute(
        "INSERT INTO time_gained (total_time, number_of_durations, duration_avg) VALUES (?1, ?2, ?3)",
        params![time_gained, 0,0],
    )?;

    Ok(())
}

pub fn get_times() -> Result<Vec<TimeGained>> {
    let conn = Connection::open("time_gained")?;

    let mut stmt =
        conn.prepare("SELECT id, total_time, number_of_durations, duration_avg FROM time_gained")?;

    let times: Vec<TimeGained> = stmt
        .query_map([], |row| {
            Ok(TimeGained {
                id: row.get(0)?,
                total_time: row.get(1)?,
                number_of_durations: row.get(2)?,
                duration_avg: row.get(2)?,
            })
        })?
        .map(Result::unwrap)
        .collect();

    Ok(times)
}
