use crate::{App, Arc};

use afire::Server;

mod sub;
mod sub_confirm;
mod unsub;
mod unsub_confirm;

pub fn attach(server: &mut Server, app: Arc<App>) {
    sub::attach(server, app.clone());
    sub_confirm::attach(server, app.clone());
    unsub::attach(server, app.clone());
    unsub_confirm::attach(server, app);
}
