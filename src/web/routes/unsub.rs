use afire::*;
use lettre::message::header;
use lettre::transport::smtp::authentication::Credentials;
use lettre::Message;
use lettre::SmtpTransport;
use lettre::Transport;
use rand::Rng;
use std::collections::HashMap;
use std::fs;

use super::super::super::common::common;
use super::super::Auth;

/// Dir to find files to serve
const DATA_DIR: &str = "data/web";

/// Fun Quotes to show on unsubscribe page
const QUOTES: [Quote; 5] = [
    Quote {
        quote: "Go, throw yourself into the sea!",
        author: "Jesus",
    },
    Quote {
        quote: "Im not mad im just dissapointed",
        author: "Every Parent Ever",
    },
    Quote {
        quote: "a threat to justice everywhere",
        author: "Martin Luther King JR",
    },
    Quote {
        quote: "worse than savage mobs",
        author: "Abraham Lincon",
    },
    Quote {
        quote: "Simba, I'm very disappointed in you.",
        author: "Mufasa",
    },
];

static mut AUTH: Option<Auth> = None;
static mut BASE_URL: Option<String> = None;
static mut TEMPLATE_PATH: Option<String> = None;
static mut USER_PATH: Option<String> = None;
static mut UNSUB_CODES: Option<HashMap<String, String>> = None;

pub fn add_route(
    server: &mut afire::Server,
    auth: Auth,
    base_url: String,
    template_path: String,
    user_path: String,
) {
    unsafe {
        AUTH = Some(auth);
        BASE_URL = Some(base_url);
        TEMPLATE_PATH = Some(template_path);
        USER_PATH = Some(user_path);
        UNSUB_CODES = Some(HashMap::new());
    }

    server.route(Method::POST, "/unsubscribe/real", |req| {
        let query = Query::from_body(req.body);

        let email = match query.get("email") {
            Some(email) => common::decode_url_chars(&email),
            None => return Response::new(400, "Invalid Email", vec![]),
        };

        let why = match query.get("why") {
            Some(why) => common::decode_url_chars(&why),
            None => return Response::new(400, "Invalid Reason", vec![]),
        };

        // Get confirm Url
        let random_chars = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(10)
            .collect::<Vec<u8>>();

        // Convert to string
        let random_chars = String::from_utf8(random_chars).unwrap();

        // Add to hashmap
        unsafe {
            UNSUB_CODES
                .as_mut()
                .unwrap()
                .insert(random_chars.clone(), email.clone());
        }
        let mut confirm_url = unsafe { BASE_URL.clone() }
            .unwrap_or("https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string());
        confirm_url.push_str(&format!("/unsubscribe/confirm?code={}", random_chars));

        // Try to read File
        let to_send = match fs::read_to_string(format!(
            "{}/unsubscribe.html",
            unsafe { TEMPLATE_PATH.clone() }.unwrap_or_default()
        )) {
            Ok(content) => content,
            Err(_) => "Unsub: {{URL}}".to_string(),
        };

        quick_email(
            unsafe { AUTH.as_mut().unwrap() },
            email.clone(),
            "FOTD BOT Unnsub".to_string(),
            to_send.replace("{{URL}}", &confirm_url),
        );

        // Print Hashmap for debug
        println!("{:?}", unsafe { UNSUB_CODES.as_ref().unwrap() });

        Response::new(
            200,
            &fs::read_to_string(format!("{}/unsubscribe/done/index.html", DATA_DIR))
                .unwrap_or_else(|_| "done. email sent to {{EMAIL}} to confirm unsub.".to_string())
                .replace("{{EMAIL}}", &email)
                .replace("{{WHY}}", &why),
            vec![Header::new("Content-Type", "text/html")],
        )
    });

    server.route(Method::GET, "/unsubscribe/confirm", |req| {
        println!(
            "code: {}",
            req.query.get("code").unwrap_or("NULL".to_string())
        );
        let code = match req.query.get("code") {
            Some(code) => common::decode_url_chars(&code),
            None => return Response::new(400, "No Code supplied???", vec![]),
        };

        // Get email from hashmap
        let email = match unsafe { UNSUB_CODES.as_ref().unwrap() }.get(&code) {
            Some(email) => email.clone(),
            None => return Response::new(400, "Invalid Code - Sorwy", vec![]),
        };

        // Remove from hashmap
        unsafe {
            UNSUB_CODES.as_mut().unwrap().remove(&code);
        }

        // Remove from 'database'
        let mut user_file =
            match fs::read_to_string(unsafe { USER_PATH.clone() }.unwrap_or_default()) {
                Ok(content) => content.replace("\r", ""),
                Err(_) => return Response::new(500, "Internal Error...", vec![]),
            };

        // Remove from file
        user_file = user_file.replace(&format!("{}\n", email), "");

        // Write to file
        fs::write(unsafe { USER_PATH.clone() }.unwrap_or_default(), user_file)
            .expect("Error ReWriting SendTo file");

        // Get a random Quote
        let quote = &QUOTES[rand::thread_rng().gen_range(0..QUOTES.len())];

        Response::new(
            200,
            &fs::read_to_string(format!("{}/unsubscribe/done/allDone.html", DATA_DIR))
                .unwrap_or_else(|_| {
                    "done. you ({{EMAIL}}) will no longer get amazing daily facts in your inbox :/"
                        .to_string()
                })
                .replace("{{EMAIL}}", &email)
                .replace("{{QUOTE}}", &quote.quote)
                .replace("{{AUTHOR}}", &quote.author),
            vec![Header::new("Content-Type", "text/html")],
        )
    });
}

// TODO: Remove this garbage
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

// this is beyond retarded
struct Quote<'a> {
    quote: &'a str,
    author: &'a str,
}
