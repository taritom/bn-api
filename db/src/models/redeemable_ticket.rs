use chrono::prelude::*;
use diesel::sql_types::{Nullable, Text, Timestamp, Uuid as dUuid};
use uuid::Uuid;

#[derive(Queryable, QueryableByName, Serialize, Deserialize)]
pub struct RedeemableTicket {
    #[sql_type = "dUuid"]
    pub id: Uuid,
    #[sql_type = "Text"]
    pub ticket_type: String,
    #[sql_type = "dUuid"]
    pub user_id: Uuid,
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
    pub status: String,
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
    #[sql_type = "Text"]
    pub venue_name: String,
}
