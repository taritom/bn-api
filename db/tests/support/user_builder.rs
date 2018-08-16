use bigneon_db::models::User;
use support::project::TestProject;

use rand::prelude::*;

pub struct UserBuilder<'a> {
    first_name: String,
    last_name: String,
    email: String,
    phone: String,
    password: String,
    test_project: &'a TestProject,
}

impl<'a> UserBuilder<'a> {
    pub fn new(test_project: &'a TestProject) -> Self {
        let x: u8 = random();

        UserBuilder {
            first_name: "Jeff".into(),
            last_name: "Wilco".into(),
            email: format!("jeff{}@tari.com", x).into(),
            phone: "555-555-5555".into(),
            password: "examplePassword".into(),
            test_project,
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

    pub fn finish(&self) -> User {
        User::create(
            &self.first_name,
            &self.last_name,
            &self.email,
            &self.phone,
            &self.password,
        ).commit(self.test_project)
            .unwrap()
    }
}
