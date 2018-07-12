use actix_web::{HttpRequest, Json, Path, Result, State};
use bigneon_db::models::{Artist, NewArtist};
use serde_json;
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

pub fn index(request: HttpRequest<AppState>) -> Result<String> {
    let connection = request.state().database.get_connection();
    let artists_response = Artist::all(&*connection);
    match artists_response {
        Ok(artists) => Ok(serde_json::to_string(&artists)?),
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}

pub fn show(data: (State<AppState>, Path<PathParameters>)) -> Result<String> {
    let (state, parameters) = data;
    let connection = state.database.get_connection();
    let artist_response = Artist::find(&parameters.id, &*connection);

    match artist_response {
        Ok(artist) => Ok(serde_json::to_string(&artist)?),
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}

pub fn create(data: (State<AppState>, Json<NewArtist>)) -> Result<String> {
    let (state, new_artist) = data;
    let connection = state.database.get_connection();
    let artist_response = new_artist.commit(&*connection);

    match artist_response {
        Ok(artist) => Ok(serde_json::to_string(&artist)?),
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}

pub fn update(data: (State<AppState>, Path<PathParameters>, Json<NewArtist>)) -> Result<String> {
    let (state, parameters, artist_parameters) = data;
    let connection = state.database.get_connection();

    let artist_response = Artist::find(&parameters.id, &*connection);

    match artist_response {
        Ok(artist) => {
            let artist_update_response = artist.update_attributes(&artist_parameters, &*connection);

            match artist_update_response {
                Ok(updated_artist) => Ok(serde_json::to_string(&updated_artist)?),
                Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
            }
        }
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}

pub fn destroy(data: (State<AppState>, Path<PathParameters>)) -> Result<String> {
    let (state, parameters) = data;
    let connection = state.database.get_connection();
    let artist_response = Artist::find(&parameters.id, &*connection);

    match artist_response {
        Ok(artist) => {
            let artist_destroy_response = artist.destroy(&*connection);
            match artist_destroy_response {
                Ok(_destroyed_records) => Ok("{}".to_string()),
                Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
            }
        }
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}
