use actix_web::{HttpResponse, State};
use bigneon_db::prelude::*;
use chrono::Utc;
use db::Connection;
use errors::BigNeonError;
use itertools::Itertools;
use server::AppState;
use utils::gen_sitemap;

pub fn index(
    (connection, state): (Connection, State<AppState>),
) -> Result<HttpResponse, BigNeonError> {
    let conn = connection.get();
    //find all active events
    let events = Event::find_all_active_events(conn)?;
    let slug_ids = events
        .iter()
        .filter(|e| e.publish_date.is_some() && e.publish_date.unwrap() < Utc::now().naive_utc())
        .map(|e| e.slug_id)
        .filter(|s| s.is_some())
        .map(|s| s.unwrap())
        .collect_vec();

    let slugs = Slug::find_all(slug_ids, conn)?;

    let mut urls = slugs
        .iter()
        .map(|slug| format!("{}/tickets/{}", state.config.front_end_url, &slug.slug))
        .collect_vec();

    // Venues
    let venue_slugs = Venue::all(None, conn)?
        .iter()
        .map(|venue| venue.slug_id)
        .filter(|s| s.is_some())
        .map(|s| s.unwrap())
        .collect_vec();

    let slugs = Slug::find_all(venue_slugs, conn)?;

    for s in slugs
        .iter()
        .map(|slug| format!("{}/venues/{}", state.config.front_end_url, &slug.slug))
    {
        urls.push(s);
    }

    // Organizations
    let organizations_slugs = Organization::all(conn)?
        .iter()
        .map(|org| org.slug_id)
        .filter(|s| s.is_some())
        .map(|s| s.unwrap())
        .collect_vec();

    let slugs = Slug::find_all(organizations_slugs, conn)?;

    for s in slugs.iter().map(|slug| {
        format!(
            "{}/organizations/{}",
            state.config.front_end_url, &slug.slug
        )
    }) {
        urls.push(s);
    }

    let sitemap_xml = gen_sitemap::create_sitemap(&urls)?;

    Ok(HttpResponse::Ok()
        .content_type("text/xml")
        .body(sitemap_xml)
        .into())
}
