use bigneon_db::models::{Organization, OrganizationInvite, User};
use support::project::TestProject;
use uuid::Uuid;
extern crate chrono;
use chrono::NaiveDateTime;
use support::organization_invite_builder::chrono::prelude::*;

pub struct OrgInviteBuilder<'a> {
    organization_id: Option<Uuid>,
    invitee_id: Option<Uuid>,
    user_email: String,
    create_at: NaiveDateTime,
    security_token: Option<Uuid>,
    user_id: Option<Uuid>,
    status_change_at: Option<NaiveDateTime>,
    accepted: Option<i16>,
    test_project: &'a TestProject,
}

impl<'a> OrgInviteBuilder<'a> {
    pub fn new(test_project: &TestProject) -> OrgInviteBuilder {
        OrgInviteBuilder {
            organization_id: None,
            invitee_id: None,
            user_email: "test@test.com".into(),
            create_at: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
            security_token: Some(Uuid::new_v4()),
            test_project: &test_project,
            user_id: None,
            status_change_at: None,
            accepted: None,
        }
    }

    pub fn with_org(mut self, org: &Organization) -> OrgInviteBuilder<'a> {
        self.organization_id = Some(org.id.clone());
        self
    }

    pub fn with_invitee(mut self, invitee: &User) -> OrgInviteBuilder<'a> {
        self.invitee_id = Some(invitee.id.clone());
        self
    }

    pub fn link_to_user(mut self, user: &User) -> OrgInviteBuilder<'a> {
        self.user_id = Some(user.id.clone());
        self
    }

    pub fn update_status_changed(mut self, date: &NaiveDateTime) -> OrgInviteBuilder<'a> {
        self.status_change_at = Some(date.clone());
        self
    }

    pub fn accepted(mut self, status: bool) -> OrgInviteBuilder<'a> {
        self.accepted = Some(status as i16);
        self
    }

    pub fn finish(&self) -> OrganizationInvite {
        let orginvite = OrganizationInvite::create(
            self.organization_id.unwrap(),
            self.invitee_id.unwrap(),
            &self.user_email,
            self.user_id,
        ).commit(self.test_project)
            .unwrap();
        orginvite
    }
}
