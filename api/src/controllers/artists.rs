use actix_web::{HttpResponse, Json, Path};
use auth::user::Scopes;
use auth::user::User;
use bigneon_db::models::{Artist, ArtistEditableAttributes, NewArtist, Organization};
use db::Connection;
use errors::*;
use helpers::application;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

pub fn index((connection, user): (Connection, Option<User>)) -> Result<HttpResponse, BigNeonError> {
    let artists = match user {
        Some(u) => Artist::all(Some(u.id()), connection.get())?,
        None => Artist::all(None, connection.get())?,
    };
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

pub fn show_from_organizations(
    (connection, organization_id, user): (Connection, Path<PathParameters>, Option<User>),
) -> Result<HttpResponse, BigNeonError> {
    let artists = match user {
        Some(u) => {
            Artist::find_for_organization(Some(u.id()), organization_id.id, connection.get())?
        }
        None => Artist::find_for_organization(None, organization_id.id, connection.get())?,
    };
    Ok(HttpResponse::Ok().json(&artists))
}

pub fn update(
    (connection, parameters, artist_parameters, user): (
        Connection,
        Path<PathParameters>,
        Json<ArtistEditableAttributes>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let artist = Artist::find(&parameters.id, connection)?;

    if !(user.has_scope(Scopes::ArtistWrite)
        || (user.has_scope(Scopes::OrgWrite) && artist.organization_id.is_some()
            && Organization::find(artist.organization_id.unwrap(), connection)?
                .is_member(&user.user, connection)?))
    {
        return application::unauthorized();
    }

    match artist_parameters.validate() {
        Ok(_) => {
            let updated_artist = artist.update(&artist_parameters, connection)?;
            Ok(HttpResponse::Ok().json(&updated_artist))
        }
        Err(e) => application::validation_error_response(e),
    }
}
