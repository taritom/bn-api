use chrono::{Duration, NaiveDateTime, Utc};
use db::Connectable;
use diesel;
use diesel::prelude::*;
use schema::organization_invites;
use utils::errors::{DatabaseError, ErrorCode};
use uuid::Uuid;

const INVITE_EXPIRATION_PERIOD_IN_DAYS: i64 = 7;

#[derive(Insertable, Queryable, Identifiable, PartialEq, Debug, Clone, Serialize, Deserialize,
         AsChangeset, QueryableByName)]
#[table_name = "organization_invites"]
pub struct OrganizationInvite {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub inviter_id: Uuid,
    pub user_email: String,
    pub create_at: NaiveDateTime,
    pub security_token: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub status_change_at: Option<NaiveDateTime>,
    pub accepted: Option<i16>,
}

#[derive(Insertable, PartialEq, Debug, Deserialize)]
#[table_name = "organization_invites"]
pub struct NewOrganizationInvite {
    pub organization_id: Uuid,
    pub inviter_id: Uuid,
    pub user_email: String,
    pub security_token: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

impl NewOrganizationInvite {
    pub fn commit(&mut self, conn: &Connectable) -> Result<OrganizationInvite, DatabaseError> {
        self.security_token = Some(Uuid::new_v4());
        let res = diesel::insert_into(organization_invites::table)
            .values(&*self)
            .get_result(conn.get_connection());
        DatabaseError::wrap(ErrorCode::InsertError, "Could not create new invite", res)
    }
}

impl OrganizationInvite {
    pub fn create(
        org_id: Uuid,
        invitee_id: Uuid,
        email: &str,
        user_id: Option<Uuid>,
    ) -> NewOrganizationInvite {
        NewOrganizationInvite {
            organization_id: org_id,
            inviter_id: invitee_id,
            user_email: email.into(),
            security_token: None,
            user_id: user_id,
        }
    }

    pub fn change_invite_status(
        &self,
        change_status: i16,
        conn: &Connectable,
    ) -> Result<OrganizationInvite, DatabaseError> {
        let null: Option<Uuid> = None; //this here so the compiler can infer the type of None
        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update organization invite table",
            diesel::update(self)
                .set((
                    organization_invites::security_token.eq(null),
                    organization_invites::accepted.eq(change_status),
                    organization_invites::status_change_at.eq(Utc::now().naive_utc()),
                ))
                .get_result(conn.get_connection()),
        )
    }

    pub fn acepted_invite(&self, conn: &Connectable) -> Result<OrganizationInvite, DatabaseError> {
        self.change_invite_status(1, conn)
    }

    pub fn decline_invite(&self, conn: &Connectable) -> Result<OrganizationInvite, DatabaseError> {
        self.change_invite_status(0, conn)
    }

    pub fn get_invite_details(
        token: &Uuid,
        conn: &Connectable,
    ) -> Result<OrganizationInvite, DatabaseError> {
        let expiredate = Utc::now().naive_utc() - Duration::days(INVITE_EXPIRATION_PERIOD_IN_DAYS);
        DatabaseError::wrap(
            ErrorCode::AccessError,
            "No valid token found",
            diesel::sql_query(format!(
                "SELECT * FROM organization_invites WHERE security_token = '{}' AND create_at > '{}' AND accepted is NULL;"
                ,token, expiredate //todo convert to use the .bind 
            )).get_result(conn.get_connection()),
        )
    }

    pub fn add_user_id(
        &self,
        user_id: &Uuid,
        conn: &Connectable,
    ) -> Result<OrganizationInvite, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update organization invite table",
            diesel::update(self)
                .set((organization_invites::user_id.eq(user_id),))
                .get_result(conn.get_connection()),
        )
    }
}
