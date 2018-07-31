use actix_web::{HttpResponse, Json, Path, State};
use bigneon_db::models::artists::{NewArtist, UserEditableAttributes};
use bigneon_db::models::Artist;
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    id: Uuid,
}

pub fn index(state: State<AppState>) -> HttpResponse {
    let connection = state.database.get_connection();
    let artists_response = Artist::all(&*connection);
    match artists_response {
        Ok(artists) => HttpResponse::Ok().json(&artists),
        Err(_e) => {
            HttpResponse::InternalServerError().json(json!({"error": "An error has occurred"}))
        }
    }
}

pub fn show(data: (State<AppState>, Path<PathParameters>)) -> HttpResponse {
    let (state, parameters) = data;
    let connection = state.database.get_connection();
    let artist_response = Artist::find(&parameters.id, &*connection);

    match artist_response {
        Ok(artist) => HttpResponse::Ok().json(&artist),
        Err(_e) => HttpResponse::NotFound().json(json!({"error": "Artist not found"})),
    }
}

pub fn create(data: (State<AppState>, Json<NewArtist>)) -> HttpResponse {
    let (state, new_artist) = data;
    let connection = state.database.get_connection();
    let artist_response = new_artist.commit(&*connection);

    match artist_response {
        Ok(artist) => HttpResponse::Created().json(&artist),
        Err(_e) => HttpResponse::BadRequest().json(json!({"error": "An error has occurred"})),
    }
}

pub fn update(
    data: (
        State<AppState>,
        Path<PathParameters>,
        Json<UserEditableAttributes>,
    ),
) -> HttpResponse {
    let (state, parameters, artist_parameters) = data;
    let connection = state.database.get_connection();

    let artist_response = Artist::find(&parameters.id, &*connection);

    match artist_response {
        Ok(artist) => {
            let artist_update_response = artist.update(&artist_parameters, &*connection);

            match artist_update_response {
                Ok(updated_artist) => HttpResponse::Ok().json(&updated_artist),
                Err(_e) => {
                    HttpResponse::BadRequest().json(json!({"error": "An error has occurred"}))
                }
            }
        }
        Err(_e) => HttpResponse::NotFound().json(json!({"error": "Artist not found"})),
    }
}

pub fn destroy(data: (State<AppState>, Path<PathParameters>)) -> HttpResponse {
    let (state, parameters) = data;
    let connection = state.database.get_connection();
    let artist_response = Artist::find(&parameters.id, &*connection);

    match artist_response {
        Ok(artist) => {
            let artist_destroy_response = artist.destroy(&*connection);
            match artist_destroy_response {
                Ok(_destroyed_count) => HttpResponse::Ok().json(json!({})),
                Err(_e) => {
                    HttpResponse::BadRequest().json(json!({"error": "An error has occurred"}))
                }
            }
        }
        Err(_e) => HttpResponse::NotFound().json(json!({"error": "Artist was not found"})),
    }
}
