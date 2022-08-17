use afire::*;
use rand::Rng;
use std::fmt::Write;
use std::fs;
use std::sync::Arc;

use crate::{common::common, web::quick_email, App};

/// Dir to find files to serve
const DATA_DIR: &str = "data/web";

pub fn attach(server: &mut afire::Server, app: Arc<App>) {
    let aapp = app.clone();
    server.route(Method::POST, "/subscribe", move |req| {
        let query = Query::from_body(req.body_string().unwrap()).unwrap();

        // Get email address
        let email = match query.get("email") {
            Some(email) => {
                if email.is_empty() {
                    return Response::new().status(400).text("Email is empty");
                }
                common::decode_url_chars(&email).to_lowercase()
            }
            None => return Response::new().status(400).text("Invalid Email"),
        };

        // If email is already subscribed dont send email etc.
        let content = fs::read_to_string(&aapp.config.user_path).unwrap_or_default();
        if content.contains(&email) {
            return Response::new().text("Your Already Subscribed!\nNo need to subscribe again!");
        }

        // Get confirm Url
        let random_chars = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(10)
            .collect::<Vec<u8>>();

        // Convert to string
        let random_chars = String::from_utf8(random_chars).unwrap();

        // Add to hashmap
        aapp.sub_codes
            .write()
            .unwrap()
            .insert(random_chars.clone(), email.clone());
        let confirm_url = &format!(
            "{}/subscribe/confirm?code={}",
            aapp.config.web_url, random_chars
        );

        // Try to read File
        let to_send =
            match fs::read_to_string(format!("{}/subscribe.html", aapp.config.template_path)) {
                Ok(content) => content,
                Err(_) => "Subscribe: {{URL}}".to_string(),
            };

        quick_email(
            &aapp.config.web_auth,
            email.clone(),
            "FOTD BOT Subscription".to_string(),
            to_send.replace("{{URL}}", confirm_url),
        );

        Response::new()
            .text(
                fs::read_to_string(format!("{}/subscribe/done/index.html", DATA_DIR))
                    .unwrap_or_else(|_| "done. email sent to {{EMAIL}} to confirm sub!".to_string())
                    .replace("{{EMAIL}}", &email),
            )
            .header("Content-Type", "text/html")
    });

    let aapp = app.clone();
    server.route(Method::GET, "/subscribe/confirm/real", move |req| {
        let code = match req.query.get("code") {
            Some(code) => {
                if code.is_empty() {
                    return Response::new().status(400).text("Invalid Code");
                }
                common::decode_url_chars(&code)
            }
            None => return Response::new().status(400).text("No Code supplied???"),
        };

        // Get email from hashmap
        let email = match app.sub_codes.read().unwrap().get(&code) {
            Some(email) => {
                if email.is_empty() {
                    return Response::new().status(400).text("Invalid Code");
                }
                common::decode_url_chars(email).to_lowercase()
            }
            None => return Response::new().status(400).text("Invalid Code - Sorwy"),
        };

        // Remove from hashmap
        app.sub_codes.write().unwrap().remove(&code);

        // Add User to 'database'
        let mut user_file = match fs::read_to_string(&aapp.config.user_path) {
            Ok(content) => content.replace('\r', ""),
            Err(_) => return Response::new().status(500).text("Internal Error..."),
        };

        // Add user to file only if not already in file
        if !user_file.contains(&email) {
            write!(user_file, "\n{}", email).unwrap();
            user_file = user_file.replace("\n\n", "\n");
            if user_file.starts_with('\n') {
                user_file.remove(0);
            }
            fs::write(&aapp.config.user_path, user_file).unwrap();
        }

        Response::new()
            .text(
                &fs::read_to_string(format!("{}/subscribe/done/allDone.html", DATA_DIR))
                    .unwrap_or_else(|_| {
                        "Done! You ({{EMAIL}}) will now get daily facts in your inbox!".to_string()
                    })
                    .replace("{{EMAIL}}", &email),
            )
            .header("Content-Type", "text/html")
    });
}
