use actix_web::{HttpResponse, Path, Query};
use auth::user::User as AuthUser;
use bigneon_db::models::{Organization, Report, Scopes};
use chrono::prelude::*;
use db::Connection;
use errors::*;
use helpers::application;
use models::PathParameters;
use std::str;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ReportQueryParameters {
    pub report: String,
    pub start_utc: Option<NaiveDateTime>,
    pub end_utc: Option<NaiveDateTime>,
    pub event_id: Option<Uuid>,
}

pub fn get_report(
    (connection, query, path, user): (
        Connection,
        Query<ReportQueryParameters>,
        Path<PathParameters>,
        AuthUser,
    ),
) -> Result<HttpResponse, BigNeonError> {
    match query.report.trim() {
        "transaction_details" => transaction_detail_report((connection, query, path, user)),
        "event_summary" => event_summary_report((connection, query, path, user)),
        "ticket_count" => ticket_counts((connection, query, path, user)),
        _ => application::not_found(),
    }
}

pub fn transaction_detail_report(
    (connection, query, path, user): (
        Connection,
        Query<ReportQueryParameters>,
        Path<PathParameters>,
        AuthUser,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    //Check if they have org admin permissions
    let organization = Organization::find(path.id, connection)?;
    if query.event_id.is_some() {
        user.requires_scope_for_organization(
            Scopes::EventFinancialReports,
            &organization,
            connection,
        )?;
    } else {
        user.requires_scope_for_organization(Scopes::OrgReports, &organization, connection)?;
    }

    let result = Report::transaction_detail_report(
        query.event_id,
        Some(path.id),
        query.start_utc,
        query.end_utc,
        connection,
    )?;
    Ok(HttpResponse::Ok().json(result))
}

pub fn event_summary_report(
    (connection, query, path, user): (
        Connection,
        Query<ReportQueryParameters>,
        Path<PathParameters>,
        AuthUser,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    //Check if they have org admin permissions
    let organization = Organization::find(path.id, connection)?;
    if query.event_id.is_some() {
        user.requires_scope_for_organization(
            Scopes::EventFinancialReports,
            &organization,
            connection,
        )?;
    } else {
        return application::unprocessable("event_id parameter is required");
    }

    let result = Report::summary_event_report(
        //We catch the is_none() above so I'll use unwrap here
        query.event_id.unwrap(),
        path.id,
        query.start_utc,
        query.end_utc,
        connection,
    )?;
    Ok(HttpResponse::Ok().json(result))
}

pub fn ticket_counts(
    (connection, query, path, user): (
        Connection,
        Query<ReportQueryParameters>,
        Path<PathParameters>,
        AuthUser,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    //Check if they have org admin permissions
    let organization = Organization::find(path.id, connection)?;
    if query.event_id.is_some() {
        user.requires_scope_for_organization(
            Scopes::EventFinancialReports,
            &organization,
            connection,
        )?;
    } else {
        user.requires_scope_for_organization(Scopes::OrgReports, &organization, connection)?;
    }

    let result = Report::ticket_count_report(query.event_id, Some(path.id), connection)?;
    Ok(HttpResponse::Ok().json(result))
}
