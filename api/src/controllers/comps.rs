use actix_web::{HttpResponse, Json, Path, Query};
use auth::user::User;
use bigneon_db::models::*;
use db::Connection;
use errors::BigNeonError;
use helpers::application;
use models::{CompPathParameters, Paging, PagingParameters, PathParameters, Payload};

#[derive(Default, Deserialize, Serialize)]
pub struct NewCompRequest {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub quantity: u16,
}

pub fn index(
    (conn, path, query_parameters, user): (
        Connection,
        Path<PathParameters>,
        Query<PagingParameters>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let conn = conn.get();
    let hold = Hold::find(path.id, conn)?;
    user.requires_scope_for_organization(Scopes::CompWrite, &hold.organization(conn)?, conn)?;

    //TODO implement proper paging on db
    let comps = hold.comps(conn)?;

    let query_parameters = Paging::new(&query_parameters.into_inner());
    let comp_count = comps.len();
    let mut payload = Payload {
        data: comps,
        paging: Paging::clone_with_new_total(&query_parameters, comp_count as u64),
    };
    payload.paging.limit = comp_count as u64;
    Ok(HttpResponse::Ok().json(&payload))
}

pub fn show(
    (conn, path, user): (Connection, Path<CompPathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let conn = conn.get();
    let hold = Hold::find(path.hold_id, conn)?;
    user.requires_scope_for_organization(Scopes::CompWrite, &hold.organization(conn)?, conn)?;

    let comp = Comp::find(path.hold_id, path.comp_id, conn)?;
    Ok(HttpResponse::Ok().json(&comp))
}

pub fn create(
    (conn, new_comp, path, user): (Connection, Json<NewCompRequest>, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let conn = conn.get();
    let hold = Hold::find(path.id, conn)?;
    user.requires_scope_for_organization(Scopes::CompWrite, &hold.organization(conn)?, conn)?;
    let new_comp = new_comp.into_inner();
    let comp = (NewComp {
        hold_id: hold.id,
        name: new_comp.name,
        email: new_comp.email,
        phone: new_comp.phone,
        quantity: new_comp.quantity as i32,
    }).commit(conn)?;

    application::created(json!(comp))
}

pub fn update(
    (conn, req, path, user): (
        Connection,
        Json<UpdateCompAttributes>,
        Path<CompPathParameters>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let conn = conn.get();

    let hold = Hold::find(path.hold_id, conn)?;
    user.requires_scope_for_organization(Scopes::CompWrite, &hold.organization(conn)?, conn)?;

    let comp = Comp::find(path.hold_id, path.comp_id, conn)?;
    Ok(HttpResponse::Ok().json(comp.update(req.into_inner(), conn)?))
}

pub fn destroy(
    (conn, path, user): (Connection, Path<CompPathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let conn = conn.get();
    let hold = Hold::find(path.hold_id, conn)?;
    user.requires_scope_for_organization(Scopes::CompWrite, &hold.organization(conn)?, conn)?;

    let comp = Comp::find(path.hold_id, path.comp_id, conn)?;
    comp.destroy(&*conn)?;
    Ok(HttpResponse::Ok().json(json!({})))
}
