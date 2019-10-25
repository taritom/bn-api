use actix_web::{HttpResponse, State};
use bigneon_db::prelude::*;
use chrono::Utc;
use db::Connection;
use diesel::PgConnection;
use errors::BigNeonError;
use itertools::Itertools;
use server::AppState;
use utils::gen_sitemap;
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

    let event_urls = create_urls(
        &state.config.front_end_url,
        events_slug_ids,
        "tickets".to_string(),
        conn,
    );

    // Cities
    let city_slug_id = Slug::find_by_slug_type("City", conn)?
        .iter()
        .map(|s| s.id)
        .collect_vec();
    let city_urls = create_urls(
        &state.config.front_end_url,
        city_slug_id,
        "cities".to_string(),
        conn,
    );

    // Venues
    let venue_slugs_ids = Slug::find_by_slug_type("Venue", conn)?
        .iter()
        .map(|s| s.id)
        .collect_vec();
    let venue_urls = create_urls(
        &state.config.front_end_url,
        venue_slugs_ids,
        "venues".to_string(),
        conn,
    );

    // Organizations
    let organizations_slugs_ids = Slug::find_by_slug_type("Organization", conn)?
        .iter()
        .map(|s| s.id)
        .collect_vec();
    let organizations_urls = create_urls(
        &state.config.front_end_url,
        organizations_slugs_ids,
        "organizations".to_string(),
        conn,
    );

    let mut urls = event_urls;
    urls.extend(venue_urls);
    urls.extend(organizations_urls);
    urls.extend(city_urls);

    let sitemap_xml = gen_sitemap::create_sitemap(&urls)?;

    Ok(HttpResponse::Ok()
        .content_type("text/xml")
        .body(sitemap_xml)
        .into())
}

fn create_urls(
    front_url: &String,
    slug_ids: Vec<Uuid>,
    url_parm: String,
    conn: &PgConnection,
) -> Vec<String> {
    let slugs = Slug::find_all(slug_ids, conn).unwrap();
    let gen_urls = slugs
        .iter()
        .map(|slug| &slug.slug)
        .unique()
        .map(|s| format!("{}/{}/{}", front_url, url_parm, s))
        .collect_vec();
    gen_urls
}
