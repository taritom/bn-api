use actix_web::{http::StatusCode, HttpResponse, Json, State, Query};
use auth::user::User as AuthUser;
use bigneon_db::models::concerns::users::password_resetable::PasswordResetable;
use bigneon_db::models::enums::{OrderTypes, PaymentMethods};
use bigneon_db::models::{User, Report, TransactionReportRow, Payload};
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
    (connection, query, user): (Connection, Query<ReportQueryParameters>, AuthUser))
    -> Result<HttpResponse, BigNeonError> {
    //If the org_id is set, check if they have org report permissions
    //If the org_id and the event_id is set, just get the event_id and ignore the org_id
    //If neither are set, throw a NotFound error
    let result = Report::transaction_detail_report(query.event_id, query.organization_id, connection.get())?;
    Ok(HttpResponse::Ok().json(result))
}