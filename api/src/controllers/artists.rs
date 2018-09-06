use actix_web::{HttpResponse, Json, Path};
use auth::user::Scopes;
use auth::user::User;
use bigneon_db::models::{Artist, ArtistEditableAttributes, NewArtist};
use db::Connection;
use errors::*;
use helpers::application;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

pub fn index(connection: Connection) -> Result<HttpResponse, BigNeonError> {
    let artists = Artist::all(connection.get())?;
    Ok(HttpResponse::Ok().json(&artists))
}

pub fn show(
    (connection, parameters): (Connection, Path<PathParameters>),
) -> Result<HttpResponse, BigNeonError> {
    let artist = Artist::find(&parameters.id, connection.get())?;
    Ok(HttpResponse::Ok().json(&artist))
}

pub fn create(
    (connection, new_artist, user): (Connection, Json<NewArtist>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::ArtistWrite) {
        return application::unauthorized();
    }

    match new_artist.validate() {
        Ok(_) => {
            let artist = new_artist.commit(connection.get())?;
            Ok(HttpResponse::Created().json(&artist))
        }
        Err(e) => application::validation_error_response(e),
    }
}

pub fn update(
    (connection, parameters, artist_parameters, user): (
        Connection,
        Path<PathParameters>,
        Json<ArtistEditableAttributes>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::ArtistWrite) {
        return application::unauthorized();
    }
    let connection = connection.get();
    let artist = Artist::find(&parameters.id, connection)?;
    match artist_parameters.validate() {
        Ok(_) => {
            let updated_artist = artist.update(&artist_parameters, connection)?;
            Ok(HttpResponse::Ok().json(&updated_artist))
        }
        Err(e) => application::validation_error_response(e),
    }
}
