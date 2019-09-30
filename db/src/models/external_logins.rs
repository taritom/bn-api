use chrono::prelude::*;
use diesel;
use diesel::dsl;
use diesel::prelude::*;

use models::domain_events::DomainEvent;
use models::DomainEventTypes::*;
use models::Tables;
use models::User;
use schema::external_logins;
use utils::errors::*;
use uuid::Uuid;

pub const FACEBOOK_SITE: &str = "facebook.com";

#[derive(Identifiable, Associations, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[belongs_to(User, foreign_key = "user_id")]
#[table_name = "external_logins"]
pub struct ExternalLogin {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: NaiveDateTime,
    pub site: String,
    pub access_token: String,
    pub external_user_id: String,
    pub updated_at: NaiveDateTime,
    pub scopes: Vec<String>,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "external_logins"]
pub struct NewExternalLogin {
    pub user_id: Uuid,
    pub site: String,
    pub access_token: String,
    pub external_user_id: String,
    pub scopes: Vec<String>,
}

impl NewExternalLogin {
    pub fn commit(
        self,
        current_user_id: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<ExternalLogin, DatabaseError> {
        let res = diesel::insert_into(external_logins::table)
            .values(&self)
            .get_result(conn);

        let res: ExternalLogin = DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new external login",
            res,
        )?;
        DomainEvent::create(ExternalLoginCreated, "External login created".to_string(),
            Tables::ExternalLogins, Some(res.id),current_user_id,
                            Some(json!({"user_id": self.user_id, "site": &self.site, "external_user_id": &self.external_user_id, "scopes": &self.scopes}))).commit(conn)?;
        Ok(res)
    }
}

impl ExternalLogin {
    pub fn create(
        external_user_id: String,
        site: String,
        user_id: Uuid,
        access_token: String,
        scopes: Vec<String>,
    ) -> NewExternalLogin {
        NewExternalLogin {
            external_user_id,
            site,
            user_id,
            access_token,
            scopes,
        }
    }

    pub fn find_for_site(
        user_id: Uuid,
        site: &str,
        conn: &PgConnection,
    ) -> Result<ExternalLogin, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading external login",
            external_logins::table
                .filter(external_logins::user_id.eq(user_id))
                .filter(external_logins::site.eq(site))
                .filter(external_logins::deleted_at.is_null())
                .first::<ExternalLogin>(conn),
        )
    }

    pub fn find_user(
        external_user_id: &str,
        site: &str,
        conn: &PgConnection,
    ) -> Result<Option<ExternalLogin>, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading external login",
            external_logins::table
                .filter(external_logins::external_user_id.eq(external_user_id))
                .filter(external_logins::site.eq(site))
                .filter(external_logins::deleted_at.is_null())
                .first::<ExternalLogin>(conn)
                .optional(),
        )
    }

    pub fn delete(
        self,
        current_user_id: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        let id = self.id;
        let data = json!({
        "external_user_id": self.external_user_id.clone(), "site": self.site.clone(), "user_id": self.user_id.clone(), "scopes": self.scopes.clone()
        });
        diesel::update(&self)
            .set((external_logins::deleted_at.eq(dsl::now)))
            .execute(conn)
            .to_db_error(ErrorCode::DeleteError, "Could not delete external login")?;
        DomainEvent::create(
            ExternalLoginDeleted,
            "External login deleted".to_string(),
            Tables::ExternalLogins,
            Some(id),
            current_user_id,
            Some(data),
        )
        .commit(conn)?;
        Ok(())
    }
}
