use simple_config_parser::config::Config;
use std::env;
use std::fs;

use chrono::prelude::*;

#[macro_use]
mod color;
mod email;
use color::Color;

const VERSION: &str = "0.0.1";
const SPINNER: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

fn main() {
    let args: Vec<String> = env::args().collect();

    println!(
        "{}",
        color::color_bold(
            &format!("[*] Starting FOTD Bot Gen2 (V{})", VERSION),
            Color::Green
        )
    );

    let mut config = Config::new(Some("./data/config/config.cfg"));
    config.read().ok().expect("Error reading the config file");

    let local_date = Local::now().format("%Y-%m-%d").to_string();

    let template =
        fs::read_to_string(&cfg_get(&config, "templatePath")).expect("Error Reading Template")
        .replace("{{DATE}}", &local_date);

    // Init Mailer and add some users
    let mailer = email::Mailer::new(
        vec![email::User::user_from_email("connorslade@bernardsboe.com")],
        email::User::user_from_email("connorslade@bernardsboe.com"),
        &cfg_get(&config, "subject").replace("&2", &local_date),
        &template,
        &cfg_get(&config, "server"),
        &cfg_get(&config, "username"),
        &cfg_get(&config, "password"),
    );

    mailer.send_all().unwrap();

    // loop {
    //     for i in SPINNER.iter() {
    //         print!(
    //             "\r{} {}",
    //             color::color(&format!("[{}] Waiting...", i), Color::Cyan),
    //             Local::today()
    //         );
    //         std::io::stdout().flush().expect("Err flushing STD Out");
    //         thread::sleep(Duration::from_millis(100));
    //     }
    // }
}

fn cfg_get(cfg: &Config, key: &str) -> String {
    cfg.get(key)
        .expect(&format!("The key '{}' was not defined in config :/", key))
}
