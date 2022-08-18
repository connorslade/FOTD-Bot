//! ```cargo
//! [dependencies]
//! rusqlite = { version = "0.28.0", features = ["bundled"] }
//! serde_json = "1.0.83"
//! chrono = "0.4"
//! ```

use std::fs;

use chrono::{Datelike, NaiveDate};
use rusqlite::{params, Connection};
use serde_json::{from_str, Value};

const NEW_DATABASE: &str = "../data/data.db";
const DISCORD_DOWNLOAD: &str = "";

fn main() {
    let mut db = Connection::open(NEW_DATABASE).unwrap();
    let trans = db.transaction().unwrap();

    let value: Value = from_str(&fs::read_to_string(DISCORD_DOWNLOAD).unwrap()).unwrap();
    let messages = value
        .get("messages")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.get("embeds").unwrap().as_array().unwrap().get(0).unwrap())
        .map(|x| {
            (
                process_date(x.get("title").unwrap().as_str().unwrap()),
                x.get("description").unwrap().as_str().unwrap(),
            )
        });

    for (date, fact) in messages {
        trans
            .execute(
                "INSERT OR IGNORE INTO facts VALUES (?, ?)",
                params![fact, date],
            )
            .unwrap();
    }

    trans.commit().unwrap();
    db.close().unwrap();
}

fn process_date(str: &str) -> u32 {
    let parts = str
        .split("-")
        .map(|x| x.parse::<u32>().unwrap())
        .collect::<Vec<_>>();

    (NaiveDate::from_ymd(parts[0] as i32, parts[1], parts[2]).num_days_from_ce() - 719163) as u32
}
