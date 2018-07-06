use actix_web::{HttpRequest, Json, Path, Result, State};
use bigneon_db::models::{Artist, NewArtist};
use database::ConnectionGranting;
use serde_json;
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    id: Uuid,
}

pub fn index(request: HttpRequest<AppState>) -> Result<String> {
    let artists = Artist::all(&*request.state().database.get_connection());
    Ok(serde_json::to_string(&artists)?)
}

pub fn show(data: (State<AppState>, Path<PathParameters>)) -> Result<String> {
    let (state, parameters) = data;
    let artist = Artist::find(&parameters.id, &*state.database.get_connection());
    Ok(serde_json::to_string(&artist)?)
}

pub fn create(data: (State<AppState>, Json<NewArtist>)) -> Result<String> {
    let (state, new_artist) = data;
    let artist = new_artist
        .commit(&*state.database.get_connection())
        .expect("Failed to create artist");
    Ok(serde_json::to_string(&artist)?)
}

pub fn update(data: (State<AppState>, Path<PathParameters>, Json<NewArtist>)) -> Result<String> {
    let (state, parameters, artist_parameters) = data;
    let artist = Artist::find(&parameters.id, &*state.database.get_connection());
    let artist = artist.update_attributes(&artist_parameters, &*state.database.get_connection());
    Ok(serde_json::to_string(&artist)?)
}

pub fn destroy(data: (State<AppState>, Path<PathParameters>)) -> Result<String> {
    let (state, parameters) = data;
    let artist = Artist::find(&parameters.id, &*state.database.get_connection());
    artist.destroy(&*state.database.get_connection());
    Ok("{}".to_string())
}
