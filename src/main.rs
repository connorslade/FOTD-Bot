use std::env;
use std::fs;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use chrono::prelude::*;
use rand::prelude::*;
use simple_config_parser::Config;

#[macro_use]
mod common;
mod email;
mod web;
mod webhook;
use common::color::*;
use common::*;

const VERSION: &str = "2.3.5";
const SPINNER: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

// Global Fact
pub static mut FACT: Option<String> = None;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config_file: &str =
        arg_parse::get_arg_value(&args, "--config").unwrap_or("./data/config/config.cfg");

    println!(
        "{}",
        color::color_bold(
            &format!("[*] Starting FOTD Bot Gen2 (V{})", VERSION),
            Color::Green
        )
    );

    let config = Config::new()
        .file(config_file)
        .expect("Error reading the config file");

    // Read some values from the config file
    let template = fs::read_to_string(&format!("{}/index.html", cfg_get(&config, "templatePath")))
        .expect("Error Reading Template");
    let template_path = cfg_get(&config, "templatePath");
    let user_path = cfg_get(&config, "emailListPath");
    let send_time = SendTime::from_str(&cfg_get(&config, "sendTime"));
    let subject = cfg_get(&config, "subject");
    let server = cfg_get(&config, "server");
    let sender_name = cfg_get(&config, "senderName");
    let username = cfg_get(&config, "username");
    let password = cfg_get(&config, "password");
    let fact_api = cfg_get(&config, "factApi") == "true";
    let web_url = cfg_get(&config, "webUrl");
    let web_auth = web::Auth::new(
        cfg_get(&config, "username"),
        cfg_get(&config, "password"),
        cfg_get(&config, "senderName"),
        cfg_get(&config, "server"),
    );
    let webhooks = webhook::parse_config(&config);

    // Verify Webhook
    for i in &webhooks {
        match i.verify() {
            Some(()) => color_print!(
                Color::Green,
                &"[*] Webhook `{}` Verified".replace("{}", &i.id)
            ),
            None => {
                println!(
                    "{}",
                    color::color_bold(
                        &"[!] Webhook Verification Failed for `{}`".replace("{}", &i.id),
                        Color::Red
                    )
                );
                panic!("Webhook Verification Failed");
            }
        };
    }

    // Start the webserver in another thread
    if cfg_get(&config, "webServer").to_lowercase() == "true" {
        let ip = cfg_get(&config, "webHost");
        let port = cfg_get(&config, "webPort").parse::<u16>().unwrap_or(8080);

        color_print!(
            Color::Magenta,
            "[*] Starting Web Server {}:{}",
            &ip,
            &port.to_string()
        );

        let clone_web_url = web_url.clone();
        thread::spawn(move || {
            web::start(
                &ip,
                port,
                web_auth,
                clone_web_url,
                template_path,
                user_path,
                fact_api,
            );
        });
    }

    println!();
    let mut locked = false;

    // TODO: Put this in its own file
    loop {
        for i in SPINNER.iter() {
            thread::sleep(Duration::from_millis(100));
            print!(
                "\x1b[2K\r{} {}",
                color::color(&format!("[{}] Waiting", i), Color::Cyan),
                color::color(
                    &format!("[{}:{}]", Local::now().hour(), Local::now().minute()),
                    Color::Blue
                )
            );
            io::stdout().flush().expect("Err flushing STD Out");
            if send_time.is_time() && !locked {
                locked = true;
                let local_date = Local::now().format("%Y-%m-%d").to_string();
                let users = user_array_from_file(&cfg_get(&config, "emailListPath"));

                println!(
                    "\x1b[2K\r{} {}",
                    color::color("[*] Sending", Color::Green),
                    color::color(&format!("[{}]", local_date), Color::Blue)
                );

                let fotd = random_fotd(cfg_get(&config, "factPath"));

                unsafe { FACT = Some(fotd.clone()) }

                // Send Webhooks
                for i in &webhooks {
                    match i.send(fotd.clone(), local_date.clone()) {
                        Some(()) => color_print!(
                            Color::Green,
                            &"\x1b[2K\r[*] Sending Webhook `{}`".replace("{}", &i.id)
                        ),
                        None => println!(
                            "{}",
                            color::color_bold(
                                &"[!] Webhook Send Failed for {}".replace("{}", &i.id),
                                Color::Red
                            )
                        ),
                    };
                }

                // Init Mailer and add some users
                let mut mailer = email::Mailer::new(
                    users.to_vec(),
                    email::User::new(username.clone(), sender_name.clone()),
                    &subject.replace("&1", &local_date),
                    &template
                        .replace("{{DATE}}", &local_date)
                        .replace("{{FOTD}}", &fotd)
                        .replace("{{BASE_URL}}", &web_url),
                    &server,
                    &username,
                    &password,
                );

                mailer.add_foreach(Box::new(|user| {
                    print!(
                        "\x1b[2K\r{}",
                        color::color(&format!("[*] Sending: {}", user.email), Color::Yellow)
                    );
                    std::io::stdout().flush().expect("Err flushing STD Out");
                }));

                mailer.send_all().expect("Error Sending Mail...");
            }

            if !send_time.is_time() {
                locked = false;
            }
        }
    }
}

fn cfg_get(cfg: &Config, key: &str) -> String {
    cfg.get_str(key)
        .unwrap_or_else(|_| panic!("The key '{}' was not defined in config :/", key))
}

fn random_fotd(path: String) -> String {
    // Read Facts and pick a random one
    let all_facts = fs::read_to_string(&path)
        .expect("Error Reading Fact File")
        .replace('\r', "");
    let facts: Vec<&str> = all_facts.split('\n').collect();
    let mut rng = rand::thread_rng();
    let fact = &facts.choose(&mut rng).unwrap();

    // Remove fact from list and write back
    let mut new_facts = String::new();
    for f in &facts {
        if &f == fact || f == &"" {
            continue;
        }
        new_facts.push_str(f);
        new_facts.push('\n');
    }
    fs::write(&path, new_facts).expect("Error ReWriting Fact File");

    // Return Fact
    fact.to_string()
}

fn user_array_from_file(path: &str) -> Vec<email::User> {
    let all_users = fs::read_to_string(&path)
        .expect("Error Reading User File")
        .replace("\r", "");
    let users: Vec<&str> = all_users.split('\n').collect();
    let mut users_vec: Vec<email::User> = Vec::new();
    for user in users {
        if user.is_empty() {
            continue;
        }
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
        let time_parts: Vec<&str> = time.split(':').collect();
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
