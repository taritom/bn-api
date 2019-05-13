use chrono::prelude::*;
use diesel;
use diesel::prelude::*;
use schema::*;
use utils::errors::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Identifiable, Queryable, Serialize, Deserialize)]
pub struct RefundItem {
    pub id: Uuid,
    pub refund_id: Uuid,
    pub order_item_id: Uuid,
    pub quantity: i64,
    pub amount: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl RefundItem {
    pub fn create(
        refund_id: Uuid,
        order_item_id: Uuid,
        quantity: i64,
        amount: i64,
    ) -> NewRefundItem {
        NewRefundItem {
            refund_id,
            order_item_id,
            quantity,
            amount,
        }
    }
}

#[derive(Clone, Insertable)]
#[table_name = "refund_items"]
pub struct NewRefundItem {
    pub refund_id: Uuid,
    pub order_item_id: Uuid,
    pub quantity: i64,
    pub amount: i64,
}

impl NewRefundItem {
    pub fn commit(self, conn: &PgConnection) -> Result<RefundItem, DatabaseError> {
        diesel::insert_into(refund_items::table)
            .values(&self)
            .get_result(conn)
            .to_db_error(
                ErrorCode::InsertError,
                "Could not insert refund item record",
            )
    }
}
