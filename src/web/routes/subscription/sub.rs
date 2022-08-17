use afire::{prelude::*, Query};
use rand::Rng;
use std::fs;
use std::sync::Arc;

use crate::{common::common, web::quick_email, App};

pub fn attach(server: &mut afire::Server, app: Arc<App>) {
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
        if common::is_subbed(&app, &email) {
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
        app.sub_codes
            .lock()
            .insert(random_chars.to_owned(), email.to_owned());
        let confirm_url = &format!(
            "{}/subscribe/confirm?code={}",
            app.config.web_url, random_chars
        );

        // Try to read File
        let to_send = match fs::read_to_string(
            app.config.data_path.join("template").join("subscribe.html"),
        ) {
            Ok(content) => content,
            Err(_) => "Subscribe: {{URL}}".to_string(),
        };

        quick_email(
            &app.config.web_auth,
            email.clone(),
            "FOTD BOT Subscription".to_string(),
            to_send.replace("{{URL}}", confirm_url),
        );

        Response::new()
            .text(
                fs::read_to_string(&app.config.data_path.join("web/subscribe/done/index.html"))
                    .unwrap_or_else(|_| "done. email sent to {{EMAIL}} to confirm sub!".to_string())
                    .replace("{{EMAIL}}", &email),
            )
            .header("Content-Type", "text/html")
    });
}
