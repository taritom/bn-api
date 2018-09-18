use actix_web::HttpResponse;
use actix_web::Json;
use auth::user::User;
use bigneon_db::models::TicketType;
use chrono::NaiveDateTime;
use db::Connection;
use errors::BigNeonError;
use helpers::application;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateTicketPricingRequest {
    pub ticket_type_id: Uuid,
    pub name: String,
    pub price: i64,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
}

pub fn create(
    (connection, json, user): (Connection, Json<CreateTicketPricingRequest>, User),
) -> Result<HttpResponse, BigNeonError> {
    let db = connection.get();
    let ticket_type = TicketType::find(json.ticket_type_id, db)?;
    let pricing = ticket_type.add_ticket_pricing(
        json.name.clone(),
        json.start_date,
        json.end_date,
        json.price,
        db,
    )?;
    application::created(json!({
        "ticket_pricing_id": pricing.id
    }))
}
