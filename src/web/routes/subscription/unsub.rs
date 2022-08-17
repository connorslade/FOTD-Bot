use afire::*;
use rand::Rng;
use std::fs;
use std::sync::Arc;

use crate::{common::common, web::quick_email, App};

pub fn attach(server: &mut afire::Server, app: Arc<App>) {
    server.route(Method::POST, "/unsubscribe/real", move |req| {
        let query = Query::from_body(req.body_string().unwrap()).unwrap();

        let email = match query.get("email") {
            Some(email) => {
                if email.is_empty() {
                    return Response::new().status(400).text("Invalid Email");
                }
                common::decode_url_chars(&email).to_lowercase()
            }
            None => return Response::new().status(400).text("Invalid Email"),
        };

        let why = match query.get("why") {
            Some(why) => {
                if why.is_empty() {
                    return Response::new().status(400).text("Invalid Reason");
                }
                common::decode_url_chars(&why)
            }
            None => return Response::new().status(400).text("Invalid Reason"),
        };

        // Check if email is in database
        if !common::is_subbed(&app, &email) {
            return Response::new()
                .text("You're not even subscribed.\nwhat are you trying to do???");
        }

        // Get confirm Url
        let random_chars = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(10)
            .collect::<Vec<u8>>();

        // Convert to string
        let random_chars = String::from_utf8(random_chars).unwrap();

        // Add to hashmap
        app.unsub_codes
            .lock()
            .insert(random_chars.clone(), email.clone());
        let confirm_url = format!(
            "{}/unsubscribe/confirm?code={}",
            app.config.web_url, random_chars
        );

        // Try to read File
        let to_send = match fs::read_to_string(
            app.config
                .data_path
                .join("template")
                .join("unsubscribe.html"),
        ) {
            Ok(content) => content,
            Err(_) => "Unsub: {{URL}}".to_string(),
        };

        quick_email(
            &app.config.web_auth,
            email.clone(),
            "FOTD BOT Unnsub".to_string(),
            to_send.replace("{{URL}}", &confirm_url),
        );

        Response::new()
            .text(
                fs::read_to_string(app.config.data_path.join("web/unsubscribe/done/index.html"))
                    .unwrap_or_else(|_| {
                        "done. email sent to {{EMAIL}} to confirm unsub.".to_string()
                    })
                    .replace("{{EMAIL}}", &email)
                    .replace("{{WHY}}", &why),
            )
            .header("Content-Type", "text/html")
    });
}
