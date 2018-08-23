use actix_web::{HttpResponse, Json, Path, State};
use auth::user::Scopes;
use auth::user::User;
use bigneon_db::models::{Artist, ArtistEditableAttributes, NewArtist};
use errors::*;
use helpers::application;
use server::AppState;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

pub fn index(state: State<AppState>) -> Result<HttpResponse, BigNeonError> {
    let connection = state.database.get_connection();
    let artists = Artist::all(&*connection)?;
    Ok(HttpResponse::Ok().json(&artists))
}

pub fn show(
    (state, parameters): (State<AppState>, Path<PathParameters>),
) -> Result<HttpResponse, BigNeonError> {
    let connection = state.database.get_connection();
    let artist = Artist::find(&parameters.id, &*connection)?;
    Ok(HttpResponse::Ok().json(&artist))
}

pub fn create(
    (state, new_artist, user): (State<AppState>, Json<NewArtist>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::ArtistWrite) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();

    match new_artist.validate() {
        Ok(_) => {
            let artist = new_artist.commit(&*connection)?;
            Ok(HttpResponse::Created().json(&artist))
        }
        Err(e) => application::validation_error_response(e),
    }
}

pub fn update(
    (state, parameters, artist_parameters, user): (
        State<AppState>,
        Path<PathParameters>,
        Json<ArtistEditableAttributes>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::ArtistWrite) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let artist = Artist::find(&parameters.id, &*connection)?;
    match artist_parameters.validate() {
        Ok(_) => {
            let updated_artist = artist.update(&artist_parameters, &*connection)?;
            Ok(HttpResponse::Ok().json(&updated_artist))
        }
        Err(e) => application::validation_error_response(e),
    }
}
