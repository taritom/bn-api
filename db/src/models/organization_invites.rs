use chrono::{Duration, NaiveDateTime, Utc};
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use diesel::sql_types::{Text, Timestamp};
use models::*;
use schema::organization_invites;
use utils::errors::ConvertToDatabaseError;
use utils::errors::{DatabaseError, ErrorCode};
use uuid::Uuid;
use validator::Validate;

const INVITE_EXPIRATION_PERIOD_IN_DAYS: i64 = 7;

#[derive(
    Associations,
    Insertable,
    Queryable,
    Identifiable,
    PartialEq,
    Debug,
    Clone,
    Serialize,
    Deserialize,
    AsChangeset,
    QueryableByName,
)]
#[belongs_to(Organization, foreign_key = "organization_id")]
#[belongs_to(User, foreign_key = "inviter_id")]
#[table_name = "organization_invites"]
pub struct OrganizationInvite {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub inviter_id: Uuid,
    pub user_email: String,
    pub created_at: NaiveDateTime,
    pub security_token: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub accepted: Option<i16>,
    pub updated_at: NaiveDateTime,
    pub sent_invite: bool,
}

#[derive(Insertable, PartialEq, Debug, Deserialize, Validate)]
#[table_name = "organization_invites"]
pub struct NewOrganizationInvite {
    pub organization_id: Uuid,
    pub inviter_id: Uuid,
    #[validate(email)]
    pub user_email: String,
    pub security_token: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Queryable, Serialize, QueryableByName)]
pub struct DisplayInvite {
    #[sql_type = "Text"]
    pub organization_name: String,
    #[sql_type = "Text"]
    pub inviter_name: String,
    #[sql_type = "Timestamp"]
    pub expires_at: NaiveDateTime,
}

impl NewOrganizationInvite {
    pub fn commit(&mut self, conn: &PgConnection) -> Result<OrganizationInvite, DatabaseError> {
        self.security_token = Some(Uuid::new_v4());
        self.validate()?;
        let res = diesel::insert_into(organization_invites::table)
            .values(&*self)
            .get_result(conn);
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
            user_id,
        }
    }

    pub fn change_invite_status(
        &self,
        change_status: i16,
        conn: &PgConnection,
    ) -> Result<OrganizationInvite, DatabaseError> {
        let null: Option<Uuid> = None; //this here so the compiler can infer the type of None
        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update organization invite table",
            diesel::update(self)
                .set((
                    organization_invites::security_token.eq(null),
                    organization_invites::accepted.eq(change_status),
                    organization_invites::updated_at.eq(dsl::now),
                )).get_result(conn),
        )
    }

    pub fn accept_invite(&self, conn: &PgConnection) -> Result<OrganizationInvite, DatabaseError> {
        self.change_invite_status(1, conn)
    }

    pub fn decline_invite(&self, conn: &PgConnection) -> Result<OrganizationInvite, DatabaseError> {
        self.change_invite_status(0, conn)
    }

    pub fn get_invite_display(
        token: &Uuid,
        conn: &PgConnection,
    ) -> Result<DisplayInvite, DatabaseError> {
        let expiry_date = Utc::now().naive_utc() - Duration::days(INVITE_EXPIRATION_PERIOD_IN_DAYS);
        let query = r#"
                SELECT
                    CONCAT(users.first_name, ' ',  users.last_name) AS inviter_name,
                    organizations.name AS organization_name,
                    organization_invites.created_at + INTERVAL '$1' day AS expires_at
                FROM organization_invites
                LEFT JOIN users ON (users.id = organization_invites.inviter_id)
                LEFT JOIN organizations ON (organizations.id = organization_invites.organization_id)
                WHERE
                    organization_invites.security_token = $2
                    AND organization_invites.created_at > $3
                    AND organization_invites.accepted is NULL;"#;

        diesel::sql_query(query)
            .bind::<diesel::sql_types::BigInt, _>(INVITE_EXPIRATION_PERIOD_IN_DAYS)
            .bind::<diesel::sql_types::Uuid, _>(token)
            .bind::<diesel::sql_types::Timestamp, _>(expiry_date)
            .get_result(conn)
            .to_db_error(ErrorCode::QueryError, "Cannot find organization invite")
    }

    pub fn get_invite_details(
        token: &Uuid,
        conn: &PgConnection,
    ) -> Result<OrganizationInvite, DatabaseError> {
        let expiry_date = Utc::now().naive_utc() - Duration::days(INVITE_EXPIRATION_PERIOD_IN_DAYS);
        DatabaseError::wrap(
            ErrorCode::AccessError,
            "No valid token found",
            organization_invites::table
                .filter(organization_invites::accepted.is_null())
                .filter(organization_invites::security_token.eq(token))
                .filter(organization_invites::created_at.gt(expiry_date))
                .get_result(conn),
        )
    }

    pub fn find_active_invite_by_email(
        email: &String,
        conn: &PgConnection,
    ) -> Result<Option<OrganizationInvite>, DatabaseError> {
        organization_invites::table
            .filter(organization_invites::user_email.eq(email))
            .filter(organization_invites::security_token.is_not_null())
            .first::<OrganizationInvite>(conn)
            .optional()
            .to_db_error(ErrorCode::QueryError, "Cannot find organization invite")
    }

    pub fn change_sent_status(
        &self,
        sent_status: bool,
        conn: &PgConnection,
    ) -> Result<OrganizationInvite, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update region",
            diesel::update(self)
                .set(organization_invites::sent_invite.eq(sent_status))
                .get_result(conn),
        )
    }
}
