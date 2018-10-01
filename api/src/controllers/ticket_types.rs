use actix_web::{HttpResponse, Json, Path, State};
use auth::user::User;
use bigneon_db::models::*;
use chrono::prelude::*;
use db::Connection;
use errors::*;
use helpers::application;
use models::{AdminDisplayTicketType, EventTicketPathParameters, PathParameters};
use server::AppState;
use tari_client::tari_messages::NewAsset as TariNewAsset;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateTicketPricingRequest {
    pub name: String,
    pub price_in_cents: i64,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
}

#[derive(Deserialize)]
pub struct CreateTicketTypeRequest {
    pub name: String,
    pub capacity: u32,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub ticket_pricing: Vec<CreateTicketPricingRequest>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateTicketPricingRequest {
    pub id: Option<Uuid>,
    pub name: Option<String>,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
    pub price_in_cents: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateTicketTypeRequest {
    //pub id: Uuid,
    pub name: Option<String>,
    pub capacity: Option<u32>,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
    pub ticket_pricing: Option<Vec<UpdateTicketPricingRequest>>,
}

#[derive(Serialize, Deserialize)]
pub struct DisplayCreatedTicket {
    pub id: Uuid,
}

pub fn create(
    (connection, path, data, user, state): (
        Connection,
        Path<PathParameters>,
        Json<CreateTicketTypeRequest>,
        User,
        State<AppState>,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let event = Event::find(path.id, connection)?;
    if !user.has_scope(
        Scopes::EventWrite,
        Some(&event.organization(connection)?),
        connection,
    )? {
        return application::unauthorized();
    }
    //Add new ticket type
    let ticket_type = event.add_ticket_type(
        data.name.clone(),
        data.capacity,
        data.start_date,
        data.end_date,
        connection,
    )?;
    //Add each ticket pricing entry for newly created ticket type
    for curr_pricing_entry in &data.ticket_pricing {
        let _pricing_result = ticket_type.add_ticket_pricing(
            curr_pricing_entry.name.clone(),
            curr_pricing_entry.start_date,
            curr_pricing_entry.end_date,
            curr_pricing_entry.price_in_cents,
            connection,
        )?;
    }
    // TODO: move this to an async processor...
    let tari_asset_id = state.config.tari_client.create_asset(TariNewAsset {
        name: format!("{}.{}", event.name, data.name),
        symbol: "sym".into(), //TODO remove symbol from asset spec,
        decimals: 0,
        total_supply: data.capacity as i64,
        authorised_signers: vec![user.id().hyphenated().to_string()],
        issuer: user.id().hyphenated().to_string(),
        valid: true,
        rule_flags: 0,
        rule_metadata: "".to_string(),
        expiry_date: data.end_date.timestamp(),
    })?;
    let asset = Asset::find_by_ticket_type(&ticket_type.id, connection)?;
    let _asset = asset.update_blockchain_id(tari_asset_id, connection)?;
    Ok(HttpResponse::Created().json(DisplayCreatedTicket { id: ticket_type.id }))
}

#[derive(Deserialize, Serialize)]
pub struct TicketTypesResponse {
    pub ticket_types: Vec<AdminDisplayTicketType>,
}

pub fn index(
    (connection, path, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let event = Event::find(path.id, connection)?;
    if !user.has_scope(
        Scopes::EventWrite,
        Some(&event.organization(connection)?),
        connection,
    )? {
        return application::unauthorized();
    }

    let ticket_types = TicketType::find_by_event_id(path.id, connection)?;
    let mut encoded_ticket_types = Vec::new();
    for t in ticket_types {
        encoded_ticket_types.push(AdminDisplayTicketType::from_ticket_type(&t, connection)?);
    }

    Ok(HttpResponse::Ok().json(TicketTypesResponse {
        ticket_types: encoded_ticket_types,
    }))
}

pub fn update(
    (connection, path, data, user): (
        Connection,
        Path<EventTicketPathParameters>,
        Json<UpdateTicketTypeRequest>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let event = Event::find(path.event_id, connection)?;
    if !user.has_scope(
        Scopes::EventWrite,
        Some(&event.organization(connection)?),
        connection,
    )? {
        return application::unauthorized();
    }

    //Update the editable attributes of the ticket type
    let update_parameters = TicketTypeEditableAttributes {
        name: data.name.clone(),
        start_date: data.start_date.clone(),
        end_date: data.end_date.clone(),
    };
    let ticket_type = TicketType::find(path.ticket_type_id, connection)?;
    let updated_ticket_type = ticket_type.update(update_parameters, connection)?;

    if data.ticket_pricing.is_some() {
        let data_ticket_pricings = data.into_inner().ticket_pricing.unwrap();
        //Retrieve the current list of pricings associated with this ticket_type and remove unwanted pricings
        let ticket_pricings = updated_ticket_type.ticket_pricing(connection)?;
        for curr_ticket_pricing in &ticket_pricings {
            let mut found_flag = false;
            for request_ticket_pricing in &data_ticket_pricings {
                if request_ticket_pricing.id.is_some() {
                    if curr_ticket_pricing.id == request_ticket_pricing.id.unwrap() {
                        found_flag = true;
                        break;
                    }
                }
            }
            if !found_flag {
                curr_ticket_pricing.destroy(connection)?;
            }
        }

        //Update the editable attributes for remaining ticket pricings
        for curr_ticket_pricing in &data_ticket_pricings {
            if curr_ticket_pricing.id.is_some() {
                //Update the ticket pricing
                let update_parameters = TicketPricingEditableAttributes {
                    name: curr_ticket_pricing.name.clone(),
                    price_in_cents: curr_ticket_pricing.price_in_cents,
                    start_date: curr_ticket_pricing.start_date,
                    end_date: curr_ticket_pricing.end_date,
                };
                let curr_ticket_pricing_id = curr_ticket_pricing.id.unwrap();
                let found_index = ticket_pricings
                    .iter()
                    .position(|ref r| r.id == curr_ticket_pricing_id);
                if found_index.is_some() {
                    ticket_pricings[found_index.unwrap()].update(update_parameters, connection)?;
                }
            } else {
                //Only create a new pricing entry if all of its required data was provided
                if curr_ticket_pricing.name.is_some()
                    && curr_ticket_pricing.price_in_cents.is_some()
                    && curr_ticket_pricing.start_date.is_some()
                    && curr_ticket_pricing.end_date.is_some()
                {
                    let curr_ticket_pricing_name = curr_ticket_pricing.name.clone().unwrap();

                    //Add new ticket pricing
                    let _pricing_result = updated_ticket_type.add_ticket_pricing(
                        curr_ticket_pricing_name,
                        curr_ticket_pricing.start_date.unwrap(),
                        curr_ticket_pricing.end_date.unwrap(),
                        curr_ticket_pricing.price_in_cents.unwrap(),
                        connection,
                    )?;
                } else {
                    //TODO send error when all data was not specified

                }
            }
        }
    }

    Ok(HttpResponse::Ok().finish())
}
