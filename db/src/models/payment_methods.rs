use chrono::prelude::*;
use diesel;
use diesel::prelude::*;
use schema::*;
use serde_json;
use utils::errors::*;
use uuid::Uuid;

#[derive(Identifiable, Queryable)]
pub struct PaymentMethod {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub is_default: bool,
    pub provider: String,
    pub provider_data: serde_json::Value,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl PaymentMethod {
    pub fn create(
        user_id: Uuid,
        name: String,
        is_default: bool,
        provider: String,
        data: serde_json::Value,
    ) -> NewPaymentMethod {
        NewPaymentMethod {
            user_id,
            name,
            is_default,
            provider,
            provider_data: data,
        }
    }
}

#[derive(Insertable)]
#[table_name = "payment_methods"]
pub struct NewPaymentMethod {
    user_id: Uuid,
    name: String,
    is_default: bool,
    provider: String,
    provider_data: serde_json::Value,
}

impl NewPaymentMethod {
    pub fn commit(self, conn: &PgConnection) -> Result<PaymentMethod, DatabaseError> {
        diesel::insert_into(payment_methods::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create payment method")
    }
}
