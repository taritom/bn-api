use chrono::NaiveDateTime;
use db::Connectable;
use diesel;
use diesel::prelude::*;
use models::User;
use schema::orders;
use serde::export::fmt::Display;
use serde_json;
use std::fmt;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum OrderStatus {
    Unpaid,
    Paid,
    Cancelled,
}

impl Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl OrderStatus {
    pub fn parse(s: &str) -> Result<OrderStatus, &'static str> {
        serde_json::from_str(s).map_err(|_| "Could not parse order status")
    }
}

#[derive(Associations, Identifiable, Queryable)]
#[belongs_to(User)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    status: String,
    created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "orders"]
pub struct NewOrder {
    user_id: Uuid,
    status: String,
}

impl NewOrder {
    pub fn commit(&self, conn: &Connectable) -> Result<Order, DatabaseError> {
        use schema::orders;
        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new order",
            diesel::insert_into(orders::table)
                .values(self)
                .get_result(conn.get_connection()),
        )
    }
}

impl Order {
    pub fn create(user_id: Uuid) -> NewOrder {
        NewOrder {
            user_id,
            status: OrderStatus::Unpaid.to_string(),
        }
    }
    pub fn status(&self) -> OrderStatus {
        return OrderStatus::parse(&self.status).unwrap();
    }
}
