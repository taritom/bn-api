use actix_web::{http::StatusCode, HttpResponse, Path, Query};
use auth::user::User as AuthUser;
use bigneon_db::models::*;
use db::Connection;
use errors::*;
use extractors::*;
use models::{PathParameters, WebPayload};

pub fn create(
    (connection, new_settlement_json, path, auth_user): (
        Connection,
        Json<NewSettlementRequest>,
        Path<PathParameters>,
        AuthUser,
    ),
) -> Result<HttpResponse, BigNeonError> {
    auth_user.requires_scope(Scopes::OrgAdmin)?;
    let connection = connection.get();
    let new_settlement = new_settlement_json.commit(path.id, auth_user.id(), connection)?;
    Ok(HttpResponse::Created().json(&new_settlement))
}

pub fn prepare(
    (connection, new_settlement_json, path, auth_user): (
        Connection,
        Json<NewSettlementRequest>,
        Path<PathParameters>,
        AuthUser,
    ),
) -> Result<HttpResponse, BigNeonError> {
    auth_user.requires_scope(Scopes::OrgAdmin)?;
    let connection = connection.get();

    let new_settlement = new_settlement_json.prepare(path.id, auth_user.id(), connection)?;
    Ok(HttpResponse::Created().json(&new_settlement))
}

pub fn index(
    (connection, query, path, user): (
        Connection,
        Query<PagingParameters>,
        Path<PathParameters>,
        AuthUser,
    ),
) -> Result<WebPayload<Settlement>, BigNeonError> {
    let connection = connection.get();
    let organization = Organization::find(path.id, connection)?;
    user.requires_scope_for_organization(Scopes::OrgFinancialReports, &organization, connection)?;

    let (settlements, total) = Settlement::index(
        path.id,
        Some(query.limit()),
        Some(query.page() * query.limit()),
        connection,
    )?;

    let mut paging = Paging::new(query.page(), query.limit());
    paging.total = total as u64;
    let payload = Payload::new(settlements, paging);
    Ok(WebPayload::new(StatusCode::OK, payload))
}

pub fn show(
    (connection, path, user): (Connection, Path<PathParameters>, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let settlement = Settlement::read(path.id, connection)?;
    let organization = Organization::find(settlement.organization_id, connection)?;
    user.requires_scope_for_organization(Scopes::OrgFinancialReports, &organization, connection)?;
    let display_settlement: DisplaySettlement = settlement.for_display(connection)?;
    Ok(HttpResponse::Ok().json(&display_settlement))
}

pub fn destroy(
    (connection, path, user): (Connection, Path<PathParameters>, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    user.requires_scope(Scopes::OrgAdmin)?;
    let settlement = Settlement::read(path.id, connection)?;
    settlement.destroy(connection)?;
    Ok(HttpResponse::Ok().json({}))
}
