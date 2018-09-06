use actix_web::{HttpResponse, Json, Path};
use auth::user::Scopes;
use auth::user::User;
use bigneon_db::models::{NewRegion, Region, RegionEditableAttributes};
use db::Connection;
use errors::*;
use helpers::application;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

pub fn index(connection: Connection) -> Result<HttpResponse, BigNeonError> {
    let regions = Region::all(connection.get())?;
    Ok(HttpResponse::Ok().json(&regions))
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
    if !user.has_scope(Scopes::RegionWrite) {
        return application::unauthorized();
    }
    let region = new_region.into_inner().commit(connection.get())?;
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
    if !user.has_scope(Scopes::RegionWrite) {
        return application::unauthorized();
    }
    let connection = connection.get();
    let region = Region::find(&parameters.id, connection)?;
    let updated_region = region.update(region_parameters.into_inner(), connection)?;
    Ok(HttpResponse::Ok().json(updated_region))
}
