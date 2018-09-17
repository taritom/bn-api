use chrono::NaiveDateTime;
use diesel::prelude::*;
use schema::wallets;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Identifiable, Queryable)]
pub struct Wallet {
    id: Uuid,
    user_id: Option<Uuid>,
    organization_id: Option<Uuid>,
    name: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl Wallet {
    pub fn find_default_wallet_for_user(
        user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Wallet, DatabaseError> {
        wallets::table
            .filter(wallets::user_id.eq(user_id))
            .first(conn)
            .to_db_error(ErrorCode::QueryError, "Could not find wallet for user")
    }
}
