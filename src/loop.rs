use std::fs;
use std::io::{self, Write};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::prelude::*;
use rusqlite::params;

use crate::{
    color,
    misc::email::{Mailer, User},
    App, Color,
};

const SPINNER: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

pub fn start(app: Arc<App>) {
    let mut locked = false;

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
                let users = get_users(&app);

                println!(
                    "\x1b[2K\r{} {}",
                    color::color("[*] Sending", Color::Green),
                    color::color(&format!("[{}]", local_date), Color::Blue)
                );

                let fotd = random_fotd(&app);
                *app.fact.lock() = fotd.clone();

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
                let mut mailer = Mailer::new(
                    users.to_vec(),
                    User::new(&app.config.username, &app.config.sender_name),
                    &app.config.subject.replace("&1", &local_date),
                    &fs::read_to_string(app.config.data_path.join("template").join("index.html"))
                        .unwrap()
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

fn random_fotd(app: &Arc<App>) -> String {
    let db = app.database.lock();

    // Get number of available facts
    let row_id = match db.query_row(
        "SELECT rowid FROM facts WHERE used IS NULL ORDER BY RANDOM() LIMIT 1",
        [],
        |row| row.get::<_, usize>(0),
    ) {
        Ok(i) => i,
        Err(rusqlite::Error::QueryReturnedNoRows) => return "no more Facts!?".to_owned(),
        Err(e) => panic!("{:?}", e),
    };

    let epoch_day = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / (60 * 60 * 24);

    // Update row
    db.execute(
        "UPDATE facts SET used = ? WHERE rowid = ?",
        params![epoch_day, row_id],
    )
    .unwrap();

    // Return fact
    db.query_row("SELECT fact FROM facts WHERE rowid = ?", [row_id], |row| {
        row.get::<_, String>(0)
    })
    .unwrap()
}

fn get_users(app: &Arc<App>) -> Vec<User> {
    let db = app.database.lock();
    let mut stmt = db.prepare("SELECT email FROM users").unwrap();
    stmt.query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .map(|x| x.unwrap())
        .map(|x| User::user_from_email(&x))
        .collect::<Vec<_>>()
}
