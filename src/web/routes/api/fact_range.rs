use std::sync::Arc;

use afire::{Content, Method, Response, Server};
use serde_json::json;

use crate::App;

pub fn attach(server: &mut Server, app: Arc<App>) {
    server.route(Method::GET, "/api/fact/{start}/{end}", move |req| {
        let start = match req.path_param("start").unwrap().parse::<usize>() {
            Ok(i) => i / (60 * 60 * 24),
            Err(_) => {
                return Response::new()
                    .text("Start epoch must be a USIZE")
                    .status(400)
            }
        };
        let end = match req.path_param("end").unwrap().parse::<usize>() {
            Ok(i) => i / (60 * 60 * 24),
            Err(_) => {
                return Response::new()
                    .text("End epoch must be a USIZE")
                    .status(400)
            }
        };

        if start > end {
            return Response::new()
                .text("End epoch must be after the start epoch.")
                .status(400);
        }

        let db = app.database.lock();
        let mut stmt = db
            .prepare(
                "SELECT fact, used FROM facts WHERE used IS NOT NULL AND used >= ? AND used <= ?",
            )
            .unwrap();

        let facts = stmt
            .query_map([start, end], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?))
            })
            .unwrap()
            .map(|x| x.unwrap())
            .map(|x| json!({"fact": x.0, "day": x.1}))
            .collect::<Vec<_>>();

        Response::new().text(json!(facts)).content(Content::JSON)
    });
}
