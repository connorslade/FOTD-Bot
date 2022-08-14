use simple_config_parser::Config;

pub enum Service {
    Discord,
    Slack,
}

pub struct Webhook {
    pub id: String,
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
    pub fn new(id: String, service: Service, token: String, channel: String) -> Webhook {
        Webhook {
            id,
            service,
            token,
            channel,
        }
    }

    pub fn send(&self, message: &str, title: &str) -> Option<()> {
        match self.service {
            Service::Discord => {
                let url = format!(
                    "https://discord.com/api/webhooks/{}/{}",
                    self.channel, self.token
                );

                match ureq::post(&url).set("Content-Type", "application/json").send_string(
                    &r#"{"embeds":[{"title":"{{TITLE}}","description":"{{MESSAGE}}","color":6053119}]}"#
                        .replace("{{TITLE}}", title)
                        .replace("{{MESSAGE}}", message),
                ) {
                   Ok(_) => Some(()),
                   Err(_) => None,
                }
            }

            Service::Slack => {
                let url = format!(
                    "https://hooks.slack.com/services/{}/{}",
                    self.channel, self.token
                );

                let to_send = r##"{"attachments":[{"title":"{{TITLE}}","text":"{{MESSAGE}}","color":"#5C5CFF"}]}"##
                    .replace("{{TITLE}}", title)
                    .replace("{{MESSAGE}}", message);

                match ureq::post(&url)
                    .set("Content-Type", "application/json")
                    .send_string(&to_send)
                {
                    Ok(_) => Some(()),
                    Err(_) => None,
                }
            }
        }
    }

    /// True => Valid Webhook
    /// False => Invalid Webhook
    pub fn verify(&self) -> bool {
        match self.service {
            Service::Discord => {
                let url = format!(
                    "https://discordapp.com/api/webhooks/{}/{}",
                    self.channel, self.token
                );

                ureq::get(&url).call().is_ok()
            }

            Service::Slack => {
                let url = format!(
                    "https://hooks.slack.com/services/{}/{}",
                    self.channel, self.token
                );

                if let Err(ureq::Error::Status(_, resp)) = ureq::get(&url).call() {
                    // Ok I know it seams weird that it has been verified if thare is an Invalid Payload.
                    // But the Token and service mut be valid to get to this point
                    return resp.into_string().unwrap() == "invalid_payload";
                }

                false
            }
        }
    }
}

pub fn parse_config(config: &Config) -> Vec<Webhook> {
    let mut out = Vec::new();
    for i in &config.data {
        if !i[0].starts_with("webhook_") {
            continue;
        }

        let id = i[0].split("webhook_").nth(1).unwrap_or("None").to_owned();
        if let Some(j) = parse_line(i[1].clone(), id) {
            out.push(j);
        }
    }
    out
}

fn parse_line(line: String, id: String) -> Option<Webhook> {
    let mut parts = line.split(',');

    let service = parts.next()?.trim();
    let channel = parts.next()?.trim().to_owned();
    let token = parts.next()?.trim().to_owned();

    Some(Webhook::new(
        id,
        Service::from_string(service)?,
        token,
        channel,
    ))
}

// TODO: Use a macro for the send function service options
