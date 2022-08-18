//! ```cargo
//! [dependencies]
//! rusqlite = { version = "0.28.0", features = ["bundled"] }
//! ```

use std::fs;

use rusqlite::Connection;

// CHANGE THESE
const OLD_FACTS: &str = "../data/facts/facts.txt";
const OLD_EMAILS: &str = "../data/config/sendTo.txt";

const NEW_DATABASE: &str = "../data/data.db";

fn main() {
    let mut db = Connection::open(NEW_DATABASE).unwrap();
    let trans = db.transaction().unwrap();

    for i in fs::read_to_string(OLD_EMAILS)
        .unwrap()
        .lines()
        .filter(|x| !x.is_empty())
    {
        trans
            .execute("INSERT OR IGNORE INTO users VALUES (?)", [i])
            .unwrap();
    }

    for i in fs::read_to_string(OLD_FACTS)
        .unwrap()
        .lines()
        .filter(|x| !x.is_empty())
    {
        trans
            .execute("INSERT OR IGNORE INTO facts (fact) VALUES (?)", [i])
            .unwrap();
    }

    trans.commit().unwrap();
    db.close().unwrap();
}
