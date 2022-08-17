use std::sync::Arc;
use std::time::Duration;

use afire::Server;

pub use crate::misc::email::{quick_email, Auth};
use crate::{app::App, VERSION};
mod logger;
mod routes;
mod serve_static;

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
    serve_static::add_route(&mut server, app.clone());

    // Add routes
    routes::attach(&mut server, app);

    server.start();
}
