use actix_web::{HttpResponse, Json, Path, Query};
use auth::user::User;
use bigneon_db::models::*;
use db::Connection;
use errors::*;
use helpers::application;
use models::PathParameters;

pub fn index(
    (connection, query_parameters, user): (Connection, Query<PagingParameters>, Option<User>),
) -> Result<HttpResponse, BigNeonError> {
    //TODO implement proper paging on db
    let artists = match user {
        Some(u) => Artist::all(Some(&u.user), connection.get())?,
        None => Artist::all(None, connection.get())?,
    };
    let payload = Payload::new(artists, query_parameters.into_inner().into());
    Ok(HttpResponse::Ok().json(&payload))
}

pub fn show(
    (connection, parameters): (Connection, Path<PathParameters>),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let artist = Artist::find(&parameters.id, connection)?;
    Ok(HttpResponse::Ok().json(&artist))
}

pub fn create(
    (connection, new_artist, user): (Connection, Json<NewArtist>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    if !user.has_scope(Scopes::ArtistWrite, None, connection)? {
        if new_artist.organization_id.is_none() {
            return application::unauthorized();
        } else if let Some(organization_id) = new_artist.organization_id {
            let organization = Organization::find(organization_id, connection)?;
            if !user.has_scope(Scopes::ArtistWrite, Some(&organization), connection)? {
                return application::unauthorized();
            }
        }
    }

    let mut artist = new_artist.commit(connection)?;

    // New artists belonging to an organization start private
    if artist.organization_id.is_some() {
        artist = artist.set_privacy(true, connection)?;
    }
    Ok(HttpResponse::Created().json(&artist))
}

pub fn show_from_organizations(
    (connection, organization_id, query_parameters, user): (
        Connection,
        Path<PathParameters>,
        Query<PagingParameters>,
        Option<User>,
    ),
) -> Result<HttpResponse, BigNeonError> {
    //TODO implement proper paging on db
    let artists = match user {
        Some(u) => {
            Artist::find_for_organization(Some(u.id()), organization_id.id, connection.get())?
        }
        None => Artist::find_for_organization(None, organization_id.id, connection.get())?,
    };
    let payload = Payload::new(artists, query_parameters.into_inner().into());
    Ok(HttpResponse::Ok().json(&payload))
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
    if !user.has_scope(Scopes::ArtistWrite, None, connection)? {
        if !artist.is_private || artist.organization_id.is_none() {
            return application::unauthorized();
        } else if let Some(organization) = artist.organization(connection)? {
            if !user.has_scope(Scopes::ArtistWrite, Some(&organization), connection)? {
                return application::unauthorized();
            }
        }
    }

    let updated_artist = artist.update(&artist_parameters, connection)?;
    Ok(HttpResponse::Ok().json(&updated_artist))
}

pub fn toggle_privacy(
    (connection, parameters, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    if !user.has_scope(Scopes::ArtistWrite, None, connection)? {
        return application::unauthorized();
    }
    let artist = Artist::find(&parameters.id, connection)?;
    let updated_artist = artist.set_privacy(!artist.is_private, connection)?;
    Ok(HttpResponse::Ok().json(updated_artist))
}
