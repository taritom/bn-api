use actix_web::{http::header, HttpRequest, HttpResponse, Query};
use bigneon_db::models::analytics::PageView;
use chrono::prelude::*;
use db::Connection;
use errors::BigNeonError;
use server::AppState;

#[derive(Deserialize)]
pub struct PageViewTrackingData {
    event_id: String,
    source: Option<String>,
    medium: Option<String>,
    term: Option<String>,
    content: Option<String>,
    platform: Option<String>,
    campaign: Option<String>,
    url: String,
    code: Option<String>,
    client_id: Option<String>,
}

pub fn track(
    (query, request, connection): (Query<PageViewTrackingData>, HttpRequest<AppState>, Connection),
) -> Result<HttpResponse, BigNeonError> {
    let conn = connection.get();
    let ip_address = request.connection_info().remote().map(|i| i.to_string());
    let user_agent = if let Some(ua) = request.headers().get(header::USER_AGENT) {
        let s = ua.to_str()?;
        Some(s.to_string())
    } else {
        None
    };

    PageView::create(
        Utc::now().naive_utc(),
        query.event_id.clone(),
        query.source.clone().unwrap_or("".to_string()),
        query.medium.clone().unwrap_or("".to_string()),
        query.term.clone().unwrap_or("".to_string()),
        query.content.clone().unwrap_or("".to_string()),
        query.platform.clone().unwrap_or("".to_string()),
        query.campaign.clone().unwrap_or("".to_string()),
        query.url.clone(),
        query.client_id.clone().unwrap_or("".to_string()),
        query.code.clone().unwrap_or("".to_string()),
        ip_address.unwrap_or("".to_string()),
        user_agent.unwrap_or("".to_string()),
    )
    .commit(conn)?;

    Ok(HttpResponse::Ok().finish())
}
