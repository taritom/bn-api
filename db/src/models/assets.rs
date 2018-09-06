use chrono::prelude::*;
use diesel;
use diesel::prelude::*;
use models::AssetStatus;
use schema::assets;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Queryable, Identifiable)]
pub struct Asset {
    pub id: Uuid,
    blockchain_name: String,
    // TODO: This will be populated after it is created on the blockchain.
    blockchain_asset_id: Option<String>,
    status: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl Asset {
    pub fn create(blockchain_name: String) -> NewAsset {
        NewAsset {
            blockchain_name,
            status: AssetStatus::Unsynced.to_string(),
        }
    }
}

#[derive(Insertable)]
#[table_name = "assets"]
pub struct NewAsset {
    blockchain_name: String,
    status: String,
}

impl NewAsset {
    pub fn commit(self, conn: &PgConnection) -> Result<Asset, DatabaseError> {
        diesel::insert_into(assets::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create asset")
    }
}
