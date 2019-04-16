use chrono::prelude::*;
use diesel::sql_types::{BigInt, Nullable, Text, Timestamp, Uuid as dUuid};
use models::*;
use uuid::Uuid;

#[derive(Clone, Queryable, QueryableByName, PartialEq, Serialize, Deserialize, Debug)]
pub struct RedeemableTicket {
    #[sql_type = "dUuid"]
    pub id: Uuid,
    #[sql_type = "Text"]
    pub ticket_type: String,
    #[sql_type = "Nullable<dUuid>"]
    pub user_id: Option<Uuid>,
    #[sql_type = "dUuid"]
    pub order_id: Uuid,
    #[sql_type = "dUuid"]
    pub order_item_id: Uuid,
    #[sql_type = "BigInt"]
    pub price_in_cents: i64,
    #[sql_type = "Nullable<Text>"]
    pub first_name: Option<String>,
    #[sql_type = "Nullable<Text>"]
    pub last_name: Option<String>,
    #[sql_type = "Nullable<Text>"]
    pub email: Option<String>,
    #[sql_type = "Nullable<Text>"]
    pub phone: Option<String>,
    #[sql_type = "Nullable<Text>"]
    pub redeem_key: Option<String>,
    #[sql_type = "Nullable<Timestamp>"]
    pub redeem_date: Option<NaiveDateTime>,
    #[sql_type = "Text"]
    pub status: TicketInstanceStatus,
    #[sql_type = "dUuid"]
    pub event_id: Uuid,
    #[sql_type = "Text"]
    pub event_name: String,
    #[sql_type = "Nullable<Timestamp>"]
    pub door_time: Option<NaiveDateTime>,
    #[sql_type = "Nullable<Timestamp>"]
    pub event_start: Option<NaiveDateTime>,
    #[sql_type = "Nullable<dUuid>"]
    pub venue_id: Option<Uuid>,
    #[sql_type = "Nullable<Text>"]
    pub venue_name: Option<String>,
    #[sql_type = "Timestamp"]
    pub updated_at: NaiveDateTime,
}
