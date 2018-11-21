use actix_web::{http::StatusCode, HttpResponse, Json, Path, Query, State};
use auth::user::User;
use bigneon_db::models::*;
use db::Connection;
use errors::*;
use helpers::application;
use models::{ExternalPathParameters, PathParameters, WebPayload};
use server::AppState;
use utils::spotify::*;

#[derive(Serialize, Debug)]
pub enum ArtistOrSpotify {
    Artist(Artist),
    Spotify(SpotifyArtist),
}

pub fn search(
    (state, connection, query_parameters, user): (
        State<AppState>,
        Connection,
        Query<PagingParameters>,
        Option<User>,
    ),
) -> Result<WebPayload<ArtistOrSpotify>, BigNeonError> {
    let db_user = user.map(|u| u.user);
    let artists = Artist::search(&db_user, query_parameters.get_tag("q"), connection.get())?;
    let try_spotify = query_parameters
        .get_tag("spotify")
        .map(|spotify| spotify != "0")
        .unwrap_or(false);
    if try_spotify && artists.is_empty() && query_parameters.get_tag("q").is_some() {
        //Try spotify
        let auth_token = state.config.spotify_auth_token.clone();
        let spotify_client = Spotify::connect(auth_token)?;
        let spotify_artists = spotify_client.search(
            query_parameters
                .get_tag("q")
                .unwrap_or("".to_string())
                .to_string(),
        )?;

        let wrapper = spotify_artists
            .iter()
            .map(|s| ArtistOrSpotify::Spotify(s.to_owned()))
            .collect();
        let payload = Payload::new(wrapper, query_parameters.into_inner().into());
        Ok(WebPayload::new(StatusCode::OK, payload))
    } else {
        let wrapper = artists
            .iter()
            .map(|a| ArtistOrSpotify::Artist(a.to_owned()))
            .collect();
        let payload = Payload::new(wrapper, query_parameters.into_inner().into());
        Ok(WebPayload::new(StatusCode::OK, payload))
    }
}

pub fn create_from_spotify(
    (state, connection, parameters, user): (
        State<AppState>,
        Connection,
        Path<ExternalPathParameters>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let auth_token = state.config.spotify_auth_token.clone();
    let spotify_client = Spotify::connect(auth_token)?;
    let spotify_artist_result = spotify_client.read_artist(&parameters.id);
    match spotify_artist_result {
        Ok(spotify_artist) => match spotify_artist {
            Some(artist) => {
                let new_artist = NewArtist {
                    organization_id: None,
                    name: artist["name"].as_str().unwrap_or(&"").to_string(),
                    bio: "".to_string(),
                    //TODO Add the image_url from the images[0] object
                    ..Default::default()
                };
                Ok(create((connection, Json(new_artist), user))?)
            }
            None => application::not_found(),
        },
        Err(e) => Err(e),
    }
}

pub fn index(
    (connection, query_parameters, user): (Connection, Query<PagingParameters>, Option<User>),
) -> Result<HttpResponse, BigNeonError> {
    let db_user = user.map(|u| u.user);
    let artists = Artist::search(&db_user, query_parameters.get_tag("q"), connection.get())?;
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
    if let Some(organization_id) = new_artist.organization_id {
        let organization = Organization::find(organization_id, connection)?;
        user.requires_scope_for_organization(Scopes::ArtistWrite, &organization, connection)?;
    } else {
        user.requires_scope(Scopes::ArtistWrite)?;
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
    if !artist.is_private || artist.organization_id.is_none() {
        user.requires_scope(Scopes::ArtistWrite)?;
    } else {
        let organization = artist.organization(connection)?.unwrap();
        user.requires_scope_for_organization(Scopes::ArtistWrite, &organization, connection)?;
    }

    let updated_artist = artist.update(&artist_parameters, connection)?;
    Ok(HttpResponse::Ok().json(&updated_artist))
}

pub fn toggle_privacy(
    (connection, parameters, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    user.requires_scope(Scopes::ArtistWrite)?;

    let connection = connection.get();
    let artist = Artist::find(&parameters.id, connection)?;
    let updated_artist = artist.set_privacy(!artist.is_private, connection)?;
    Ok(HttpResponse::Ok().json(updated_artist))
}
