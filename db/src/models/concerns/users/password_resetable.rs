use chrono::{NaiveDateTime, Utc};
use db::Connectable;
use diesel;
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
        conn: &Connectable,
    ) -> Result<User, DatabaseError>;
    fn create_password_reset_token(&self, conn: &Connectable) -> Result<User, DatabaseError>;
    fn consume_password_reset_token(
        token: &Uuid,
        password: &str,
        conn: &Connectable,
    ) -> Result<User, DatabaseError>;
}

impl PasswordResetable for User {
    fn consume_password_reset_token(
        token: &Uuid,
        password: &str,
        conn: &Connectable,
    ) -> Result<User, DatabaseError> {
        use schema::users::dsl::*;

        let result = User::find_by_password_reset_token(token, conn);
        match result {
            Ok(user) => {
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
                                PasswordReset {
                                    password_reset_token: None,
                                    password_reset_requested_at: None,
                                },
                            ))
                            .get_result(conn.get_connection()),
                    )
                } else {
                    Err(DatabaseError::new(
                        ErrorCode::InternalError,
                        Some("Password reset token is expired"),
                    ))
                }
            }
            Err(e) => Err(e),
        }
    }

    fn find_by_password_reset_token(
        password_reset_token: &Uuid,
        conn: &Connectable,
    ) -> Result<User, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading user",
            users::table
                .filter(users::password_reset_token.eq(password_reset_token))
                .first::<User>(conn.get_connection()),
        )
    }

    fn has_valid_password_reset_token(&self) -> bool {
        match self.password_reset_requested_at {
            Some(password_reset_requested_at) => {
                let now = Utc::now().naive_utc();
                now.signed_duration_since(password_reset_requested_at)
                    .num_days() < PASSWORD_RESET_EXPIRATION_PERIOD_IN_DAYS
            }
            None => false,
        }
    }

    fn create_password_reset_token(&self, conn: &Connectable) -> Result<User, DatabaseError> {
        let data = PasswordReset {
            password_reset_token: Some(Uuid::new_v4()),
            password_reset_requested_at: Some(Utc::now().naive_utc()),
        };

        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not create token for resetting password",
            diesel::update(self)
                .set(data)
                .get_result(conn.get_connection()),
        )
    }
}
