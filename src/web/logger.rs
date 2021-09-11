use super::super::common::color::{self, Color};
use afire::Server;

pub fn attach(server: &mut Server) {
    server.every(Box::new(|req| {
        let text = format!(
            "[{}] {} {}",
            remove_address_port(&req.address),
            req.method.to_string(),
            req.path,
        );

        color_print!(Color::Blue, "\x1b[2K\r{}", &text);

        None
    }));
}

fn remove_address_port(address: &str) -> String {
    address
        .split(':')
        .collect::<Vec<&str>>()
        .first()
        .unwrap_or(&"null")
        .to_string()
}
