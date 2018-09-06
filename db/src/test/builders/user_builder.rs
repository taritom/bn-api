use diesel::prelude::*;
use models::User;
use rand::prelude::*;

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
        let x: u16 = random();
        UserBuilder {
            first_name: "Jeff".into(),
            last_name: "Wilco".into(),
            email: format!("jeff{}@tari.com", x).into(),
            phone: "555-555-5555".into(),
            password: "examplePassword".into(),
            connection,
        }
    }

    pub fn with_first_name(mut self, first_name: String) -> Self {
        self.first_name = first_name;
        self
    }

    pub fn with_last_name(mut self, last_name: String) -> Self {
        self.last_name = last_name;
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

    pub fn finish(&self) -> User {
        User::create(
            &self.first_name,
            &self.last_name,
            &self.email,
            &self.phone,
            &self.password,
        ).commit(self.connection)
            .unwrap()
    }
}
