use crate::{App, Arc};

use afire::Server;

mod fact;

pub fn attach(server: &mut Server, app: Arc<App>) {
    fact::attach(server, app);
}
