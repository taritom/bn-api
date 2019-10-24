use actix_web::HttpResponse;
use bigneon_db::models::*;
use db::Connection;
use errors::*;

pub fn index(connection: Connection) -> Result<HttpResponse, BigNeonError> {
    let genres = Genre::all(connection.get())?;
    Ok(HttpResponse::Ok()
        .json(json!({"genres": &genres.into_iter().map(|g| g.name).collect::<Vec<String>>()})))
}

#[derive(Serialize)]
struct GenreListItem {
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    id: String,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    name: String,
}

pub fn list(connection: Connection) -> Result<HttpResponse, BigNeonError> {
    let genres = Genre::all(connection.get())?;
    Ok(HttpResponse::Ok()
        .json(json!({"genres": &genres.into_iter().map(|g| GenreListItem{id: g.id.to_string(), name: g.name }).collect::<Vec<GenreListItem >>()})))
}
