use std::sync::Arc;

use afire::{Content, Method, Response, Server};
use serde_json::json;

use crate::App;

pub fn attach(server: &mut Server, app: Arc<App>) {
    server.route(Method::GET, "/api/fact/history", move |req| {
        let page = match req.query.get("page") {
            Some(i) => i.parse().unwrap(),
            None => 0,
        };
        let page_size = match req.query.get("size") {
            Some(i) => i.parse().unwrap(),
            None => 20,
        };

        let db = app.database.lock();
        let mut stmt = db
            .prepare("SELECT fact, used FROM facts WHERE used IS NOT NULL LIMIT ? OFFSET ?")
            .unwrap();

        let facts = stmt
            .query_map([page_size, page_size * page], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?))
            })
            .unwrap()
            .map(|x| x.unwrap())
            .map(|x| json!({"fact": x.0, "date": x.1}))
            .collect::<Vec<_>>();

        Response::new()
            .text(json!({"page": page, "end": facts.len() < page_size, "facts": vec![facts[0].clone();20]}))
            .content(Content::JSON)
    });
}
