use config::Config;
use lettre_email::EmailBuilder;

pub struct Mailer {
    config: Config,
    to: (String, String),
    from: (String, String),
    subject: String,
    body: String,
}

impl Mailer {
    pub fn new(
        config: Config,
        to: (String, String),
        from: (String, String),
        subject: String,
        body: String,
    ) -> Mailer {
        Mailer {
            config: config,
            to: to,
            from: from,
            subject: subject,
            body: body,
        }
    }

    pub fn to(&self) -> (String, String) {
        self.to.clone()
    }

    pub fn from(&self) -> (String, String) {
        self.from.clone()
    }

    pub fn subject(&self) -> String {
        self.subject.clone()
    }

    pub fn body(&self) -> String {
        self.body.clone()
    }

    pub fn deliver(&mut self) -> Result<String, String> {
        let email = EmailBuilder::new()
            .to(self.to())
            .from(self.from())
            .subject(self.subject())
            .text(self.body())
            .build()
            .unwrap();
        self.config.mail_transport.send(email)
    }
}
