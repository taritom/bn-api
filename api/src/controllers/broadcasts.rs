use actix_web::Path;
use actix_web::{HttpResponse, Query};
use auth::user::User;
use bigneon_db::models::enums::{BroadcastAudience, BroadcastChannel, BroadcastType};
use bigneon_db::models::scopes::Scopes;
use bigneon_db::models::{Broadcast, BroadcastEditableAttributes, Organization, PagingParameters};
use chrono::NaiveDateTime;
use db::Connection;
use errors::BigNeonError;
use extractors::Json;
use models::{PathParameters, WebPayload};
use reqwest::StatusCode;

#[derive(Deserialize, Serialize)]
pub struct NewBroadcastData {
    // TODO: Should this change to subject?
    pub notification_type: BroadcastType,
    //None is now
    pub send_at: Option<NaiveDateTime>,
    pub message: Option<String>,
    pub channel: Option<BroadcastChannel>,
    pub audience: BroadcastAudience,
    pub subject: Option<String>,
}

pub fn create(
    (conn, path, json, user): (
        Connection,
        Path<PathParameters>,
        Json<NewBroadcastData>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = conn.get();
    let organization = Organization::find_for_event(path.id, connection)?;

    let channel = json.channel.unwrap_or(BroadcastChannel::PushNotification);
;

    user.requires_scope_for_organization(Scopes::EventBroadcast, &organization, connection)?;

    let broadcast = Broadcast::create(
        path.id,
        json.notification_type,
        channel,
        json.message.clone(),
        json.send_at,
        None,
        None,
        BroadcastAudience::PeopleAtTheEvent,
    )
    .commit(connection)?;
    Ok(HttpResponse::Created().json(json!(broadcast)))
}

pub fn index(
    (conn, path, query, user): (
        Connection,
        Path<PathParameters>,
        Query<PagingParameters>,
        User,
    ),
) -> Result<WebPayload<Broadcast>, BigNeonError> {
    let connection = conn.get();
    let organization = Organization::find_for_event(path.id, connection)?;

    user.requires_scope_for_organization(Scopes::EventBroadcast, &organization, connection)?;

    let push_notifications =
        Broadcast::find_by_event_id(path.id, query.page(), query.limit(), connection)?;

    Ok(WebPayload::new(StatusCode::OK, push_notifications))
}

pub fn show(
    (conn, path, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = conn.get();
    let push_notification = Broadcast::find(path.id, connection)?;
    let organization = Organization::find_for_event(push_notification.event_id, connection)?;

    user.requires_scope_for_organization(Scopes::EventBroadcast, &organization, connection)?;

    Ok(HttpResponse::Ok().json(push_notification))
}

pub fn update(
    (conn, path, json, user): (
        Connection,
        Path<PathParameters>,
        Json<BroadcastEditableAttributes>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = conn.get();
    let broadcast = Broadcast::find(path.id, connection)?;
    let organization = Organization::find_for_event(broadcast.event_id, connection)?;

    user.requires_scope_for_organization(Scopes::EventBroadcast, &organization, connection)?;
    let broadcast_attributes = json.into_inner();
    let broadcast = broadcast.update(broadcast_attributes, connection)?;
    Ok(HttpResponse::Ok().json(broadcast))
}

pub fn delete(
    (conn, path, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = conn.get();
    let broadcast = Broadcast::find(path.id, connection)?;
    let organization = Organization::find_for_event(broadcast.event_id, connection)?;

    user.requires_scope_for_organization(Scopes::EventBroadcast, &organization, connection)?;

    let broadcast = broadcast.cancel(connection)?;
    Ok(HttpResponse::Ok().json(broadcast))
}

pub fn tracking_count(
    (conn, path, _user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = conn.get();
    Broadcast::increment_open_count(path.id, connection)?;
    Ok(HttpResponse::Ok().finish())
}
