use actix_web::{http::StatusCode, HttpResponse, Json, Query, State};
use auth::user::User as AuthUser;
use bigneon_db::models::concerns::users::password_resetable::PasswordResetable;
use bigneon_db::models::enums::{OrderTypes, PaymentMethods};
use bigneon_db::models::{Payload, Report, Scopes, TransactionReportRow, User};
use bigneon_db::utils::errors::Optional;
use chrono::prelude::*;
use communications::mailers;
use db::Connection;
use errors::*;
use helpers::application;
use models::WebPayload;
use server::AppState;
use std::str;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ReportQueryParameters {
    pub start: Option<NaiveDateTime>,
    pub end: Option<NaiveDateTime>,
    pub organization_id: Option<Uuid>,
    pub event_id: Option<Uuid>,
}

pub fn transaction_detail_report(
    (connection, query, user): (Connection, Query<ReportQueryParameters>, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    //Check if they have org admin permissions
    user.requires_scope(Scopes::OrgAdmin)?;
    //There must be an org_id or an event_id
    if query.organization_id.is_none() && query.event_id.is_none() {
        return application::unprocessable("Organization ID or Event ID must be specified");
    }
    let result =
        Report::transaction_detail_report(query.event_id, query.organization_id, connection.get())?;
    Ok(HttpResponse::Ok().json(result))
}
