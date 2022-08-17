use std::path::{Path, PathBuf};

use simple_config_parser as scp;

use crate::{
    misc::send_time::SendTime,
    web::Auth,
    webhook::{self, Webhook},
};

pub struct Config {
    // Web
    pub data_path: PathBuf,
    pub fact_api: bool,
    pub web_url: String,
    pub web_ip: String,
    pub web_port: u16,
    pub web_auth: Auth,
    pub web_server: bool,

    // Email
    pub server: String,
    pub sender_name: String,
    pub user_path: String,
    pub username: String,
    pub password: String,
    pub subject: String,

    // Misc
    pub fact_path: String,
    pub send_time: SendTime,
    pub webhooks: Vec<Webhook>,
}

impl From<scp::Config> for Config {
    fn from(cfg: scp::Config) -> Self {
        let fact_path = cfg_get(&cfg, "factPath");
        let data_path = Path::new(&cfg_get(&cfg, "dataPath")).to_path_buf();
        let webhooks = webhook::parse_config(&cfg);
        let user_path = cfg_get(&cfg, "emailListPath");
        let send_time = SendTime::from_str(&cfg_get(&cfg, "sendTime"));
        let subject = cfg_get(&cfg, "subject");
        let server = cfg_get(&cfg, "server");
        let sender_name = cfg_get(&cfg, "senderName");
        let username = cfg_get(&cfg, "username");
        let password = cfg_get(&cfg, "password");
        let fact_api = cfg_get(&cfg, "factApi").to_lowercase() == "true";
        let web_server = cfg_get(&cfg, "webServer").to_lowercase() == "true";
        let web_url = cfg_get(&cfg, "webUrl");
        let web_ip = cfg_get(&cfg, "webHost");
        let web_port = cfg_get(&cfg, "webPort").parse::<u16>().unwrap_or(8080);
        let web_auth = Auth::new(
            cfg_get(&cfg, "username"),
            cfg_get(&cfg, "password"),
            cfg_get(&cfg, "senderName"),
            cfg_get(&cfg, "server"),
        );

        Config {
            fact_path,
            fact_api,
            password,
            send_time,
            sender_name,
            server,
            subject,
            data_path,
            user_path,
            username,
            web_auth,
            web_ip,
            web_port,
            web_server,
            web_url,
            webhooks,
        }
    }
}

fn cfg_get(cfg: &scp::Config, key: &str) -> String {
    cfg.get_str(key)
        .unwrap_or_else(|_| panic!("The key '{}' was not defined in config :/", key))
}
