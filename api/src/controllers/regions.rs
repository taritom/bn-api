use actix_web::{HttpResponse, Json, Path, Query};
use auth::user::User;
use bigneon_db::models::*;
use db::Connection;
use errors::*;
use helpers::application;
use models::PathParameters;

pub fn index(
    (connection, query_parameters): (Connection, Query<PagingParameters>),
) -> Result<HttpResponse, BigNeonError> {
    //TODO refactor query using paging parameters
    let regions = Region::all(connection.get())?;

    Ok(HttpResponse::Ok().json(&Payload::new(regions, query_parameters.into_inner().into())))
}

pub fn show(
    (connection, parameters): (Connection, Path<PathParameters>),
) -> Result<HttpResponse, BigNeonError> {
    let region = Region::find(&parameters.id, connection.get())?;
    Ok(HttpResponse::Ok().json(&region))
}

pub fn create(
    (connection, new_region, user): (Connection, Json<NewRegion>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    if !user.has_scope(Scopes::RegionWrite, None, connection)? {
        return application::unauthorized();
    }
    let region = new_region.into_inner().commit(connection)?;
    Ok(HttpResponse::Created().json(&region))
}

pub fn update(
    (connection, parameters, region_parameters, user): (
        Connection,
        Path<PathParameters>,
        Json<RegionEditableAttributes>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    if !user.has_scope(Scopes::RegionWrite, None, connection)? {
        return application::unauthorized();
    }
    let region = Region::find(&parameters.id, connection)?;
    let updated_region = region.update(region_parameters.into_inner(), connection)?;
    Ok(HttpResponse::Ok().json(updated_region))
}
