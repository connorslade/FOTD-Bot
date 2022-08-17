use std::{fs, sync::Arc};

use afire::{Content, Method, Response, Server};

use crate::App;

pub fn attach(server: &mut Server, app: Arc<App>) {
    let aapp = app.clone();
    server.route(Method::GET, "/api/fact", move |_req| {
        Response::new().text(aapp.fact.lock()).content(Content::TXT)
    });

    let aapp = app.clone();
    server.route(Method::GET, "/fact", move |req| {
        if let Some(i) = req.header("User-Agent") {
            if i.contains("ScriptableWidget") {
                return Response::new().text(aapp.fact.lock()).content(Content::TXT);
            }
        }

        let file = fs::read_to_string("./data/template/fact.html")
            .unwrap()
            .replace("{{FACT}}", app.fact.lock().as_str());
        Response::new().text(file).content(Content::HTML)
    });
}
