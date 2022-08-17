use afire::prelude::*;
use std::fmt::Write;
use std::fs;
use std::sync::Arc;

use crate::{common::common, App};

// TODO: REMOVE THIS,,,
const DATA_DIR: &str = "data/web";

pub fn attach(server: &mut Server, app: Arc<App>) {
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
        let mut user_file = match fs::read_to_string(&app.config.user_path) {
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
            fs::write(&app.config.user_path, user_file).unwrap();
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
