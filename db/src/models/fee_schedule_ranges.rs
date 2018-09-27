use chrono::NaiveDateTime;
use diesel::prelude::*;
use schema::fee_schedule_ranges;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Debug, Queryable, Serialize, Deserialize, Clone, PartialEq)]
pub struct FeeScheduleRange {
    #[allow(dead_code)]
    pub id: Uuid,
    #[allow(dead_code)]
    fee_schedule_id: Uuid,
    pub min_price: i64,
    pub fee_in_cents: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct NewFeeScheduleRange {
    pub min_price: i64,
    pub fee_in_cents: i64,
}

impl FeeScheduleRange {
    pub fn find(id: Uuid, conn: &PgConnection) -> Result<FeeScheduleRange, DatabaseError> {
        fee_schedule_ranges::table
            .find(id)
            .first::<FeeScheduleRange>(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading fee schedule range")
    }
}
