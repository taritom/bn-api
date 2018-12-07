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
    pub min_price_in_cents: i64,
    pub fee_in_cents: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub company_fee_in_cents: i64,
    pub client_fee_in_cents: i64,
}

#[derive(Serialize, Deserialize)]
pub struct NewFeeScheduleRange {
    pub min_price_in_cents: i64,
    pub company_fee_in_cents: i64,
    pub client_fee_in_cents: i64,
}

impl FeeScheduleRange {
    pub fn find(id: Uuid, conn: &PgConnection) -> Result<FeeScheduleRange, DatabaseError> {
        fee_schedule_ranges::table
            .find(id)
            .first::<FeeScheduleRange>(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading fee schedule range")
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DisplayFeeScheduleRange {
    pub id: Uuid,
    pub fee_schedule_id: Uuid,
    pub min_price_in_cents: i64,
    pub fee_in_cents: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<FeeScheduleRange> for DisplayFeeScheduleRange {
    fn from(fee_schedule_range: FeeScheduleRange) -> Self {
        DisplayFeeScheduleRange {
            id: fee_schedule_range.id,
            fee_schedule_id: fee_schedule_range.fee_schedule_id,
            min_price_in_cents: fee_schedule_range.min_price_in_cents,
            fee_in_cents: fee_schedule_range.fee_in_cents,
            created_at: fee_schedule_range.created_at,
            updated_at: fee_schedule_range.updated_at,
        }
    }
}
