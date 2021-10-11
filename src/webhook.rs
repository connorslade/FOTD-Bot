use ureq;

pub enum Service {
    Discord,
}

pub struct Webhook {
    pub service: Service,
    pub token: String,
    pub channel: String,
}

impl Service {
    pub fn from_string(service: &str) -> Option<Service> {
        match service.to_lowercase().as_str() {
            "discord" => Some(Service::Discord),
            _ => None,
        }
    }
}

impl Webhook {
    pub fn new(service: Service, token: String, channel: String) -> Webhook {
        Webhook {
            service,
            token,
            channel,
        }
    }

    pub fn send(&self, message: String, title: String) -> Option<()> {
        match self.service {
            Service::Discord => {
                let url = format!(
                    "https://discord.com/api/webhooks/{}/{}",
                    self.channel, self.token
                );
                    
                match ureq::post(&url).set("Content-Type", "application/json").send_string(
                    &r#"{"embeds":[{"title":"{{TITLE}}","description":"{{MESSAGE}}","color":6053119}]}"#
                        .replace("{{TITLE}}", &title)
                        .replace("{{MESSAGE}}", &message),
                ) {
                   Ok(_) => return Some(()),
                   Err(_) => return None,
                };
            }
        }
    }

    pub fn verify(&self) -> Option<()> {
        match self.service {
            Service::Discord => {
                let url = format!(
                    "https://discordapp.com/api/webhooks/{}/{}",
                    self.channel, self.token
                );

                match ureq::get(&url).call() {
                    Ok(_) => Some(()),
                    Err(_) => None,
                }
            }
        }
    }
}