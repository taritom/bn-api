use chrono::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum HistoryItem {
    Purchase {
        revenue_in_cents: u32,
        event_name: String,
        ticket_sales: u32,
        order_id: Uuid,
        order_date: NaiveDateTime,
    },
}
