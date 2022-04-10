use std::time::Duration;

use afire::Server;

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
    let mut server: Server = Server::new(ip, port)
        // Add default headers
        .default_header("X-Frame-Options", "DENY")
        .default_header("X-Content-Type-Options", "nosniff")
        .default_header("X-Version", &format!("FOTD-BOT/{}", VERSION))
        // Set socket timeout
        .socket_timeout(Duration::from_secs(1));

    // Add Custom Logger
    logger::attach(&mut server);

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
