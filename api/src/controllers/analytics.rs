use actix_web::{http::header, HttpRequest, HttpResponse, Query};
use bigneon_db::models::analytics::PageView;
use bigneon_db::prelude::*;
use chrono::prelude::*;
use db::Connection;
use errors::BigNeonError;
use server::AppState;
use url::form_urlencoded::Parse;
use url::Url;

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

    let platform = Platforms::from_user_agent(user_agent.as_ref().map(|ua| ua.as_str()).unwrap())
        .map(|p| p.to_string())
        .ok();

    let url = Url::parse(query.url.to_string().as_str())?;
    let mut params = url.query_pairs();
    let utm_source = extract_param(&mut params, "utm_source");
    let utm_medium = extract_param(&mut params, "utm_medium");
    let utm_content = extract_param(&mut params, "utm_content");
    let utm_term = extract_param(&mut params, "utm_term");
    let utm_campaign = extract_param(&mut params, "utm_campaign");
    let utm_code = extract_param(&mut params, "code");
    let is_facebook = extract_param(&mut params, "fbclid").map(|_| "facebook".to_string());

    PageView::create(
        Utc::now().naive_utc(),
        query.event_id.clone(),
        query
            .source
            .clone()
            .or(utm_source)
            .or(is_facebook)
            .unwrap_or("".to_string()),
        query.medium.clone().or(utm_medium).unwrap_or("".to_string()),
        query.term.clone().or(utm_term).unwrap_or("".to_string()),
        query.content.clone().or(utm_content).unwrap_or("".to_string()),
        query.platform.clone().or(platform).unwrap_or("".to_string()),
        query.campaign.clone().or(utm_campaign).unwrap_or("".to_string()),
        query.url.clone(),
        query.client_id.clone().unwrap_or("".to_string()),
        query.code.clone().or(utm_code).unwrap_or("".to_string()),
        ip_address.unwrap_or("".to_string()),
        user_agent.unwrap_or("".to_string()),
    )
    .commit(conn)?;

    Ok(HttpResponse::Ok().finish())
}

fn extract_param<'a>(query_params: &mut Parse, name: &str) -> Option<String> {
    query_params.find(|i| &i.0 == name).map(|i| i.1.to_string())
}
