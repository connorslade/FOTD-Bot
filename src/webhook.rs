use ureq;

pub enum Service {
    Discord,
    Slack,
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
            "slack" => Some(Service::Slack),
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

            Service::Slack => {
                let url = format!(
                    "https://hooks.slack.com/services/{}/{}",
                    self.channel, self.token
                );

                let to_send = r##"{"attachments":[{"title":"{{TITLE}}","text":"{{MESSAGE}}","color":"#5C5CFF"}]}"##
                    .replace("{{TITLE}}", &title)
                    .replace("{{MESSAGE}}", &message);

                match ureq::post(&url)
                    .set("Content-Type", "application/json")
                    .send_string(&to_send)
                {
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

            Service::Slack => {
                let url = format!(
                    "https://hooks.slack.com/services/{}/{}",
                    self.channel, self.token
                );

                match ureq::get(&url).call() {
                    Err(ureq::Error::Status(_, resp)) => {
                        // Ok I know it seams weird that it has been verified if thare is an Invalid Payload.
                        // But the Token and service mut be valid to get to this point
                        if resp.into_string().unwrap() == "invalid_payload" {
                            return Some(());
                        }
                    }
                    Ok(_) => {}
                    Err(_) => {}
                };
                None
            }
        }
    }
}

// TODO: Use a macro for the send function service options
