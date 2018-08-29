use uuid::Uuid;

#[derive(Serialize)]
pub struct DisplayPricePoint {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub price_in_cents: i64,
}
