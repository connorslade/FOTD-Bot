use std::sync::Arc;

use afire::{Content, Method, Response, Server};
use serde_json::json;

use crate::App;

pub fn attach(server: &mut Server, app: Arc<App>) {
    server.route(Method::GET, "/api/fact/{epoch}", move |req| {
        let epoch = match req.path_param("epoch").unwrap().parse::<usize>() {
            Ok(i) => i,
            Err(_) => return Response::new().text("Epoch must be a USIZE").status(400),
        };

        let day = epoch / (60 * 60 * 24);

        let fact = match app.database.lock().query_row(
            "SELECT fact FROM facts WHERE used = ?",
            [day],
            |row| row.get::<_, String>(0),
        ) {
            Ok(i) => i,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Response::new().text("No fact for this day").status(400)
            }
            Err(e) => panic!("{:?}", e),
        };

        Response::new()
            .text(json!({"fact": fact, "date": day}))
            .content(Content::JSON)
    });
}
