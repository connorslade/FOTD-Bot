use std::collections::HashMap;
use std::env;
use std::sync::RwLock;

use simple_config_parser as scp;

use crate::{common::arg_parse, config::Config};

pub struct App {
    pub config: Config,
    pub fact: RwLock<String>,

    pub sub_codes: RwLock<HashMap<String, String>>,
    pub unsub_codes: RwLock<HashMap<String, String>>,
}

impl App {
    pub fn from_args() -> Self {
        let args: Vec<String> = env::args().collect();
        let config_file: &str =
            arg_parse::get_arg_value(&args, "--config").unwrap_or("./data/config/config.cfg");

        App {
            config: Config::from(
                scp::Config::new()
                    .file(config_file)
                    .expect("Error reading the config file"),
            ),
            fact: RwLock::new(String::new()),
            sub_codes: RwLock::new(HashMap::new()),
            unsub_codes: RwLock::new(HashMap::new()),
        }
    }
}
