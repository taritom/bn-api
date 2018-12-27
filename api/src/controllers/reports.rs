use actix_web::{HttpResponse, Path, Query};
use auth::user::User as AuthUser;
use bigneon_db::models::{Organization, Report, Scopes, User};
use chrono::prelude::*;
use db::Connection;
use errors::*;
use models::PathParameters;
use std::str;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ReportQueryParameters {
    pub start: Option<NaiveDateTime>,
    pub end: Option<NaiveDateTime>,
    pub event_id: Option<Uuid>,
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
    user.requires_scope_for_organization(Scopes::OrgRead, &organization, connection)?;

    let result = Report::transaction_detail_report(query.event_id, Some(path.id), connection)?;
    Ok(HttpResponse::Ok().json(result))
}
