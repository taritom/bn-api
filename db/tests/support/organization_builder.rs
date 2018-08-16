use bigneon_db::models::{Organization, OrganizationEditableAttributes, OrganizationUser, User};
use support::project::TestProject;
use uuid::Uuid;

pub struct OrganizationBuilder<'a> {
    name: String,
    owner_user_id: Option<Uuid>,
    member_user_id: Option<Uuid>,
    test_project: &'a TestProject,
    use_address: bool,
}

impl<'a> OrganizationBuilder<'a> {
    pub fn new(test_project: &TestProject) -> OrganizationBuilder {
        OrganizationBuilder {
            name: "test org".into(),
            owner_user_id: None,
            member_user_id: None,
            test_project: &test_project,
            use_address: false,
        }
    }

    pub fn with_owner(mut self, user: &User) -> OrganizationBuilder<'a> {
        self.owner_user_id = Some(user.id.clone());
        self
    }

    pub fn with_user(mut self, user: &User) -> OrganizationBuilder<'a> {
        self.member_user_id = Some(user.id.clone());
        self
    }

    pub fn with_address(mut self) -> OrganizationBuilder<'a> {
        self.use_address = true;
        self
    }

    pub fn finish(&self) -> Organization {
        let mut organization = Organization::create(self.owner_user_id.unwrap(), &self.name)
            .commit(self.test_project)
            .unwrap();
        if !self.member_user_id.is_none() {
            OrganizationUser::create(organization.id, self.member_user_id.unwrap())
                .commit(self.test_project)
                .unwrap();
        }
        if self.use_address {
            let mut attrs: OrganizationEditableAttributes = Default::default();

            attrs.address = Some(<String>::from("Test Address"));
            attrs.city = Some(<String>::from("Test Address"));
            attrs.state = Some(<String>::from("Test state"));
            attrs.country = Some(<String>::from("Test country"));
            attrs.zip = Some(<String>::from("0124"));
            attrs.phone = Some(<String>::from("+27123456789"));
            organization = organization.update(attrs, self.test_project).unwrap();
        }
        organization
    }
}
