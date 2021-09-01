use afire::*;
use std::fs;

const DATA_DIR: &str = "static";

fn main() {
    let mut server: Server = Server::new("localhost", 1800);

    Logger::attach(&mut server, Logger::new(Level::Info, None, true));

    server.all(|req| {
        let mut path = format!("{}/{}", DATA_DIR, req.path.replace("/..", ""));
        // Add Index.html if path ends with /
        if path.ends_with('/') {
            path.push_str("index.html");
        }

        // Also add '/index.html' if path dose not end with a file
        if !path.split('/').last().unwrap().contains('.') {
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

    server.route(Method::POST, "/unsubscribe/real", |req| {
        let query = Query::from_body(req.body);
        println!(
            "email: {} - why: {}",
            decode_url_chars(&query.get("email").unwrap_or_default()),
            decode_url_chars(&query.get("why").unwrap_or_default())
        );
        Response::new(
            200,
            "N O S E",
            vec![Header::new("Content-Type", "text/plain")],
        )
    });

    println!("Listening on http://localhost:1800");
    server.start();
}

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

struct Query {
    data: Vec<[String; 2]>,
}

impl Query {
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

    fn get(&self, key: &str) -> Option<String> {
        for i in self.data.clone() {
            if &i[0] == key {
                return Some(i[1].clone());
            }
        }
        None
    }
}
