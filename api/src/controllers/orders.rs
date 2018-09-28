use actix_web::HttpResponse;
use auth::user::User;
use bigneon_db::models::*;
use db::Connection;
use errors::BigNeonError;

pub fn index((conn, user): (Connection, User)) -> Result<HttpResponse, BigNeonError> {
    user.requires_scope(Scopes::OrderRead)?;
    let orders = Order::find_for_user_for_display(user.id(), conn.get())?;
    Ok(HttpResponse::Ok().json(json!(orders)))
}
