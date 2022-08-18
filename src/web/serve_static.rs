use std::{fs, sync::Arc};

use afire::*;

use crate::{app::App, common::common};

pub fn add_route(server: &mut afire::Server, app: Arc<App>) {
    let web_path = app.config.data_path.join("web");

    // from before afire had a serve static extension i guess?
    server.route(Method::GET, "**", move |req| {
        let mut path = format!("{}{}", web_path.to_string_lossy(), safe_path(req.path));

        // Add Index.html if path ends with /
        if path.ends_with('/') {
            path.push_str("index.html");
        }

        // Also add '/index.html' if path dose not end with a file
        if !path.split('/').last().unwrap_or_default().contains('.') {
            path.push_str("/index.html");
        }

        // Try to read File
        match fs::read(&path) {
            // If its found send it as response
            Ok(content) => Response::new()
                .bytes(content)
                .header("Content-Type", common::get_type(&path)),

            // If not send 404.html
            Err(_) => Response::new()
                .status(404)
                .bytes(
                    fs::read(web_path.join("404.html"))
                        .unwrap_or_else(|_| "Not Found :/".as_bytes().to_owned()),
                )
                .header("Content-Type", "text/html"),
        }
    });
}

#[inline]
fn safe_path(mut path: String) -> String {
    while path.contains("/..") {
        path = path.replace("/..", "");
    }
    path
}
