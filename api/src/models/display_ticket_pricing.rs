use bigneon_db::models::TicketPricing;
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct DisplayTicketPricing {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub price_in_cents: i64,
}

impl From<TicketPricing> for DisplayTicketPricing {
    fn from(ticket_pricing: TicketPricing) -> Self {
        DisplayTicketPricing {
            id: ticket_pricing.id,
            name: ticket_pricing.name.clone(),
            status: ticket_pricing.status().to_string(),
            start_date: ticket_pricing.start_date,
            end_date: ticket_pricing.end_date,
            price_in_cents: ticket_pricing.price_in_cents,
        }
    }
}
