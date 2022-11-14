use serde_json::{json, Value};
use simple_config_parser::Config;

#[derive(PartialEq, Eq)]
pub enum Service {
    Discord,
    Slack,
    GoogleChat,
}

pub struct Webhook {
    pub id: String,
    pub service: Service,

    pub channel: String,
    pub token: String,
    pub key: Option<String>,
}

impl Service {
    pub fn from_string(service: &str) -> Option<Service> {
        Some(match service.to_lowercase().as_str() {
            "discord" => Service::Discord,
            "slack" => Service::Slack,
            "google_chat" => Service::GoogleChat,
            _ => return None,
        })
    }
}

impl Webhook {
    pub fn send(&self, message: &str, title: &str) -> Option<()> {
        match self.service {
            Service::Discord => {
                let url = format!(
                    "https://discord.com/api/webhooks/{}/{}",
                    self.channel, self.token
                );

                ureq::post(&url)
                    .set("Content-Type", "application/json")
                    .send_string(
                        &r#"{"embeds":[{"title":"{{TITLE}}","description":"{{MESSAGE}}","color":6053119}]}"#
                            .replace("{{TITLE}}", title)
                            .replace("{{MESSAGE}}", message),
                    )
                    .ok()
                    .map(|_| ())
            }

            Service::Slack => {
                let url = format!(
                    "https://hooks.slack.com/services/{}/{}",
                    self.channel, self.token
                );

                let to_send = r##"{"attachments":[{"title":"{{TITLE}}","text":"{{MESSAGE}}","color":"#5C5CFF"}]}"##
                    .replace("{{TITLE}}", title)
                    .replace("{{MESSAGE}}", message);

                ureq::post(&url)
                    .set("Content-Type", "application/json")
                    .send_string(&to_send)
                    .ok()
                    .map(|_| ())
            }
            Service::GoogleChat => {
                let url = format!(
                    "https://chat.googleapis.com/v1/spaces/{}/messages?key={}&token={}",
                    self.channel,
                    self.key.as_ref().unwrap(),
                    self.token
                );

                let to_send =
                    json!({ "text": format!("*{}*\n```{}```", title, message) }).to_string();

                ureq::post(&url)
                    .set("Content-Type", "application/json")
                    .send_string(&to_send)
                    .ok()
                    .map(|_| ())
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
            Service::GoogleChat => {
                let url = format!(
                    "https://chat.googleapis.com/v1/spaces/{}/messages?key={}&token={}",
                    self.channel,
                    self.key.as_ref().unwrap(),
                    self.token
                );

                if let Err(ureq::Error::Status(_, resp)) = ureq::post(&url).send_string("{}") {
                    let raw = resp.into_string().unwrap();
                    let json = serde_json::from_str::<Value>(&raw).unwrap();
                    return json.get("error").and_then(|x| x.get("message"))
                        == Some(&Value::String("Message cannot be empty.".to_owned()));
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
    let service = Service::from_string(service)?;

    let channel = parts.next()?.trim().to_owned();
    let token = parts.next()?.trim().to_owned();
    let key = parts.next().map(|x| x.trim().to_owned());

    if key.is_none() && service == Service::GoogleChat {
        println!("[-] Google Chat webhooks need a key. Disableing hook.");
        return None;
    }

    Some(Webhook {
        id,
        service,
        token,
        channel,
        key,
    })
}
