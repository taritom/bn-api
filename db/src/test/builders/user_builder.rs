use diesel::prelude::*;
use models::User;
use uuid::Uuid;

pub struct UserBuilder<'a> {
    first_name: String,
    last_name: String,
    email: String,
    phone: String,
    password: String,
    connection: &'a PgConnection,
}

impl<'a> UserBuilder<'a> {
    pub fn new(connection: &'a PgConnection) -> Self {
        let x = Uuid::new_v4();
        UserBuilder {
            first_name: "Jeff".into(),
            last_name: "Wilco".into(),
            email: format!("jeff{}@tari.com", x).into(),
            phone: "555-555-5555".into(),
            password: "examplePassword".into(),
            connection,
        }
    }

    pub fn with_first_name(mut self, first_name: &str) -> Self {
        self.first_name = first_name.to_string();
        self
    }

    pub fn with_last_name(mut self, last_name: &str) -> Self {
        self.last_name = last_name.to_string();
        self
    }

    pub fn with_password(mut self, password: String) -> Self {
        self.password = password;
        self
    }

    pub fn with_email(mut self, email: String) -> Self {
        self.email = email;
        self
    }

    pub fn with_phone(mut self, phone: String) -> Self {
        self.phone = phone;
        self
    }

    pub fn finish(&self) -> User {
        User::create(
            Some(self.first_name.to_string()),
            Some(self.last_name.to_string()),
            Some(self.email.to_string()),
            Some(self.phone.to_string()),
            &self.password,
        )
        .commit(None, self.connection)
        .unwrap()
    }
}
