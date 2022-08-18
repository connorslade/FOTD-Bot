use std::sync::Arc;

use afire::{Content, Method, Response, Server};

use crate::App;

pub fn attach(server: &mut Server, app: Arc<App>) {
    server.route(Method::GET, "/api/fact", move |_req| {
        Response::new().text(app.fact.lock()).content(Content::TXT)
    });
}
