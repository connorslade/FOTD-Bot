//! ```cargo
//! [dependencies]
//! rusqlite = { version = "0.28.0", features = ["bundled"] }
//! ```

use std::fs;

use rusqlite::Connection;

const DATABASE: &str = "../data/data.db";

fn main() {
    let mut db = Connection::open(DATABASE).unwrap();

    let total_users = db
        .query_row("SELECT COUNT(*) FROM users", [], |row| {
            row.get::<_, usize>(0)
        })
        .unwrap();

    let total_facts = db
        .query_row("SELECT COUNT(*) FROM facts", [], |row| {
            row.get::<_, usize>(0)
        })
        .unwrap();

    let used_facts = db
        .query_row(
            "SELECT COUNT(*) FROM facts WHERE used IS NOT NULL",
            [],
            |row| row.get::<_, usize>(0),
        )
        .unwrap();

    db.close().unwrap();

    println!(
        "TOTAL EMAILS: {}\nTOTAL FACTS: {}\nUSED FACTS: {} ({}%)",
        total_users,
        total_facts,
        used_facts,
        (used_facts as f32 / total_facts as f32 * 100.0).round()
    );
}
