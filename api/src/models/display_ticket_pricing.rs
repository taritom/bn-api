use bigneon_db::models::{FeeSchedule, TicketPricing};
use bigneon_db::utils::errors::DatabaseError;
use chrono::NaiveDateTime;
use diesel::PgConnection;
use uuid::Uuid;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct DisplayTicketPricing {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub price_in_cents: i64,
    pub fee_in_cents: i64,
}

impl DisplayTicketPricing {
    pub fn from_ticket_pricing(
        ticket_pricing: &TicketPricing,
        fee_schedule: &FeeSchedule,
        conn: &PgConnection,
    ) -> Result<DisplayTicketPricing, DatabaseError> {
        let fee_in_cents = fee_schedule
            .get_range(ticket_pricing.price_in_cents, conn)?
            .fee_in_cents;

        Ok(DisplayTicketPricing {
            id: ticket_pricing.id,
            name: ticket_pricing.name.clone(),
            status: ticket_pricing.status()?.to_string(),
            start_date: ticket_pricing.start_date,
            end_date: ticket_pricing.end_date,
            price_in_cents: ticket_pricing.price_in_cents,
            fee_in_cents,
        })
    }
}
