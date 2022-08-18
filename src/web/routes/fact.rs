use std::{fs, sync::Arc};

use afire::{Content, Method, Response, Server};

use crate::App;

pub fn attach(server: &mut Server, app: Arc<App>) {
    server.route(Method::GET, "/fact", move |req| {
        // Backwards compatibility--
        // ikr
        if let Some(i) = req.header("User-Agent") {
            if i.contains("ScriptableWidget") {
                return Response::new().text(app.fact.lock()).content(Content::TXT);
            }
        }

        let mut fact = app.fact.lock().to_string();
        if let Some(i) = req.query.get("day") {
            let day = i.parse::<usize>().unwrap();
            fact = match app.database.lock().query_row(
                "SELECT fact FROM facts WHERE used = ?",
                [day],
                |row| row.get::<_, String>(0),
            ) {
                Ok(i) => i,
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    return Response::new().header("Location", "/fact").status(308)
                }
                Err(e) => panic!("{:?}", e),
            };
        }

        let file = fs::read_to_string("./data/template/fact.html")
            .unwrap()
            .replace("{{FACT}}", &fact);
        Response::new().text(file).content(Content::HTML)
    });
}
