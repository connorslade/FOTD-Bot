use std::sync::Arc;
use std::thread;

#[macro_use]
mod common;
mod app;
mod config;
mod database;
mod r#loop;
mod misc;
mod web;
mod webhook;
use common::color::*;
use common::*;

use crate::app::App;

const VERSION: &str = "2.4.0";

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

    r#loop::start(app);
}
