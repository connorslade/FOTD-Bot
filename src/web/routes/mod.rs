use crate::{App, Arc};

use afire::Server;

mod api;
mod subscription;

pub fn attach(server: &mut Server, app: Arc<App>) {
    if app.config.fact_api {
        api::attach(server, app.clone());
    }

    subscription::attach(server, app);
}
