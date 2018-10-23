use actix_web::{HttpResponse, Json, Path};
use auth::user::User;
use bigneon_db::models::*;
use chrono::prelude::*;
use db::Connection;
use errors::BigNeonError;
use helpers::application;
use models::PathParameters;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct CreateHoldRequest {
    pub name: String,
    pub redemption_code: String,
    pub discount_in_cents: u32,
    pub end_at: Option<NaiveDateTime>,
    pub max_per_order: Option<u32>,
    pub items: Vec<HoldItem>,
}

#[derive(Serialize, Deserialize)]
pub struct HoldItem {
    pub ticket_type_id: Uuid,
    pub quantity: u32,
}

// add update fields in here as well

pub fn create(
    (conn, req, path, user): (
        Connection,
        Json<CreateHoldRequest>,
        Path<PathParameters>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let conn = conn.get();
    let event = Event::find(path.id, conn)?;
    user.requires_scope_for_organization(Scopes::HoldWrite, &event.organization(conn)?, conn)?;

    let hold = Hold::create(
        req.name.clone(),
        path.id,
        req.redemption_code.clone(),
        req.discount_in_cents,
        req.end_at,
        req.max_per_order,
    ).commit(conn)?;

    for line in &req.items {
        hold.set_quantity(line.ticket_type_id, line.quantity, conn)?;
    }

    application::created(json!(hold))
}

pub fn update(
    (conn, req, path, user): (
        Connection,
        Json<UpdateHoldAttributes>,
        Path<PathParameters>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let conn = conn.get();

    let hold = Hold::find(path.id, conn)?;
    user.requires_scope_for_organization(Scopes::HoldWrite, &hold.organization(conn)?, conn)?;
    Ok(HttpResponse::Ok().json(hold.update(req.into_inner(), conn)))
}

#[derive(Deserialize)]
pub struct UpdateHoldItemsRequest {
    pub items: Vec<HoldItem>,
}

pub fn add_remove_from_hold(
    (conn, req, path, user): (
        Connection,
        Json<UpdateHoldItemsRequest>,
        Path<PathParameters>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let conn = conn.get();
    let hold = Hold::find(path.id, conn)?;
    user.requires_scope_for_organization(Scopes::HoldWrite, &hold.organization(conn)?, conn)?;
    for line in &req.items {
        hold.set_quantity(line.ticket_type_id, line.quantity, conn)?;
    }

    Ok(HttpResponse::Ok().finish())
}
