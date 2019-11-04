use chrono::prelude::*;
use diesel;
use diesel::prelude::*;
use models::*;
use schema::{refund_items, refunds};
use utils::errors::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Identifiable, Queryable, Serialize, Deserialize)]
pub struct Refund {
    pub id: Uuid,
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub reason: Option<String>,
    #[serde(skip_serializing)]
    pub settlement_id: Option<Uuid>,
}

impl Refund {
    pub fn create(order_id: Uuid, user_id: Uuid, reason: Option<String>) -> NewRefund {
        NewRefund {
            order_id,
            user_id,
            reason,
        }
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<Refund, DatabaseError> {
        refunds::table
            .filter(refunds::id.eq(id))
            .first(conn)
            .to_db_error(ErrorCode::QueryError, "Could not retrieve refund data")
    }

    pub fn items(&self, conn: &PgConnection) -> Result<Vec<RefundItem>, DatabaseError> {
        refund_items::table
            .filter(refund_items::refund_id.eq(self.id))
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Could not load refund items")
    }
}

#[derive(Insertable, Clone)]
#[table_name = "refunds"]
pub struct NewRefund {
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub reason: Option<String>,
}

impl NewRefund {
    pub fn commit(self, conn: &PgConnection) -> Result<Refund, DatabaseError> {
        let refund: Refund = diesel::insert_into(refunds::table)
            .values(&self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not insert refund record")?;

        let order = Order::find(refund.order_id, conn)?;
        DomainEvent::create(
            DomainEventTypes::OrderRefund,
            "Order refund created".to_string(),
            Tables::Orders,
            Some(refund.order_id),
            Some(order.on_behalf_of_user_id.unwrap_or(order.user_id)),
            None,
        )
        .commit(conn)?;

        Ok(refund)
    }
}
