use crate::{App, Arc};

use afire::Server;

mod fact;
mod fact_at;
mod fact_history;
mod fact_range;

pub fn attach(server: &mut Server, app: Arc<App>) {
    fact::attach(server, app.clone());
    fact_at::attach(server, app.clone());
    fact_history::attach(server, app.clone());
    fact_range::attach(server, app);
}
