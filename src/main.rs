use std::env;
use std::fs;
use std::io::Write;
use std::thread;
use std::time::Duration;

use chrono::prelude::*;
use rand::prelude::*;
use simple_config_parser::config::Config;

#[macro_use]
mod color;
mod arg_parse;
mod email;
use color::Color;

const VERSION: &str = "2.1.0";
const SPINNER: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

fn main() {
    // Get Args
    let args: Vec<String> = env::args().collect();
    let config_file: &str =
        &arg_parse::get_arg_value(&args, "--config").unwrap_or("./data/config/config.cfg");

    println!(
        "{}",
        color::color_bold(
            &format!("[*] Starting FOTD Bot Gen2 (V{})", VERSION),
            Color::Green
        )
    );

    let mut config = Config::new(Some(config_file));
    config.read().ok().expect("Error reading the config file");

    // Read some values from the config file
    let template =
        fs::read_to_string(&cfg_get(&config, "templatePath")).expect("Error Reading Template");
    let users = user_array_from_file(&cfg_get(&config, "emailListPath"));
    let send_time = SendTime::from_str(&cfg_get(&config, "sendTime"));
    let subject = cfg_get(&config, "subject");
    let server = cfg_get(&config, "server");
    let sender_name = cfg_get(&config, "senderName");
    let username = cfg_get(&config, "username");
    let password = cfg_get(&config, "password");

    let mut locked = false;

    loop {
        for i in SPINNER.iter() {
            thread::sleep(Duration::from_millis(100));
            print!(
                "\r{} {}",
                color::color(&format!("[{}] Waiting...", i), Color::Cyan),
                color::color(
                    &format!("[{}:{}]", Local::now().hour(), Local::now().minute()),
                    Color::Blue
                )
            );
            std::io::stdout().flush().expect("Err flushing STD Out");
            if send_time.is_time() && !locked {
                locked = true;
                let local_date = Local::now().format("%Y-%m-%d").to_string();

                println!(
                    "\x1b[2K\r{} {}",
                    color::color("[*] Sending", Color::Green),
                    color::color(&format!("[{}]", local_date), Color::Blue)
                );

                let fotd = random_fotd(cfg_get(&config, "factPath"));

                // Init Mailer and add some users
                let mailer = email::Mailer::new(
                    users.to_vec(),
                    email::User::new(username.clone(), sender_name.clone()),
                    &subject.replace("&1", &local_date),
                    &template
                        .replace("{{DATE}}", &local_date)
                        .replace("{{FOTD}}", &fotd),
                    &server,
                    &username,
                    &password,
                );

                mailer.send_all().expect("Error Sending Mail...");
            }

            if !send_time.is_time() {
                locked = false;
            }
        }
    }
}

fn cfg_get(cfg: &Config, key: &str) -> String {
    cfg.get(key)
        .expect(&format!("The key '{}' was not defined in config :/", key))
}

fn random_fotd(path: String) -> String {
    // Read Facts and pick a random one
    let all_facts = fs::read_to_string(&path).expect("Error Reading Fact File");
    let facts: Vec<&str> = all_facts.split("\n").collect();
    let mut rng = rand::thread_rng();
    let fact = &facts.choose(&mut rng).unwrap();

    // Remove fact from list and write back
    let mut new_facts = String::new();
    for f in &facts {
        if &f == fact || f == &"" {
            continue;
        }
        new_facts.push_str(f);
        new_facts.push_str("\n");
    }
    fs::write(&path, new_facts).expect("Error ReWriting Fact File");

    // Return Fact
    fact.to_string()
}

fn user_array_from_file(path: &str) -> Vec<email::User> {
    let all_users = fs::read_to_string(&path).expect("Error Reading User File").replace("\r", "");
    let users: Vec<&str> = all_users.split("\n").collect();
    let mut users_vec: Vec<email::User> = Vec::new();
    for user in users {
        users_vec.push(email::User::user_from_email(user));
    }
    users_vec
}

struct SendTime {
    hour: u32,
    minute: u32,
}

impl SendTime {
    fn from_str(time: &str) -> Self {
        let time_parts: Vec<&str> = time.split(":").collect();
        SendTime {
            hour: time_parts[0].parse::<u32>().expect("Invalid Send Hour"),
            minute: time_parts[1].parse::<u32>().expect("Invalid Send Minute"),
        }
    }

    fn is_time(&self) -> bool {
        let now = Local::now();
        now.hour() == self.hour && now.minute() == self.minute
    }
}
