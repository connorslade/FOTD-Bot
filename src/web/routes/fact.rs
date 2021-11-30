use afire::{Method, Response, Server};

use crate::FACT;

pub fn attach(server: &mut Server) {
    server.route(Method::GET, "/fact", |_req| {
        Response::new().text(unsafe { FACT.clone() }.unwrap_or_default())
    })
}
