use bigneon_db::models::{TicketType, TicketTypeStatus};
use bigneon_db::utils::errors::*;
use chrono::NaiveDateTime;
use diesel::PgConnection;
use models::DisplayTicketPricing;
use uuid::Uuid;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct UserDisplayTicketType {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub quantity: u32,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub ticket_pricing: Option<DisplayTicketPricing>,
}

impl UserDisplayTicketType {
    pub fn from_ticket_type(
        ticket_type: &TicketType,
        conn: &PgConnection,
    ) -> Result<UserDisplayTicketType, DatabaseError> {
        let ticket_type_status = ticket_type.status();
        let mut status = ticket_type_status.to_string();
        let quantity = ticket_type.remaining_ticket_count(conn)?;

        let ticket_pricing: Option<DisplayTicketPricing> = ticket_type
            .current_ticket_pricing(conn)
            .optional()?
            .and_then(|ticket_pricing| Some(ticket_pricing.into()));

        if ticket_type_status == TicketTypeStatus::Published {
            if quantity == 0 {
                status = TicketTypeStatus::SoldOut.to_string();
            } else if ticket_pricing.is_none() {
                status = TicketTypeStatus::NoActivePricing.to_string();
            }
        }

        Ok(UserDisplayTicketType {
            id: ticket_type.id,
            name: ticket_type.name.clone(),
            status,
            start_date: ticket_type.start_date,
            end_date: ticket_type.end_date,
            ticket_pricing,
            quantity,
        })
    }
}
