use actix_web::{HttpResponse, Json, Path, State};
use auth::user::Scopes;
use auth::user::User;
use bigneon_db::models::{NewRegion, Region, RegionEditableAttributes};
use errors::*;
use helpers::application;
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

pub fn index(state: State<AppState>) -> Result<HttpResponse, BigNeonError> {
    let connection = state.database.get_connection();
    let regions = Region::all(&*connection)?;
    Ok(HttpResponse::Ok().json(&regions))
}

pub fn show(
    (state, parameters): (State<AppState>, Path<PathParameters>),
) -> Result<HttpResponse, BigNeonError> {
    let connection = state.database.get_connection();
    let region = Region::find(&parameters.id, &*connection)?;
    Ok(HttpResponse::Ok().json(&region))
}

pub fn create(
    (state, new_region, user): (State<AppState>, Json<NewRegion>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::RegionWrite) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let region = new_region.into_inner().commit(&*connection)?;
    Ok(HttpResponse::Created().json(&region))
}

pub fn update(
    (state, parameters, region_parameters, user): (
        State<AppState>,
        Path<PathParameters>,
        Json<RegionEditableAttributes>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::RegionWrite) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();

    let region = Region::find(&parameters.id, &*connection)?;
    let updated_region = region.update(region_parameters.into_inner(), &*connection)?;
    Ok(HttpResponse::Ok().json(updated_region))
}
