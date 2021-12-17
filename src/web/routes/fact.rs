use std::fs;

use afire::{Header, Method, Response, Server};

use crate::FACT;

pub fn attach(server: &mut Server) {
    server.route(Method::GET, "/api/fact", |_req| {
        Response::new().text(unsafe { FACT.clone() }.unwrap_or_default())
    });

    server.route(Method::GET, "/fact", |req| {
        if let Some(i) = req.header("User-Agent") {
            if i.contains("ScriptableWidget") {
                return Response::new().text(unsafe { FACT.clone() }.unwrap_or_default());
            }
        }

        let file = fs::read_to_string("./data/template/fact.html")
            .unwrap()
            .replace(
                "{{FACT}}",
                &unsafe { FACT.clone() }
                    .unwrap_or_else(|| "There will be new a fact tomorrow.".to_owned()),
            );
        Response::new()
            .text(file)
            .header(Header::new("Content-Type", "text/html"))
    });
}
