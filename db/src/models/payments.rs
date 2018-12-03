use chrono::NaiveDateTime;
use diesel;
use diesel::expression::dsl;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use models::orders::Order;
use models::*;
use schema::payments;
use serde_json;
use utils::errors::*;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Identifiable, Queryable)]
pub struct Payment {
    pub id: Uuid,
    order_id: Uuid,
    created_by: Uuid,
    status: String,
    payment_method: String,
    amount: i64,
    provider: String,
    external_reference: Option<String>,
    raw_data: Option<serde_json::Value>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl Payment {
    pub(crate) fn create(
        order_id: Uuid,
        created_by: Uuid,
        status: PaymentStatus,
        method: PaymentMethods,
        provider: String,
        external_reference: Option<String>,
        amount: i64,
        raw_data: Option<serde_json::Value>,
    ) -> NewPayment {
        NewPayment {
            order_id,
            created_by,
            status: status.to_string(),
            payment_method: method.to_string(),
            provider,
            external_reference,
            amount,
            raw_data,
        }
    }

    pub fn mark_complete(
        &self,
        raw_data: serde_json::Value,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        diesel::update(
            payments::table.filter(
                payments::id
                    .eq(self.id)
                    .and(payments::updated_at.eq(self.updated_at)),
            ),
        ).set((
            payments::status.eq(PaymentStatus::Completed.to_string()),
            payments::updated_at.eq(dsl::now),
        )).execute(conn)
        .to_db_error(
            ErrorCode::UpdateError,
            "Could not change the status of payment to completed.",
        )?;

        println!("Saved payment");
        DomainEvent::create(
            DomainEventTypes::PaymentCompleted,
            "Payment was completed".to_string(),
            Tables::Payments,
            Some(self.id),
            Some(raw_data),
        ).commit(conn)?;

        println!("Domain Action created");

        self.order(conn)?.complete_if_fully_paid(conn)?;

        Ok(())
    }

    fn order(&self, conn: &PgConnection) -> Result<Order, DatabaseError> {
        use schema::*;
        orders::table
            .find(self.order_id)
            .get_result(conn)
            .to_db_error(ErrorCode::QueryError, "Could not retrieve order")
    }
}

#[derive(Insertable)]
#[table_name = "payments"]
pub struct NewPayment {
    order_id: Uuid,
    created_by: Uuid,
    status: String,
    payment_method: String,
    external_reference: Option<String>,
    amount: i64,
    provider: String,
    raw_data: Option<serde_json::Value>,
}

impl NewPayment {
    pub(crate) fn commit(self, conn: &PgConnection) -> Result<Payment, DatabaseError> {
        let res: Payment = diesel::insert_into(payments::table)
            .values(&self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create payment")?;

        DomainEvent::create(
            DomainEventTypes::PaymentCreated,
            "Payment created".to_string(),
            Tables::Payments,
            Some(res.id),
            self.raw_data,
        ).commit(conn)?;
        Ok(res)
    }
}
