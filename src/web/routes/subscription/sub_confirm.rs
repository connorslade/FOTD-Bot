use afire::prelude::*;
use std::fs;
use std::sync::Arc;

use crate::{common::common, App};

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
        let email = match app.sub_codes.lock().get(&code) {
            Some(email) => {
                if email.is_empty() {
                    return Response::new().status(400).text("Invalid Code");
                }
                common::decode_url_chars(email).to_lowercase()
            }
            None => return Response::new().status(400).text("Invalid Code - Sorwy"),
        };

        // Remove from hashmap
        app.sub_codes.lock().remove(&code);

        // Add User to database
        app.database
            .lock()
            .execute("INSERT OR IGNORE INTO users VALUES (?)", [&email])
            .unwrap();

        Response::new()
            .text(
                &fs::read_to_string(&app.config.data_path.join("web/subscribe/done/allDone.html"))
                    .unwrap_or_else(|_| {
                        "Done! You ({{EMAIL}}) will now get daily facts in your inbox!".to_string()
                    })
                    .replace("{{EMAIL}}", &email),
            )
            .header("Content-Type", "text/html")
    });
}
