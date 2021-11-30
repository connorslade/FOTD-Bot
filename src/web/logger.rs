use super::super::common::color::{self, Color};
use afire::Server;

pub fn attach(server: &mut Server) {
    server.middleware(Box::new(|req| {
        let text = format!(
            "[{}] {} {}",
            remove_address_port(&req.address),
            req.method.to_string(),
            slash_path(&req.path),
        );

        color_print!(Color::Blue, "\x1b[2K\r{}", &text);

        None
    }));
}

fn remove_address_port(address: &str) -> String {
    address.split(':').next().unwrap_or("null").to_string()
}

fn slash_path(path: &str) -> String {
    if path.starts_with('/') {
        return path.to_string();
    }
    format!("/{}", path)
}
