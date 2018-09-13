use uuid::Uuid;

#[derive(Serialize)]
pub struct DisplayTicketPricing {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub price_in_cents: i64,
}
