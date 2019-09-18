use actix_web::{http::StatusCode, HttpResponse, Path, Query};
use auth::user::User as AuthUser;
use bigneon_db::models::*;
use db::Connection;
use errors::*;
use models::{PathParameters, WebPayload};

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

    let payload = Settlement::find_for_organization(
        path.id,
        Some(query.limit()),
        Some(query.page() * query.limit()),
        connection,
    )?;

    Ok(WebPayload::new(StatusCode::OK, payload))
}

pub fn show(
    (connection, path, user): (Connection, Path<PathParameters>, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let settlement = Settlement::find(path.id, connection)?;
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
    let settlement = Settlement::find(path.id, connection)?;
    settlement.destroy(connection)?;
    Ok(HttpResponse::Ok().json({}))
}
