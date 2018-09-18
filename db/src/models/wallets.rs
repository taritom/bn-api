use chrono::NaiveDateTime;
use diesel;
use diesel::prelude::*;
use schema::wallets;
use std::default::Default;
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
    pub fn create_for_user(user_id: Uuid, name: String) -> NewWallet {
        NewWallet {
            user_id: Some(user_id),
            name,
            ..Default::default()
        }
    }

    pub fn create_for_organization(organization_id: Uuid, name: String) -> NewWallet {
        NewWallet {
            organization_id: Some(organization_id),
            name,
            ..Default::default()
        }
    }

    pub fn find_for_user(user_id: Uuid, conn: &PgConnection) -> Result<Vec<Wallet>, DatabaseError> {
        wallets::table
            .filter(wallets::user_id.eq(user_id))
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Could not find wallets")
    }
}

#[derive(Default, Insertable)]
#[table_name = "wallets"]
pub struct NewWallet {
    user_id: Option<Uuid>,
    organization_id: Option<Uuid>,
    name: String,
}

impl NewWallet {
    pub fn commit(self, conn: &PgConnection) -> Result<Wallet, DatabaseError> {
        diesel::insert_into(wallets::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create wallet")
    }
}
