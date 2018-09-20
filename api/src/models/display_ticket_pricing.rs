use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Serialize)]
pub struct DisplayTicketPricing {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub price_in_cents: i64,
}
