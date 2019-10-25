use actix_web::HttpResponse;
use bigneon_db::models::*;
use db::Connection;
use errors::*;

pub fn index(connection: Connection) -> Result<HttpResponse, BigNeonError> {
    let genres = Genre::all(connection.get())?;
    Ok(HttpResponse::Ok()
        .json(json!({"genres": &genres.into_iter().map(|g| g.name).collect::<Vec<String>>()})))
}
