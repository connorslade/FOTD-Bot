use afire::*;
use lettre::message::header;
use lettre::transport::smtp::authentication::Credentials;
use lettre::Message;
use lettre::SmtpTransport;
use lettre::Transport;
use std::fs;

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
                    vec![Header::new("Content-Type", get_type(&path))],
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
        let query = Query::from_body(req.body);

        let email = match &query.get("email") {
            Some(email) => decode_url_chars(email),
            None => return Response::new(400, "Invalid Email", vec![]),
        };

        let why = match &query.get("why") {
            Some(why) => decode_url_chars(why),
            None => return Response::new(400, "Invalid Reason", vec![]),
        };

        let auth = Auth {
            username: "".to_string(),
            password: "".to_string(),
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

/// Get the type MMIE content type of a file from its extension
fn get_type(path: &str) -> &str {
    match path.split(".").last() {
        Some(ext) => match ext {
            "html" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "png" => "image/png",
            "jpg" => "image/jpeg",
            "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "ico" => "image/x-icon",
            "svg" => "image/svg+xml",
            _ => "application/octet-stream",
        },

        None => "application/octet-stream",
    }
}

/// i dont even know...
fn append_frag(text: &mut String, frag: &mut String) {
    if !frag.is_empty() {
        let encoded = frag
            .chars()
            .collect::<Vec<char>>()
            .chunks(2)
            .map(|ch| u8::from_str_radix(&ch.iter().collect::<String>(), 16).unwrap())
            .collect::<Vec<u8>>();
        text.push_str(&std::str::from_utf8(&encoded).unwrap());
        frag.clear();
    }
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

/// Decode URL encoded strings
fn decode_url_chars(text: &str) -> String {
    let mut output = String::new();
    let mut encoded_ch = String::new();
    let mut iter = text.chars();
    while let Some(ch) = iter.next() {
        if ch == '%' {
            encoded_ch.push_str(&format!("{}{}", iter.next().unwrap(), iter.next().unwrap()));
            continue;
        }
        append_frag(&mut output, &mut encoded_ch);
        output.push(ch);
    }
    append_frag(&mut output, &mut encoded_ch);
    output
}

/// Holds key value pairs from Query Strings
struct Query {
    data: Vec<[String; 2]>,
}

impl Query {
    /// Create a new Query from a Form POST body
    fn from_body(body: String) -> Query {
        let mut data = Vec::new();

        let parts: Vec<&str> = body.split('&').collect();
        for i in parts {
            let sub: Vec<&str> = i.split('=').collect();
            if sub.len() < 2 {
                continue;
            }

            let key: String = sub[0].to_string();
            let value: String = sub[1].to_string();

            data.push([key, value])
        }

        Query { data }
    }

    /// Get a value from a key
    fn get(&self, key: &str) -> Option<String> {
        for i in self.data.clone() {
            if &i[0] == key {
                return Some(i[1].clone());
            }
        }
        None
    }
}
