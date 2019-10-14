use chrono::{NaiveDateTime, Utc};
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use models::User;
use schema::users;
use utils::errors::{DatabaseError, ErrorCode};
use utils::passwords::PasswordHash;
use uuid::Uuid;

const PASSWORD_RESET_EXPIRATION_PERIOD_IN_DAYS: i64 = 1;

#[derive(AsChangeset)]
#[changeset_options(treat_none_as_null = "true")]
#[table_name = "users"]
pub struct PasswordReset {
    pub password_reset_token: Option<Uuid>,
    pub password_reset_requested_at: Option<NaiveDateTime>,
}

pub trait PasswordResetable {
    fn has_valid_password_reset_token(&self) -> bool;
    fn find_by_password_reset_token(
        password_reset_token: &Uuid,
        conn: &PgConnection,
    ) -> Result<User, DatabaseError>;
    fn create_password_reset_token(&self, conn: &PgConnection) -> Result<User, DatabaseError>;
    fn consume_password_reset_token(
        token: &Uuid,
        password: &str,
        conn: &PgConnection,
    ) -> Result<User, DatabaseError>;
}

impl PasswordResetable for User {
    fn consume_password_reset_token(
        token: &Uuid,
        password: &str,
        conn: &PgConnection,
    ) -> Result<User, DatabaseError> {
        use schema::users::dsl::*;

        let user = User::find_by_password_reset_token(token, conn)?;

        if user.has_valid_password_reset_token() {
            let hash = PasswordHash::generate(password, None);
            let now = Utc::now().naive_utc();

            DatabaseError::wrap(
                ErrorCode::UpdateError,
                "Could not save new password for user",
                diesel::update(users.filter(id.eq(user.id)))
                    .set((
                        hashed_pw.eq(&hash.to_string()),
                        password_modified_at.eq(now),
                        updated_at.eq(dsl::now),
                        PasswordReset {
                            password_reset_token: None,
                            password_reset_requested_at: None,
                        },
                    ))
                    .get_result(conn),
            )
        } else {
            Err(DatabaseError::new(
                ErrorCode::InternalError,
                Some("Password reset token is expired".to_string()),
            ))
        }
    }

    fn find_by_password_reset_token(
        password_reset_token: &Uuid,
        conn: &PgConnection,
    ) -> Result<User, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading user",
            users::table
                .filter(users::password_reset_token.eq(password_reset_token))
                .first::<User>(conn),
        )
    }

    fn has_valid_password_reset_token(&self) -> bool {
        match self.password_reset_requested_at {
            Some(password_reset_requested_at) => {
                let now = Utc::now().naive_utc();
                now.signed_duration_since(password_reset_requested_at)
                    .num_days()
                    < PASSWORD_RESET_EXPIRATION_PERIOD_IN_DAYS
            }
            None => false,
        }
    }

    fn create_password_reset_token(&self, conn: &PgConnection) -> Result<User, DatabaseError> {
        let data = PasswordReset {
            password_reset_token: Some(Uuid::new_v4()),
            password_reset_requested_at: Some(Utc::now().naive_utc()),
        };

        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not create token for resetting password",
            diesel::update(self)
                .set((data, users::updated_at.eq(dsl::now)))
                .get_result(conn),
        )
    }
}
