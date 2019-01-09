use actix_web::{HttpResponse, Path};
use auth::user::User as AuthUser;
use bigneon_db::models::*;
use db::Connection;
use errors::*;
use extractors::*;
use models::PathParameters;
//
//pub fn index(
//    (connection, query_parameters, user): (Connection, Query<PathParameters>, OptionalUser),
//) -> Result<HttpResponse, BigNeonError> {
//    let venues = match user.into_inner() {
//        Some(u) => Venue::all(Some(&u.user), connection.get())?,
//        None => Venue::all(None, connection.get())?,
//    };
//
//    Ok(HttpResponse::Ok().json(&Payload::from_data(
//        venues,
//        query_parameters.page(),
//        query_parameters.limit(),
//    )))
//}
//
//pub fn show(
//    (connection, parameters): (Connection, Path<PathParameters>),
//) -> Result<HttpResponse, BigNeonError> {
//
//    let connection = connection.get();
//    let venue = Venue::find(parameters.id, connection)?;
//
//    Ok(HttpResponse::Ok().json(&venue))
//}
//
//
#[derive(Deserialize)]
pub struct CreateStage {
    pub name: String,
    pub description: Option<String>,
    pub capacity: Option<i64>,
}

pub fn create(
    (connection, parameters, create_stage, user): (
        Connection,
        Path<PathParameters>,
        Json<CreateStage>,
        AuthUser,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let venue = Venue::find(parameters.id, connection)?;

    if let Some(organization_id) = venue.organization_id {
        let organization = Organization::find(organization_id, connection)?;
        user.requires_scope_for_organization(Scopes::VenueWrite, &organization, connection)?;
    } else {
        println!("Checking user");
        user.requires_scope(Scopes::VenueWrite)?;
    }

    let new_stage = Stage::create(
        parameters.id,
        create_stage.name.clone(),
        create_stage.description.clone(),
        create_stage.capacity.clone(),
    );
    let stage = new_stage.commit(connection)?;

    Ok(HttpResponse::Created().json(&stage))
}
//
//
//pub fn update(
//    (connection, parameters, venue_parameters, user): (
//        Connection,
//        Path<PathParameters>,
//        Json<VenueEditableAttributes>,
//        User,
//    ),
//) -> Result<HttpResponse, BigNeonError> {
//    let connection = connection.get();
//    let venue = Venue::find(parameters.id, connection)?;
//    if !venue.is_private || venue.organization_id.is_none() {
//        user.requires_scope(Scopes::VenueWrite)?;
//    } else {
//        let organization = venue.organization(connection)?.unwrap();
//        user.requires_scope_for_organization(Scopes::VenueWrite, &organization, connection)?;
//    }
//
//    let updated_venue = venue.update(venue_parameters.into_inner(), connection)?;
//    Ok(HttpResponse::Ok().json(updated_venue))
//}
//
//pub fn delete(
//    (connection, parameters, venue_parameters, user): (
//        Connection,
//        Path<PathParameters>,
//        Json<VenueEditableAttributes>,
//        User,
//    ),
//) -> Result<HttpResponse, BigNeonError> {
//    let connection = connection.get();
//    let venue = Venue::find(parameters.id, connection)?;
//    if !venue.is_private || venue.organization_id.is_none() {
//        user.requires_scope(Scopes::VenueWrite)?;
//    } else {
//        let organization = venue.organization(connection)?.unwrap();
//        user.requires_scope_for_organization(Scopes::VenueWrite, &organization, connection)?;
//    }
//
//    let updated_venue = venue.update(venue_parameters.into_inner(), connection)?;
//    Ok(HttpResponse::Ok().json(updated_venue))
//}
//
