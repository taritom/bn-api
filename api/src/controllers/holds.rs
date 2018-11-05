use actix_web::{HttpResponse, Json, Path};
use auth::user::User;
use bigneon_db::models::*;
use chrono::prelude::*;
use db::Connection;
use errors::BigNeonError;
use helpers::application;
use models::PathParameters;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct CreateHoldRequest {
    pub name: String,
    pub redemption_code: String,
    pub discount_in_cents: Option<u32>,
    pub hold_type: HoldTypes,
    pub quantity: u32,
    pub ticket_type_id: Uuid,
    pub end_at: Option<NaiveDateTime>,
    pub max_per_order: Option<u32>,
}

#[derive(Default, Deserialize, Serialize)]
pub struct UpdateHoldRequest {
    pub name: Option<String>,
    pub redemption_code: Option<String>,
    pub hold_type: Option<HoldTypes>,
    pub quantity: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_some")]
    pub discount_in_cents: Option<Option<i64>>,
    pub end_at: Option<Option<NaiveDateTime>>,
    pub max_per_order: Option<Option<i64>>,
}

impl From<UpdateHoldRequest> for UpdateHoldAttributes {
    fn from(attributes: UpdateHoldRequest) -> Self {
        UpdateHoldAttributes {
            name: attributes.name,
            hold_type: attributes
                .hold_type
                .and_then(|hold_type| Some(hold_type.to_string())),
            discount_in_cents: attributes.discount_in_cents,
            end_at: attributes.end_at,
            max_per_order: attributes.max_per_order,
            redemption_code: attributes.redemption_code,
        }
    }
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
        req.hold_type,
        req.ticket_type_id,
    ).commit(conn)?;

    hold.set_quantity(req.quantity, conn)?;

    application::created(json!(hold))
}

pub fn update(
    (conn, req, path, user): (
        Connection,
        Json<UpdateHoldRequest>,
        Path<PathParameters>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let conn = conn.get();

    let hold = Hold::find(path.id, conn)?;
    user.requires_scope_for_organization(Scopes::HoldWrite, &hold.organization(conn)?, conn)?;
    Ok(HttpResponse::Ok().json(hold.update(req.into_inner().into(), conn)?))
}

pub fn show(
    (conn, path, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let conn = conn.get();
    let hold = Hold::find(path.id, conn)?;
    user.requires_scope_for_organization(Scopes::OrgRead, &hold.organization(conn)?, conn)?;

    Ok(HttpResponse::Ok().json(hold))
}

#[derive(Deserialize)]
pub struct SetQuantityRequest {
    pub quantity: u32,
}

pub fn add_remove_from_hold(
    (conn, req, path, user): (
        Connection,
        Json<SetQuantityRequest>,
        Path<PathParameters>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let conn = conn.get();
    let hold = Hold::find(path.id, conn)?;
    user.requires_scope_for_organization(Scopes::HoldWrite, &hold.organization(conn)?, conn)?;
    hold.set_quantity(req.quantity, conn)?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize, Serialize)]
pub struct SplitHoldRequest {
    pub name: String,
    pub redemption_code: String,
    pub discount_in_cents: Option<u32>,

    pub hold_type: HoldTypes,
    pub quantity: u32,
    pub end_at: Option<NaiveDateTime>,
    pub max_per_order: Option<u32>,
}

pub fn split(
    (conn, req, path, user): (
        Connection,
        Json<SplitHoldRequest>,
        Path<PathParameters>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let conn = conn.get();
    let hold = Hold::find(path.id, conn)?;
    user.requires_scope_for_organization(Scopes::HoldWrite, &hold.organization(conn)?, conn)?;
    let new_hold = hold.split(
        req.name.clone(),
        req.redemption_code.clone(),
        req.quantity,
        req.discount_in_cents,
        req.hold_type,
        req.end_at,
        req.max_per_order,
        conn,
    )?;
    Ok(HttpResponse::Created().json(new_hold))
}

pub fn destroy(
    (conn, path, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let conn = conn.get();
    let hold = Hold::find(path.id, conn)?;
    user.requires_scope_for_organization(Scopes::HoldWrite, &hold.organization(conn)?, conn)?;
    hold.destroy(conn)?;
    Ok(HttpResponse::Ok().finish())
}
