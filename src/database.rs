use crate::oauth::get_token;
use dirs::home_dir;
use dotenv;
use reqwest;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use rusqlite::{params, Connection, Result};
use serde::Deserialize;
use std::fmt;
use std::fs::File;

// TODO better db name
const DATABASE_NAME: &str = "time_gained";
const DRIVE_FILE_URL: &str = "https://www.googleapis.com/drive/v3/files";
// NB the trailing "/"
const DRIVE_FILE_UPLOAD_URL: &str = "https://www.googleapis.com/upload/drive/v3/files/";

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
            durations_avg               TEXT,
            tag                         TEXT
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
    tag: String,
) -> Result<()> {
    let conn = connect_to_database()?;
    match conn.execute(
        "UPDATE sessions SET (total_time, durations_count, durations_avg, tag) = (?1, ?2, ?3, ?4) WHERE id = ?5",
        params![time_gained, durations_count, durations_avg, tag, session_id],
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

pub fn get_lifetime_overview(session_tag: &String) -> Result<Vec<LifetimeOverview>> {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";
    let database = working_directory + "/" + &DATABASE_NAME.to_string();
    let conn = Connection::open(database)?;

    // TODO: interpolate the database name to make it dynamic
    // Use sqlite3's datetime fns to get total seconds
    let mut stmt =
        conn.prepare("SELECT time(sum(strftime('%s', total_time) - strftime('%s', '00:00:00')) / count(total_time), 'unixepoch'), time(sum(strftime('%s', durations_avg) - strftime('%s', '00:00:00')) / count(durations_avg), 'unixepoch') FROM sessions WHERE tag = :tag")?;

    let times: Vec<LifetimeOverview> = stmt
        .query_map(&[(":tag", &session_tag)], |row| {
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

pub fn get_tags_pane(session_tag: &String) -> Result<String> {
    let conn = connect_to_database()?;

    let mut stmt =
        conn.prepare("SELECT value, time(sum(strftime('%s', time) - strftime('%s', '00:00:00')), 'unixepoch') AS time FROM tags t JOIN sessions s ON s.id = t.session_id WHERE s.tag = :tag AND t.time IS NOT NULL AND t.value IS NOT NULL GROUP BY VALUE ORDER BY t.time DESC")?;

    let tags: String = stmt
        .query_map(&[(":tag", &session_tag)], |row| {
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

pub fn get_total_time_as_seconds(session_tag: &String) -> Result<Vec<TotalTimeAsSeconds>> {
    let working_directory =
        home_dir().unwrap().as_path().display().to_string() + "/.productivity-timer";
    let database = working_directory + "/" + &DATABASE_NAME.to_string();
    let conn = Connection::open(database)?;

    // TODO: interpolate the database name to make it dynamic
    // Use sqlite3's datetime fns to get total seconds
    let mut stmt = conn
        .prepare("SELECT strftime('%s', total_time) - strftime('%s', '00:00:00') FROM sessions WHERE total_time IS NOT NULL AND tag = :tag")?;

    //let mut rows = stmt.query(&[(":tag_value", tag_value)])?;
    let total_times: Vec<TotalTimeAsSeconds> = stmt
        .query_map(&[(":tag", &session_tag)], |row| {
            Ok(TotalTimeAsSeconds {
                total_time: row.get(0)?,
            })
        })?
        .map(Result::unwrap)
        .collect();

    Ok(total_times)
}

#[derive(Deserialize, Debug)]
struct DriveFile {
    kind: String,
    id: String,
    name: String,
    // TODO: not picking up?
    #[allow(non_camel_case_types)]
    mimeType: String,
}

#[derive(Deserialize, Debug)]
struct FilesResponse {
    kind: String,
    // TODO: not picking up?
    #[allow(non_camel_case_types)]
    incompleteSearch: bool,
    files: Vec<DriveFile>,
}

pub fn backup() -> Result<(), reqwest::Error> {
    dotenv::dotenv().ok();
    let api_key = dotenv::var("API_KEY").unwrap();
    let token = get_token();

    let client = reqwest::blocking::Client::new();

    // TODO better error handling--need to figure out uniform error handling across app
    let drive_database_file_id = &client
        .get(DRIVE_FILE_URL.to_string() + "?key=" + &api_key)
        // TODO figure out if I actually need this content-type
        .header(ACCEPT, "application/json")
        .bearer_auth(&token)
        .send()?
        .json::<FilesResponse>()
        .unwrap()
        .files[0]
        // NB also has mimetype
        .id;

    let database_filename = database_filename();
    let local_database_file = File::open(database_filename).unwrap();

    let result = &client
        // NB uploadType=media is good up to 5mb, which is ~416x the size of my current sqlite db;
        // we'll worry about multipart uploads whenever we actually have to worry about them, but
        // reqwest has an api for it
        .patch(
            DRIVE_FILE_UPLOAD_URL.to_string()
                + drive_database_file_id
                + "?key="
                + &api_key
                + "&uploadType=media",
        )
        .header(ACCEPT, "application/json")
        .header(CONTENT_TYPE, "application/x-sqlite3")
        .bearer_auth(token)
        .body(local_database_file)
        .send();

    println!("result: {:?}", result);

    Ok(())
}
