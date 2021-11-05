use afire::*;
use rand::Rng;
use std::collections::HashMap;
use std::fs;

use super::super::super::common::common;
use super::super::quick_email;
use super::super::Auth;

/// Dir to find files to serve
const DATA_DIR: &str = "data/web";

/// Fun Quotes to show on unsubscribe page
const QUOTES: [Quote; 6] = [
    Quote {
        quote: "Go, throw yourself into the sea!",
        author: "Jesus",
    },
    Quote {
        quote: "Im not mad im just disappointed",
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
    Quote {
        quote: "You always have a choice. It's just that some people make the wrong one.",
        author: "Nicholas Sparks",
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
        let query = Query::from_body(req.body).unwrap();

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
        let content = fs::read_to_string(unsafe { USER_PATH.clone() }.unwrap_or_default())
            .unwrap_or_default();
        if !content.contains(&email) {
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
        unsafe {
            UNSUB_CODES
                .as_mut()
                .unwrap()
                .insert(random_chars.clone(), email.clone());
        }
        let mut confirm_url = unsafe { BASE_URL.clone() }
            .unwrap_or_else(|| "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string());
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

        Response::new()
            .text(
                fs::read_to_string(format!("{}/unsubscribe/done/index.html", DATA_DIR))
                    .unwrap_or_else(|_| {
                        "done. email sent to {{EMAIL}} to confirm unsub.".to_string()
                    })
                    .replace("{{EMAIL}}", &email)
                    .replace("{{WHY}}", &why),
            )
            .header(Header::new("Content-Type", "text/html"))
    });

    server.route(Method::GET, "/unsubscribe/confirm/real", |req| {
        let code = match req.query.get("code") {
            Some(code) => common::decode_url_chars(&code),
            None => return Response::new().status(400).text("No Code supplied???"),
        };

        // Get email from hashmap
        let email = match unsafe { UNSUB_CODES.as_ref().unwrap() }.get(&code) {
            Some(email) => email.clone().to_lowercase(),
            None => return Response::new().status(400).text("Invalid Code - Sorwy"),
        };

        if email.is_empty() {
            return Response::new().status(400).text("Invalid Email");
        }

        if code.is_empty() {
            return Response::new().status(400).text("Invalid Code");
        }

        // Remove from hashmap
        unsafe {
            UNSUB_CODES.as_mut().unwrap().remove(&code);
        }

        // Remove from 'database'
        let mut user_file =
            match fs::read_to_string(unsafe { USER_PATH.clone() }.unwrap_or_default()) {
                Ok(content) => content.replace("\r", ""),
                Err(_) => return Response::new().status(500).text("Internal Error..."),
            };

        // Remove email from file
        user_file = user_file.replace(&email, "").replace("\n\n", "\n");
        if user_file.starts_with('\n') {
            user_file.pop();
        }

        // Write to file
        fs::write(unsafe { USER_PATH.clone() }.unwrap_or_default(), user_file)
            .expect("Error ReWriting SendTo file");

        // Get a random Quote
        let quote = &QUOTES[rand::thread_rng().gen_range(0..QUOTES.len())];

        let base_url = &unsafe { BASE_URL.clone() }
            .unwrap_or_else(|| "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string());

        Response::new().text(fs::read_to_string(format!("{}/unsubscribe/done/allDone.html", DATA_DIR))
            .unwrap_or_else(|_| {
                "done. you ({{EMAIL}}) will no longer get amazing daily facts in your inbox :/"
                    .to_string()
            })
            .replace("{{EMAIL}}", &email)
            .replace("{{QUOTE}}", quote.quote)
            .replace("{{AUTHOR}}", quote.author)
            .replace("{{BASE_URL}}", base_url),).header(Header::new("Content-Type", "text/html"))
    });
}

// this is beyond retarded
struct Quote<'a> {
    quote: &'a str,
    author: &'a str,
}
