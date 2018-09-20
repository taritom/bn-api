use bigneon_db::models::TicketType;
use bigneon_db::utils::errors::DatabaseError;
use chrono::NaiveDateTime;
use diesel::PgConnection;
use models::DisplayTicketPricing;
use uuid::Uuid;

#[derive(Serialize)]
pub struct DisplayTicketType {
    pub id: Uuid,
    pub name: String,
    pub capacity: u32,
    pub status: String,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub ticket_pricing: Vec<DisplayTicketPricing>,
}

impl DisplayTicketType {
    pub fn from_ticket_type(
        ticket_type: &TicketType,
        conn: &PgConnection,
    ) -> Result<DisplayTicketType, DatabaseError> {
        let ticket_pricing: Vec<DisplayTicketPricing> = ticket_type
            .ticket_pricing(conn)?
            .iter()
            .map(|p| DisplayTicketPricing {
                id: p.id,
                name: p.name.clone(),
                status: p.status().to_string(),
                start_date: p.start_date,
                end_date: p.end_date,
                price_in_cents: p.price_in_cents,
            }).collect();
        let ticket_capacity = ticket_type.ticket_capacity(conn)?;

        Ok(DisplayTicketType {
            id: ticket_type.id,
            name: ticket_type.name.clone(),
            capacity: ticket_capacity,
            status: ticket_type.status().to_string(),
            start_date: ticket_type.start_date,
            end_date: ticket_type.end_date,
            ticket_pricing,
        })
    }
}
