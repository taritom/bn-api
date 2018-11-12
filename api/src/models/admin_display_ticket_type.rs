use bigneon_db::models::{FeeSchedule, TicketType};
use bigneon_db::utils::errors::DatabaseError;
use chrono::NaiveDateTime;
use diesel::PgConnection;
use models::DisplayTicketPricing;
use uuid::Uuid;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct AdminDisplayTicketType {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub capacity: u32,
    pub status: String,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub quantity: u32,
    pub increment: i32,
    pub ticket_pricing: Vec<DisplayTicketPricing>,
}

impl AdminDisplayTicketType {
    pub fn from_ticket_type(
        ticket_type: &TicketType,
        fee_schedule: &FeeSchedule,
        conn: &PgConnection,
    ) -> Result<AdminDisplayTicketType, DatabaseError> {
        let quantity = ticket_type.remaining_ticket_count(conn)?;
        let capacity = ticket_type.ticket_capacity(conn)?;
        let mut ticket_pricing_list = Vec::new();
        for ticket_pricing in ticket_type.valid_ticket_pricing(conn)? {
            ticket_pricing_list.push(DisplayTicketPricing::from_ticket_pricing(
                &ticket_pricing,
                fee_schedule,
                conn,
            )?);
        }

        Ok(AdminDisplayTicketType {
            id: ticket_type.id,
            name: ticket_type.name.clone(),
            description: ticket_type.description.clone(),
            status: ticket_type.status().to_string(),
            start_date: ticket_type.start_date,
            end_date: ticket_type.end_date,
            ticket_pricing: ticket_pricing_list,
            quantity,
            capacity,
            increment: ticket_type.increment,
        })
    }
}
