use afire::*;
use lettre::message::header;
use lettre::transport::smtp::authentication::Credentials;
use lettre::Message;
use lettre::SmtpTransport;
use lettre::Transport;
use std::fs;

mod common;

/// Dir to find files to serve
const DATA_DIR: &str = "static";

fn main() {
    let mut server: Server = Server::new("localhost", 1800);

    // Add Logger and Rate Limiter
    Logger::attach(&mut server, Logger::new(Level::Info, None, true));
    RateLimiter::attach(&mut server, 10, 30);

    // Serve Static files from DATA_DIR
    server.all(|req| {
        let mut path = format!("{}/{}", DATA_DIR, req.path.replace("/..", ""));
        // Add Index.html if path ends with /
        if path.ends_with('/') {
            path.push_str("index.html");
        }

        // Also add '/index.html' if path dose not end with a file
        if !path.split('/').last().unwrap_or_default().contains('.') {
            path.push_str("/index.html");
        }

        // Try to read File
        match fs::read_to_string(&path) {
            // If its found send it as response
            Ok(content) => {
                return Response::new(
                    200,
                    &content,
                    vec![Header::new("Content-Type", common::get_type(&path))],
                );
            }

            // If not send 404.html
            Err(_) => {
                return Response::new(
                    404,
                    &fs::read_to_string(format!("{}/404.html", DATA_DIR))
                        .unwrap_or("Not Found :/".to_string()),
                    vec![Header::new("Content-Type", "text/html")],
                );
            }
        };
    });

    // Process Unsub Requests
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

        let auth = Auth {
            username: "connorslade@bernardsboe.com".to_string(),
            password: "305931".to_string(),
            server: "smtp.gmail.com".to_string(),
        };
        quick_email(auth, email.clone(), "FOTD BOT Unnsub".to_string(), "<a href=\"https://duck.com\">UNSUB</a>".to_string());

        Response::new(
            200,
            &fs::read_to_string(format!("{}/unsubscribe/done/index.html", DATA_DIR))
                .unwrap_or("done. email sent to {{EMAIL}} to confirm unsub.".to_string())
                .replace("{{EMAIL}}", &email)
                .replace("{{WHY}}", &why),
            vec![Header::new("Content-Type", "text/html")],
        )
    });

    println!("Listening on http://localhost:1800");
    server.start();
}

struct Auth {
    username: String,
    password: String,
    server: String,
}

///
fn quick_email(auth: Auth, to: String, subject: String, body: String) -> Option<()> {
    // Build the message
    let email = match Message::builder()
        .from(auth.username.parse().unwrap())
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
    let creds = Credentials::new(
        auth.username,
        auth.password
    );

    // Open a remote connection to the mail server
    let mailer = match SmtpTransport::relay(&auth.server) {
        Ok(mailer) => mailer.credentials(creds).build(),
        Err(_) => return None,
    };

    // Send the email
    match mailer.send(&email) {
        Ok(_) => return Some(()),
        Err(_) => return None,
    }
}