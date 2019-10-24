use actix_web::{HttpResponse, State};
use bigneon_db::prelude::*;
use chrono::Utc;
use db::Connection;
use errors::BigNeonError;
use itertools::Itertools;
use server::AppState;
use utils::gen_sitemap;
use diesel::PgConnection;
use uuid::Uuid;

pub fn index(
    (connection, state): (Connection, State<AppState>),
) -> Result<HttpResponse, BigNeonError> {

    let conn = connection.get();

    //find all active events
    let events_slug_ids = Event::find_all_active_events(conn)?
        .iter()
        .filter(|e| e.publish_date.is_some() && e.publish_date.unwrap() < Utc::now().naive_utc())
        .map(|e| e.slug_id)
        .filter(|s| s.is_some())
        .map(|s| s.unwrap())
        .collect_vec();

    let event_urls = create_urls( &state.config.front_end_url, events_slug_ids, "tickets".to_string(), conn);

    // Venues
    let venue_slugs_ids = Venue::all(None, conn)?
        .iter()
        .map(|venue| venue.slug_id)
        .filter(|s| s.is_some())
        .map(|s| s.unwrap())
        .collect_vec();

    let venue_urls = create_urls( &state.config.front_end_url, venue_slugs_ids, "venues".to_string(), conn);

    // Organizations
    let organizations_slugs_ids = Organization::all(conn)?
        .iter()
        .map(|org| org.slug_id)
        .filter(|s| s.is_some())
        .map(|s| s.unwrap())
        .collect_vec();

    let organizations_urls = create_urls( &state.config.front_end_url, organizations_slugs_ids, "organizations".to_string(), conn);

    let mut urls = event_urls;
    urls.extend(venue_urls);
    urls.extend(organizations_urls);

    let sitemap_xml = gen_sitemap::create_sitemap(&urls)?;

    Ok(HttpResponse::Ok()
        .content_type("text/xml")
        .body(sitemap_xml)
        .into())
}

fn create_urls(front_url: &String, slug_ids: Vec<Uuid>, url_parm: String, conn: &PgConnection) -> Vec<String> {
    let slugs = Slug::find_all(slug_ids, conn).unwrap();
    let gen_urls = slugs.iter().map(|slug| format!("{}/{}/{}", front_url, url_parm, slug.slug)).collect_vec();
    gen_urls
}
