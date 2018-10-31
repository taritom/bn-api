use actix_web::{HttpResponse, Path, Query};
use auth::user::User;
use bigneon_db::models::*;
use db::Connection;
use errors::BigNeonError;
use helpers::application;
use models::PathParameters;

pub fn index(
    (conn, query_parameters, user): (Connection, Query<PagingParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    //@TODO Implement proper paging on db

    user.requires_scope(Scopes::OrderRead)?;
    let orders = Order::find_for_user_for_display(user.id(), conn.get())?;

    Ok(HttpResponse::Ok().json(&Payload::new(orders, query_parameters.into_inner().into())))
}

pub fn show(
    (conn, path, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    user.requires_scope(Scopes::OrderRead)?;
    let order = Order::find(path.id, conn.get())?;

    if order.user_id != user.id() || order.status == OrderStatus::Draft.to_string() {
        return application::forbidden("You do not have access to this order");
    }

    Ok(HttpResponse::Ok().json(json!(order.for_display(conn.get())?)))
}
