use crate::db::Connection;
use crate::errors::BigNeonError;
use crate::server::AppState;
use crate::utils::gen_sitemap;
use actix_web::{HttpResponse, State};

pub fn index((connection, state): (Connection, State<AppState>)) -> Result<HttpResponse, BigNeonError> {
    let conn = connection.get();

    let sitemap_xml = gen_sitemap::create_sitemap_conn(conn, &state.config.front_end_url)?;

    Ok(HttpResponse::Ok().content_type("text/xml").body(sitemap_xml).into())
}
