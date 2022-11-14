use std::env;
use std::time::UNIX_EPOCH;
use std::{collections::HashMap, time::SystemTime};

use parking_lot::Mutex;
use rusqlite::Connection;
use simple_config_parser as scp;

use crate::{common::arg_parse, config::Config, database};

pub struct App {
    pub config: Config,
    pub database: Mutex<Connection>,
    pub fact: Mutex<String>,

    pub sub_codes: Mutex<HashMap<String, String>>,
    pub unsub_codes: Mutex<HashMap<String, String>>,
}

impl App {
    pub fn from_args() -> Self {
        let args: Vec<String> = env::args().collect();
        let config_file: &str =
            arg_parse::get_arg_value(&args, "--config").unwrap_or("./data/config.cfg");

        let config = Config::from(
            scp::Config::new()
                .file(config_file)
                .expect("Error reading the config file"),
        );

        let mut database = Connection::open(&config.database_path).unwrap();
        database::init(&mut database);

        // Attempt to get the days fact
        let fact = get_todays_fact(&database);
        if fact.is_none() {
            println!("[-] Previous fact not found");
        }

        App {
            config,
            database: Mutex::new(database),
            fact: Mutex::new(fact.unwrap_or_default()),
            sub_codes: Mutex::new(HashMap::new()),
            unsub_codes: Mutex::new(HashMap::new()),
        }
    }
}

fn get_todays_fact(db: &Connection) -> Option<String> {
    let epoch_day = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / (60 * 60 * 24)
        - 1;

    db.query_row(
        "SELECT fact FROM facts WHERE used = ?",
        [epoch_day],
        |row| row.get::<_, String>(0),
    )
    .ok()
}
