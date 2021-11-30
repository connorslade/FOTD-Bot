use std::time::Duration;

use afire::{Header, Server};

pub use super::email::{quick_email, Auth};
use crate::VERSION;
mod logger;
mod routes;

pub fn start(
    ip: &str,
    port: u16,
    email_auth: Auth,
    base_url: String,
    template_path: String,
    user_path: String,
    fact_api: bool,
) {
    let mut server: Server = Server::new(ip, port);

    // Add default headers
    server.add_default_header(Header::new("X-Frame-Options", "DENY"));
    server.add_default_header(Header::new("X-Content-Type-Options", "nosniff"));
    server.add_default_header(Header::new("X-Version", &format!("FOTD-BOT/{}", VERSION)));

    // Add Custom Logger
    logger::attach(&mut server);
    // Logger::attach(&mut server, Logger::new(Level::Info, None, true));
    // RateLimiter::attach(&mut server, 10, 30);

    server.set_socket_timeout(Some(Duration::from_secs(1)));

    // Serve Static files from DATA_DIR
    routes::serve_static::add_route(&mut server);

    // Process Unsub requests
    routes::unsub::add_route(
        &mut server,
        email_auth.clone(),
        base_url.clone(),
        template_path.clone(),
        user_path.clone(),
    );

    // Process Sub requests
    routes::sub::add_route(&mut server, email_auth, base_url, template_path, user_path);

    // Fact Api
    if fact_api {
        routes::fact::attach(&mut server);
    }

    server.start();
}
