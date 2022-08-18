use std::collections::HashMap;
use std::env;

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

        App {
            config,
            database: Mutex::new(database),
            fact: Mutex::new(String::new()),
            sub_codes: Mutex::new(HashMap::new()),
            unsub_codes: Mutex::new(HashMap::new()),
        }
    }
}
