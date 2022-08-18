use std::sync::Arc;

use afire::{Content, Method, Response, Server};
use serde_json::json;

use crate::App;

pub fn attach(server: &mut Server, app: Arc<App>) {
    server.route(Method::GET, "/api/fact/random", move |_req| {
        let fact = app
            .database
            .lock()
            .query_row(
                "SELECT fact, used FROM facts WHERE used IS NOT NULL ORDER BY RANDOM()",
                [],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?)),
            )
            .unwrap();

        Response::new()
            .text(json!({"fact": fact.0, "date": fact.1}))
            .content(Content::JSON)
    });
}
