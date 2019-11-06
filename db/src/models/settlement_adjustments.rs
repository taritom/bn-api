use chrono::NaiveDateTime;
use diesel;
use diesel::prelude::*;
use models::*;
use schema::settlement_adjustments;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(AsChangeset, Clone, Debug, Deserialize, Identifiable, PartialEq, Queryable, QueryableByName, Serialize)]
#[table_name = "settlement_adjustments"]
pub struct SettlementAdjustment {
    pub id: Uuid,
    pub settlement_id: Uuid,
    pub amount_in_cents: i64,
    pub note: Option<String>,
    pub settlement_adjustment_type: SettlementAdjustmentTypes,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl SettlementAdjustment {
    pub fn create(
        settlement_id: Uuid,
        settlement_adjustment_type: SettlementAdjustmentTypes,
        note: Option<String>,
        amount_in_cents: i64,
    ) -> NewSettlementAdjustment {
        NewSettlementAdjustment {
            settlement_id,
            amount_in_cents,
            note,
            settlement_adjustment_type,
        }
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<SettlementAdjustment, DatabaseError> {
        settlement_adjustments::table
            .filter(settlement_adjustments::id.eq(id))
            .first(conn)
            .to_db_error(ErrorCode::QueryError, "Could not load Settlement Adjustment")
    }

    pub fn destroy(self, conn: &PgConnection) -> Result<usize, DatabaseError> {
        diesel::delete(settlement_adjustments::table.filter(settlement_adjustments::id.eq(self.id)))
            .execute(conn)
            .to_db_error(ErrorCode::DeleteError, "Error removing settlement adjustment")
    }
}

#[derive(Clone, Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "settlement_adjustments"]
pub struct NewSettlementAdjustment {
    pub settlement_id: Uuid,
    pub amount_in_cents: i64,
    pub note: Option<String>,
    pub settlement_adjustment_type: SettlementAdjustmentTypes,
}
impl NewSettlementAdjustment {
    pub fn commit(&self, conn: &PgConnection) -> Result<SettlementAdjustment, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new settlement adjustment",
            diesel::insert_into(settlement_adjustments::table)
                .values(self)
                .get_result::<SettlementAdjustment>(conn),
        )
    }
}
