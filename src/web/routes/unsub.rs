use afire::*;
use lettre::message::header;
use lettre::transport::smtp::authentication::Credentials;
use lettre::Message;
use lettre::SmtpTransport;
use lettre::Transport;
use std::fs;

use super::super::super::common::common;
use super::super::Auth;

/// Dir to find files to serve
const DATA_DIR: &str = "data/web";

static mut AUTH: Option<Auth> = None;

pub fn add_route(server: &mut afire::Server, auth: Auth) {
    unsafe {
        AUTH = Some(auth);
    }

    server.route(Method::POST, "/unsubscribe/real", |req| {
        let query = common::Query::from_body(req.body);

        let email = match &query.get("email") {
            Some(email) => common::decode_url_chars(email),
            None => return Response::new(400, "Invalid Email", vec![]),
        };

        let why = match &query.get("why") {
            Some(why) => common::decode_url_chars(why),
            None => return Response::new(400, "Invalid Reason", vec![]),
        };

        quick_email(
            unsafe { AUTH.as_mut().unwrap() },
            email.clone(),
            "FOTD BOT Unnsub".to_string(),
            "<a href=\"https://duck.com\">UNSUB</a>".to_string(),
        );

        Response::new(
            200,
            &fs::read_to_string(format!("{}/unsubscribe/done/index.html", DATA_DIR))
                .unwrap_or_else(|_| "done. email sent to {{EMAIL}} to confirm unsub.".to_string())
                .replace("{{EMAIL}}", &email)
                .replace("{{WHY}}", &why),
            vec![Header::new("Content-Type", "text/html")],
        )
    });
}

// TODO: Remove this
fn quick_email(email_auth: &mut Auth, to: String, subject: String, body: String) -> Option<()> {
    // Build the message
    let email = match Message::builder()
        .from(email_auth.username.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .header(header::ContentType::TEXT_HTML)
        .body(body)
    {
        // lil bodge {}
        Ok(email) => email,
        Err(_) => return None,
    };

    // Get credentials for mail server
    let creds = Credentials::new(email_auth.username.clone(), email_auth.password.clone());

    // Open a remote connection to the mail server
    let mailer = match SmtpTransport::relay(&email_auth.server) {
        Ok(mailer) => mailer.credentials(creds).build(),
        Err(_) => return None,
    };

    // Send the email
    match mailer.send(&email) {
        Ok(_) => Some(()),
        Err(_) => None,
    }
}
