use bigneon_db::models::TicketType;
use bigneon_db::utils::errors::DatabaseError;
use diesel::PgConnection;
use models::DisplayPricePoint;
use uuid::Uuid;

#[derive(Serialize)]
pub struct DisplayTicketType {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub price_points: Vec<DisplayPricePoint>,
}

impl DisplayTicketType {
    pub fn from_ticket_type(
        ticket_type: &TicketType,
        conn: &PgConnection,
    ) -> Result<DisplayTicketType, DatabaseError> {
        let price_points: Vec<DisplayPricePoint> = ticket_type
            .price_points(conn)?
            .iter()
            .map(|p| DisplayPricePoint {
                id: p.id,
                name: p.name.clone(),
                status: p.status().to_string(),
                price_in_cents: p.price_in_cents,
            })
            .collect();

        Ok(DisplayTicketType {
            id: ticket_type.id,
            name: ticket_type.name.clone(),
            status: ticket_type.status().to_string(),
            price_points,
        })
    }
}
