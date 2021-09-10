use lettre::message::header;
use lettre::transport::smtp::authentication::Credentials;
use lettre::Message;
use lettre::SmtpTransport;
use lettre::Transport;

// TODO: Use these errors...
/// Errors that can occur when sending an email.
pub enum EmailError {
    MessageBuild,
    Transport,
    Authentication,
}

// Impl debug for EmailError
impl std::fmt::Debug for EmailError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EmailError::MessageBuild => write!(f, "MessageBuild"),
            EmailError::Transport => write!(f, "Transport"),
            EmailError::Authentication => write!(f, "Authentication"),
        }
    }
}

pub struct Mailer {
    pub to: Vec<User>,
    pub from: User,
    pub subject: String,
    pub body: String,
    pub credentials: Creds,
    pub server: String,
    foreach: Option<Box<dyn Fn(User)>>,
}

pub struct User {
    pub email: String,
    pub name: String,
}

pub struct Creds {
    pub username: String,
    pub password: String,
}

/// Impl User
impl User {
    /// Make a new user
    pub fn new(email: String, name: String) -> User {
        User { email, name }
    }

    pub fn user_from_email(email: &str) -> User {
        let mut split = email.split('@');
        let name = split.next().unwrap();
        User::new(email.to_string(), name.to_string())
    }
}

// Impl to_string for User
impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} <{}>", self.name, self.email)
    }
}

/// Impl Debug for User
impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} <{}>", self.name, self.email)
    }
}

// Impl Clone for user
impl Clone for User {
    fn clone(&self) -> User {
        User::new(self.email.clone(), self.name.clone())
    }
}

/// Impl Mailer
impl Mailer {
    /// Make a new mailer
    pub fn new(
        to: Vec<User>,
        from: User,
        subject: &str,
        body: &str,
        server: &str,
        username: &str,
        password: &str,
    ) -> Mailer {
        Mailer {
            to,
            from,
            subject: subject.to_string(),
            body: body.to_string(),
            credentials: Creds {
                username: username.to_string(),
                password: password.to_string(),
            },
            server: server.to_string(),
            foreach: None,
        }
    }

    /// Send to all users as individual emails
    pub fn send_all(&self) -> Result<u32, EmailError> {
        let mut count = 0;
        for user in &self.to {
            // Run Foreach
            if let Some(f) = &self.foreach {
                f(user.clone());
            }

            // Build the message
            let email = match Message::builder()
                .from((&self.from.to_string()).parse().unwrap())
                .to(user.to_string().parse().unwrap())
                .subject(&self.subject)
                .header(header::ContentType::TEXT_HTML)
                .body(String::from(&self.body).replace("{{NAME}}", &user.name))
            {
                // lil bodge {}
                Ok(email) => email,
                Err(_) => return Err(EmailError::MessageBuild),
            };

            // Get credentials for mail server
            let creds = Credentials::new(
                (&self.credentials.username).clone(),
                (&self.credentials.password).clone(),
            );

            // Open a remote connection to the mail server
            let mailer = match SmtpTransport::relay(&self.server) {
                Ok(mailer) => mailer.credentials(creds).build(),
                Err(_) => return Err(EmailError::Authentication),
            };

            // Send the email
            match mailer.send(&email) {
                Ok(_) => count += 1,
                Err(_) => return Err(EmailError::Transport),
            }
        }
        Ok(count)
    }

    pub fn add_foreach(&mut self, f: Box<dyn Fn(User)>) {
        self.foreach = Some(f);
    }
}

pub struct Auth {
    username: String,
    password: String,
    name: String,
    server: String,
}

impl Auth {
    pub fn new(username: String, password: String, name: String, server: String) -> Auth {
        Auth {
            username,
            password,
            name,
            server,
        }
    }
}

impl Clone for Auth {
    fn clone(&self) -> Self {
        Auth {
            username: self.username.clone(),
            password: self.password.clone(),
            name: self.name.clone(),
            server: self.server.clone(),
        }
    }
}

pub fn quick_email(email_auth: &mut Auth, to: String, subject: String, body: String) -> Option<()> {
    // Get recipient
    let to = match to.parse() {
        Ok(to) => to,
        Err(_) => return None,
    };

    // Build the message
    let email = match Message::builder()
        .from(
            format!("{} <{}>", email_auth.name, email_auth.username)
                .parse()
                .unwrap(),
        )
        .to(to)
        .subject(subject)
        .header(header::ContentType::TEXT_HTML)
        .body(body)
    {
        // lil bodge {}
        Ok(email) => email,
        Err(_) => return None,
    };

    // Get credentials for mail server
    let creds = Credentials::new(email_auth.username.clone(), email_auth.password.clone());

    // Open a remote connection to the mail server
    let mailer = match SmtpTransport::relay(&email_auth.server) {
        Ok(mailer) => mailer.credentials(creds).build(),
        Err(_) => return None,
    };

    // Send the email
    match mailer.send(&email) {
        Ok(_) => Some(()),
        Err(_) => None,
    }
}
