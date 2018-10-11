use chrono::NaiveDateTime;
use diesel;
use diesel::prelude::*;
use schema::wallets;
use std::default::Default;
use tari_client;
use utils::errors;
use utils::errors::*;
use uuid::Uuid;

#[derive(Identifiable, Queryable, Clone)]
pub struct Wallet {
    pub id: Uuid,
    user_id: Option<Uuid>,
    organization_id: Option<Uuid>,
    name: String,
    pub secret_key: String,
    pub public_key: String,
    default_flag: bool,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl Wallet {
    pub fn find(id: Uuid, conn: &PgConnection) -> Result<Wallet, DatabaseError> {
        wallets::table
            .filter(wallets::id.eq(id))
            .first(conn)
            .to_db_error(errors::ErrorCode::QueryError, "Could not find wallet")
    }

    pub fn create_for_user(user_id: Uuid, name: String, conn: &PgConnection) -> NewWallet {
        let (secret_key, public_key) = tari_client::cryptographic_keypair();
        let default_flag: bool;
        match Wallet::find_for_user(user_id, conn) {
            Ok(v) => if v.len() == 0 {
                default_flag = true;
            } else {
                default_flag = false;
            },
            Err(_e) => default_flag = true,
        };
        NewWallet {
            user_id: Some(user_id),
            name,
            secret_key: tari_client::convert_bytes_to_hexstring(&secret_key),
            public_key: tari_client::convert_bytes_to_hexstring(&public_key),
            default_flag,
            ..Default::default()
        }
    }

    pub fn create_for_organization(
        organization_id: Uuid,
        name: String,
        conn: &PgConnection,
    ) -> NewWallet {
        let (secret_key, public_key) = tari_client::cryptographic_keypair();
        let default_flag: bool;
        match Wallet::find_for_organization(organization_id, conn) {
            Ok(v) => if v.len() == 0 {
                default_flag = true;
            } else {
                default_flag = false;
            },
            Err(_e) => default_flag = true,
        };
        NewWallet {
            organization_id: Some(organization_id),
            name,
            secret_key: tari_client::convert_bytes_to_hexstring(&secret_key),
            public_key: tari_client::convert_bytes_to_hexstring(&public_key),
            default_flag,
            ..Default::default()
        }
    }

    pub fn find_for_user(user_id: Uuid, conn: &PgConnection) -> Result<Vec<Wallet>, DatabaseError> {
        wallets::table
            .filter(wallets::user_id.eq(user_id))
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Could not find wallets")
    }

    pub fn find_for_organization(
        organization_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<Wallet>, DatabaseError> {
        wallets::table
            .filter(wallets::organization_id.eq(organization_id))
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Could not find wallets")
    }

    pub fn find_default_for_user(
        user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Wallet, DatabaseError> {
        let wallets = Wallet::find_for_user(user_id, conn)?;
        let mut result_wallet;
        if wallets.len() > 0 {
            result_wallet = wallets[0].clone(); //Return first wallet if no default wallet found
            for wallet in wallets {
                if wallet.default_flag {
                    result_wallet = wallet;
                    break;
                }
            }
        } else {
            //Create default wallet for user
            result_wallet =
                Wallet::create_for_user(user_id, String::from("Default"), conn).commit(conn)?;
        }
        Ok(result_wallet)
    }

    pub fn find_default_for_organization(
        organization_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Wallet, DatabaseError> {
        let wallets = Wallet::find_for_organization(organization_id, conn)?;
        let mut result_wallet;
        if wallets.len() > 0 {
            result_wallet = wallets[0].clone(); //Return first wallet if no default wallet found
            for wallet in wallets {
                if wallet.default_flag {
                    result_wallet = wallet;
                    break;
                }
            }
        } else {
            //Create default wallet for org
            result_wallet =
                Wallet::create_for_organization(organization_id, String::from("Default"), conn)
                    .commit(conn)?;
        }
        Ok(result_wallet)
    }
}

#[derive(Default, Insertable)]
#[table_name = "wallets"]
pub struct NewWallet {
    user_id: Option<Uuid>,
    organization_id: Option<Uuid>,
    name: String,
    secret_key: String,
    public_key: String,
    default_flag: bool,
}

impl NewWallet {
    pub fn commit(self, conn: &PgConnection) -> Result<Wallet, DatabaseError> {
        diesel::insert_into(wallets::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create wallet")
    }
}
