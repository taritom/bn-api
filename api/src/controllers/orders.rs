use actix_web::{HttpResponse, Path, Query};
use auth::user::User;
use bigneon_db::models::*;
use db::Connection;
use errors::BigNeonError;
use extractors::*;
use helpers::application;
use models::PathParameters;
use uuid::Uuid;

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

pub fn update(
    (conn, path, json, user): (
        Connection,
        Path<PathParameters>,
        Json<UpdateOrderAttributes>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    user.requires_scope(Scopes::OrderRead)?;
    let conn = conn.get();

    let order = Order::find(path.id, conn)?;
    if order.user_id != user.id() {
        return application::forbidden("You do not have access to this order");
    }

    let order = order.update(json.into_inner(), conn)?;

    Ok(HttpResponse::Ok().json(order.for_display(conn)?))
}

pub fn tickets(
    (conn, path, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let conn = conn.get();
    let order = Order::find(path.id, conn)?;
    // TODO: Only show the redeem key for orgs that the user has access to redeem
    let orgs: Vec<Uuid> = user
        .user
        .organizations(conn)?
        .iter()
        .map(|o| o.id)
        .collect();
    let mut results = vec![];
    for item in order
        .items(conn)?
        .iter()
        .filter(|t| t.item_type() == Ok(OrderItemTypes::Tickets))
    {
        if order.user_id != user.id() && order.on_behalf_of_user_id != Some(user.id()) {
            if item.event_id.is_none()
                || !orgs.contains(&Event::find(item.event_id.unwrap(), conn)?.organization_id)
            {
                continue;
            }
        }

        for t in TicketInstance::find_for_order_item(item.id, conn)? {
            results.push(TicketInstance::show_redeemable_ticket(t.id, conn)?);
        }
    }
    Ok(HttpResponse::Ok().json(results))
}
