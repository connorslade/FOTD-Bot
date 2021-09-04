use afire::*;
use rand::Rng;
use std::collections::HashMap;
use std::fs;

use super::super::super::common::common;
use super::super::quick_email;
use super::super::Auth;

/// Dir to find files to serve
const DATA_DIR: &str = "data/web";

static mut AUTH: Option<Auth> = None;
static mut BASE_URL: Option<String> = None;
static mut TEMPLATE_PATH: Option<String> = None;
static mut USER_PATH: Option<String> = None;
static mut SUB_CODES: Option<HashMap<String, String>> = None;

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
        SUB_CODES = Some(HashMap::new());
    }

    server.route(Method::POST, "/subscribe", |req| {
        let query = Query::from_body(req.body);

        // Get email address
        let email = match query.get("email") {
            Some(email) => common::decode_url_chars(&email).to_lowercase(),
            None => return Response::new(400, "Invalid Email", vec![]),
        };

        if email.is_empty() {
            return Response::new(400, "Invalid Email", vec![]);
        }

        // If email is already subscribed dont send email etc.
        let content = fs::read_to_string(unsafe { USER_PATH.clone() }.unwrap_or_default())
            .unwrap_or_default();
        if content.contains(&email) {
            return Response::new(
                200,
                "Your Already Subscribed!\nNo need to subscribe again!",
                vec![],
            );
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
            SUB_CODES
                .as_mut()
                .unwrap()
                .insert(random_chars.clone(), email.clone());
        }
        let mut confirm_url = unsafe { BASE_URL.clone() }
            .unwrap_or_else(|| "https://www.youtube.com/watch?v=mKwj3efLxbc".to_string());
        confirm_url.push_str(&format!("/subscribe/confirm?code={}", random_chars));

        // Try to read File
        let to_send = match fs::read_to_string(format!(
            "{}/subscribe.html",
            unsafe { TEMPLATE_PATH.clone() }.unwrap_or_default()
        )) {
            Ok(content) => content,
            Err(_) => "Subscribe: {{URL}}".to_string(),
        };

        quick_email(
            unsafe { AUTH.as_mut().unwrap() },
            email.clone(),
            "FOTD BOT Subscription".to_string(),
            to_send.replace("{{URL}}", &confirm_url),
        );

        Response::new(
            200,
            &fs::read_to_string(format!("{}/subscribe/done/index.html", DATA_DIR))
                .unwrap_or_else(|_| "done. email sent to {{EMAIL}} to confirm unsub.".to_string())
                .replace("{{EMAIL}}", &email),
            vec![Header::new("Content-Type", "text/html")],
        )
    });

    server.route(Method::GET, "/subscribe/confirm", |req| {
        let code = match req.query.get("code") {
            Some(code) => common::decode_url_chars(&code),
            None => return Response::new(400, "No Code supplied???", vec![]),
        };

        // Get email from hashmap
        let email = match unsafe { SUB_CODES.as_ref().unwrap() }.get(&code) {
            Some(email) => email.clone().to_lowercase(),
            None => return Response::new(400, "Invalid Code - Sorwy", vec![]),
        };

        if email.is_empty() {
            return Response::new(400, "Invalid Email", vec![]);
        }

        if code.is_empty() {
            return Response::new(400, "Invalid Code", vec![]);
        }

        // Remove from hashmap
        unsafe {
            SUB_CODES.as_mut().unwrap().remove(&code);
        }

        // Add User to 'database'
        let mut user_file =
            match fs::read_to_string(unsafe { USER_PATH.clone() }.unwrap_or_default()) {
                Ok(content) => content.replace("\r", ""),
                Err(_) => return Response::new(500, "Internal Error...", vec![]),
            };

        // Add user to file only if not already in file
        if !user_file.contains(&email) {
            user_file.push_str(&format!("\n{}", email));
            if user_file.starts_with("\n") {
                user_file.remove(0);
            }
            fs::write(unsafe { USER_PATH.clone() }.unwrap_or_default(), user_file).unwrap();
        }

        Response::new(
            200,
            &fs::read_to_string(format!("{}/subscribe/done/allDone.html", DATA_DIR))
                .unwrap_or_else(|_| {
                    "Done! You ({{EMAIL}}) will now get daily facts in your inbox!".to_string()
                })
                .replace("{{EMAIL}}", &email),
            vec![Header::new("Content-Type", "text/html")],
        )
    });
}
