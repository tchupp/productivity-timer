use dirs::home_dir;
use rusqlite::{params, Connection, Result};
use std::fmt;

// TODO better db name
const DATABASE_NAME: &str = "time_gained";

fn database_filename() -> String {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";
    let filename = working_directory + "/" + &DATABASE_NAME.to_string();
    filename
}

fn connect_to_database() -> Result<Connection, rusqlite::Error> {
    let database = database_filename();
    Connection::open(database)
}

pub fn new_session() -> Result<u64> {
    let conn = connect_to_database()?;

    // TODO: move thesxe table creation queries to connect_to_database
    // SQLite doesnt have a storage class set aside for storing dates and/or times. We can
    // use TEXT and the time fns will work with it (supposedly)
    match conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id                          INTEGER PRIMARY KEY,
            total_time                  TEXT,
            durations_count             INTEGER,
            durations_avg               TEXT
        )",
        [],
    ) {
        Ok(..) => (),
        Err(e) => panic!("error creating sessions table: {}", e),
    };

    let mut stmt = conn.prepare("INSERT INTO sessions DEFAULT VALUES returning id")?;

    struct Id {
        id: u64,
    }

    // TODO: fix this query to not be a map--just a simple execute, but rough with rusqlite so
    // hacked into place
    let ids: Vec<Id> = stmt
        .query_map([], |row| Ok(Id { id: row.get(0)? }))?
        .map(Result::unwrap)
        .collect();

    let id = ids[0].id;

    Ok(id)
}

pub fn save_tag(session_id: u64, tag_value: String, time: String) -> Result<()> {
    let conn = connect_to_database()?;
    match conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id                          INTEGER PRIMARY KEY,
            session_id                  INTEGER,
            value                       TEXT,
            time                        TEXT
        )",
        [],
    ) {
        Ok(..) => (),
        Err(e) => panic!("error creating sessions table: {}", e),
    };

    match conn.execute(
        "INSERT INTO tags (session_id, value, time) VALUES (?1, ?2, ?3)",
        params![session_id, tag_value, time],
    ) {
        Ok(..) => (),
        Err(e) => panic!("error inserting into db: {:?}", e),
    };

    Ok(())
}

pub fn get_tag_time(tag_value: &String) -> Result<u32> {
    let conn = connect_to_database()?;

    let mut stmt = conn.prepare("SELECT sum(strftime('%s', time) - strftime('%s', '00:00:00')) FROM tags WHERE value = :tag_value")?;
    let mut rows = stmt.query(&[(":tag_value", tag_value)])?;
    let mut time: Option<u32> = None;

    while let Some(row) = rows.next()? {
        time = row.get(0)?;
    }

    Ok(time.unwrap())
}

pub fn save_session(
    time_gained: String,
    durations_count: u32,
    durations_avg: String,
    session_id: u64,
) -> Result<()> {
    let conn = connect_to_database()?;
    match conn.execute(
        "UPDATE sessions SET (total_time, durations_count, durations_avg) = (?1, ?2, ?3) WHERE id = ?4",
        params![time_gained, durations_count, durations_avg, session_id],
    ) {
        Ok(..) => (),
        Err(e) =>panic!("error inserting into db: {:?}", e)
    };

    Ok(())
}

// TODO: reconsider typing
#[derive(Debug)]
pub struct TimeGained {
    id: i32,
    total_time: i32,
    durations_count: i32,
    durations_avg: String,
}

pub fn get_times() -> Result<Vec<TimeGained>> {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";
    let database = working_directory + "/" + &DATABASE_NAME.to_string();
    let conn = Connection::open(database)?;

    // TODO: interpolate the database name to make it dynamic
    // Use sqlite3's datetime fns to get total seconds
    let mut stmt =
        conn.prepare("SELECT id, strftime('%s', total_time) - strftime('%s', '00:00:00'), durations_count, durations_avg FROM sessions WHERE total_time IS NOT NULL")?;

    let times: Vec<TimeGained> = stmt
        .query_map([], |row| {
            Ok(TimeGained {
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

// TODO: reconsider typing
#[derive(Debug)]
pub struct LifetimeOverview {
    lifetime_total_time_avg: String,
    lifetime_durations_avg: String,
}

impl fmt::Display for LifetimeOverview {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "average total time: {}\n average duration: {}",
            self.lifetime_total_time_avg, self.lifetime_durations_avg
        )
    }
}

pub fn get_lifetime_overview() -> Result<Vec<LifetimeOverview>> {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";
    let database = working_directory + "/" + &DATABASE_NAME.to_string();
    let conn = Connection::open(database)?;

    // TODO: interpolate the database name to make it dynamic
    // Use sqlite3's datetime fns to get total seconds
    let mut stmt =
        conn.prepare("SELECT time(sum(strftime('%s', total_time) - strftime('%s', '00:00:00')) / count(total_time), 'unixepoch'), time(sum(strftime('%s', durations_avg) - strftime('%s', '00:00:00')) / count(durations_avg), 'unixepoch') FROM sessions")?;

    let times: Vec<LifetimeOverview> = stmt
        .query_map([], |row| {
            Ok(LifetimeOverview {
                lifetime_total_time_avg: row.get(0)?,
                lifetime_durations_avg: row.get(1)?,
            })
        })?
        .map(Result::unwrap)
        .collect();

    Ok(times)
}

// TODO: reconsider typing
#[derive(Debug)]
struct Tag {
    value: String,
    duration: String,
}

pub fn get_tags_pane() -> Result<String> {
    let conn = connect_to_database()?;

    let mut stmt =
        conn.prepare("SELECT value, time(sum(strftime('%s', time) - strftime('%s', '00:00:00')), 'unixepoch') AS time FROM tags WHERE time IS NOT NULL AND value IS NOT NULL GROUP BY VALUE ORDER BY time DESC")?;

    let tags: String = stmt
        .query_map([], |row| {
            Ok(Tag {
                value: row.get(0)?,
                duration: row.get(1)?,
            })
        })?
        .map(Result::unwrap)
        .map(|t| format!("{} :: {}\n", t.value, t.duration))
        .collect::<String>();
    Ok(tags)
}

#[derive(Debug)]
pub struct TotalTimeAsSeconds {
    pub total_time: i32,
}

pub fn get_total_time_as_seconds() -> Result<Vec<TotalTimeAsSeconds>> {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";
    let database = working_directory + "/" + &DATABASE_NAME.to_string();
    let conn = Connection::open(database)?;

    // TODO: interpolate the database name to make it dynamic
    // Use sqlite3's datetime fns to get total seconds
    let mut stmt = conn
        .prepare("SELECT strftime('%s', total_time) - strftime('%s', '00:00:00') FROM sessions WHERE total_time IS NOT NULL")?;

    let total_times: Vec<TotalTimeAsSeconds> = stmt
        .query_map([], |row| {
            Ok(TotalTimeAsSeconds {
                total_time: row.get(0)?,
            })
        })?
        .map(Result::unwrap)
        .collect();

    Ok(total_times)
}
