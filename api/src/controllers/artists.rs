use actix_web::{HttpResponse, Json, Path, State};
use auth::user::Scopes;
use auth::user::User;
use bigneon_db::models::{Artist, ArtistEditableAttributes, NewArtist};
use errors::database_error::ConvertToWebError;
use helpers::application;
use server::AppState;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

pub fn index(state: State<AppState>) -> HttpResponse {
    let connection = state.database.get_connection();
    let artists_response = Artist::all(&*connection);
    match artists_response {
        Ok(artists) => HttpResponse::Ok().json(&artists),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn show(data: (State<AppState>, Path<PathParameters>)) -> HttpResponse {
    let (state, parameters) = data;
    let connection = state.database.get_connection();
    let artist_response = Artist::find(&parameters.id, &*connection);

    match artist_response {
        Ok(artist) => HttpResponse::Ok().json(&artist),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn create((state, new_artist, user): (State<AppState>, Json<NewArtist>, User)) -> HttpResponse {
    if !user.has_scope(Scopes::ArtistWrite) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();

    match new_artist.validate() {
        Ok(_) => {
            let artist_response = new_artist.commit(&*connection);

            match artist_response {
                Ok(artist) => HttpResponse::Created().json(&artist),
                Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
            }
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
) -> HttpResponse {
    if !user.has_scope(Scopes::ArtistWrite) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();

    let artist_response = Artist::find(&parameters.id, &*connection);

    match artist_response {
        Ok(artist) => match artist_parameters.validate() {
            Ok(_) => {
                let artist_update_response = artist.update(&artist_parameters, &*connection);

                match artist_update_response {
                    Ok(updated_artist) => HttpResponse::Ok().json(&updated_artist),
                    Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
                }
            }
            Err(e) => application::validation_error_response(e),
        },
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}
