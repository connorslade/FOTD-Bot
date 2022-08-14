use std::sync::Arc;
use std::time::Duration;

use afire::Server;

pub use super::email::{quick_email, Auth};
use crate::{app::App, VERSION};
mod logger;
mod routes;

pub fn start(app: Arc<App>) {
    let mut server: Server = Server::new(&app.config.web_ip, app.config.web_port)
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
    routes::unsub::attach(&mut server, app.clone());

    // Process Sub requests
    routes::sub::attach(&mut server, app.clone());

    // Fact Api
    if app.config.fact_api {
        routes::fact::attach(&mut server, app);
    }

    server.start();
}
