use dev::builders::*;
use diesel::prelude::*;
use models::{FeeSchedule, Organization, OrganizationEditableAttributes, OrganizationUser, User};
use rand::prelude::*;
use uuid::Uuid;

pub struct OrganizationBuilder<'a> {
    name: String,
    owner_user_id: Option<Uuid>,
    members: Vec<Uuid>,
    connection: &'a PgConnection,
    fee_schedule: Option<FeeSchedule>,
    use_address: bool,
}

impl<'a> OrganizationBuilder<'a> {
    pub fn new(connection: &'a PgConnection) -> OrganizationBuilder {
        let x: u16 = random();
        OrganizationBuilder {
            name: format!("test org{}", x).into(),
            owner_user_id: None,
            members: Vec::new(),
            fee_schedule: None,
            connection,
            use_address: false,
        }
    }

    pub fn with_owner(mut self, user: &User) -> OrganizationBuilder<'a> {
        self.owner_user_id = Some(user.id.clone());
        self
    }

    pub fn with_user(mut self, user: &User) -> OrganizationBuilder<'a> {
        self.members.push(user.id.clone());
        self
    }

    pub fn with_address(mut self) -> OrganizationBuilder<'a> {
        self.use_address = true;
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_fee_schedule(mut self, fee_schedule: &FeeSchedule) -> OrganizationBuilder<'a> {
        self.fee_schedule = Some(fee_schedule.clone());
        self
    }

    pub fn finish(&self) -> Organization {
        let mut organization = Organization::create(
            self.owner_user_id
                .or_else(|| Some(UserBuilder::new(self.connection).finish().id))
                .unwrap(),
            &self.name,
        ).commit(self.connection)
            .unwrap();

        if self.fee_schedule.is_some() {
            let _ =
                organization.add_fee_schedule(&self.fee_schedule.clone().unwrap(), self.connection);
        }

        for user_id in self.members.clone() {
            OrganizationUser::create(organization.id, user_id)
                .commit(self.connection)
                .unwrap();
        }

        if self.use_address {
            let mut attrs: OrganizationEditableAttributes = Default::default();

            attrs.address = Some(<String>::from("Test Address"));
            attrs.city = Some(<String>::from("Test Address"));
            attrs.state = Some(<String>::from("Test state"));
            attrs.country = Some(<String>::from("Test country"));
            attrs.postal_code = Some(<String>::from("0124"));
            attrs.phone = Some(<String>::from("+27123456789"));
            organization = organization.update(attrs, self.connection).unwrap();
        }
        organization
    }
}
