use chrono::NaiveDateTime;
use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use models::PaymentMethods;
use schema::payments;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Identifiable, Queryable)]
pub struct Payment {
    pub id: Uuid,
    order_id: Uuid,
    created_by: Uuid,
    payment_method: String,
    amount: i64,
    external_reference: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl Payment {
    pub fn create(
        order_id: Uuid,
        created_by: Uuid,
        method: PaymentMethods,
        external_reference: String,
        amount: i64,
    ) -> NewPayment {
        NewPayment {
            order_id,
            created_by,
            payment_method: method.to_string(),
            external_reference,
            amount,
        }
    }
}

#[derive(Insertable)]
#[table_name = "payments"]
pub struct NewPayment {
    order_id: Uuid,
    created_by: Uuid,
    payment_method: String,
    external_reference: String,
    amount: i64,
}

impl NewPayment {
    pub fn commit(self, conn: &PgConnection) -> Result<Payment, DatabaseError> {
        diesel::insert_into(payments::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create payment")
    }
}
