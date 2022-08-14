use std::fs;
use std::io::{self, Write};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use chrono::prelude::*;
use rand::prelude::*;

#[macro_use]
mod common;
mod app;
mod config;
mod email;
mod misc;
mod web;
mod webhook;
use common::color::*;
use common::*;

use crate::app::App;

const VERSION: &str = "2.3.5";
const SPINNER: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

fn main() {
    println!(
        "{}",
        color::color_bold(
            &format!("[*] Starting FOTD Bot Gen2 (V{})", VERSION),
            Color::Green
        )
    );

    let app = Arc::new(App::from_args());

    // Verify Webhook
    for i in &app.config.webhooks {
        if i.verify() {
            color_print!(Color::Green, &format!("[*] Webhook `{}` Verified", i.id));
            continue;
        }
        println!(
            "{}",
            color::color_bold(
                &"[!] Webhook Verification Failed for `{}`".replace("{}", &i.id),
                Color::Red
            )
        );
        panic!("Webhook Verification Failed");
    }

    // Start the webserver in another thread
    if app.config.web_server {
        color_print!(
            Color::Magenta,
            "[*] Starting Web Server {}:{}",
            &app.config.web_ip,
            app.config.web_port.to_string().as_str()
        );
        let aapp = app.clone();
        thread::spawn(move || web::start(aapp));
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
            io::stdout().flush().expect("Err flushing stdout");
            if app.config.send_time.is_time() && !locked {
                locked = true;
                let local_date = Local::now().format("%Y-%m-%d").to_string();
                let users = user_array_from_file(&app.config.user_path);

                println!(
                    "\x1b[2K\r{} {}",
                    color::color("[*] Sending", Color::Green),
                    color::color(&format!("[{}]", local_date), Color::Blue)
                );

                let fotd = random_fotd(&app.config.fact_path);
                *app.fact.write().unwrap() = fotd.clone();

                // Send Webhooks
                for i in &app.config.webhooks {
                    match i.send(&fotd, &local_date) {
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
                    email::User::new(&app.config.username, &app.config.sender_name),
                    &app.config.subject.replace("&1", &local_date),
                    &app.config
                        .template
                        .replace("{{DATE}}", &local_date)
                        .replace("{{FOTD}}", &fotd)
                        .replace("{{BASE_URL}}", &app.config.web_url),
                    &app.config.server,
                    &app.config.username,
                    &app.config.password,
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

            if !app.config.send_time.is_time() {
                locked = false;
            }
        }
    }
}

fn random_fotd(path: &str) -> String {
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
