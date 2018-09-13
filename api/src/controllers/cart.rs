use actix_web::HttpResponse;
use actix_web::Json;
use auth::user::User;
use bigneon_db::models::{Order, OrderTypes};
use bigneon_db::utils::errors::Optional;
use db::Connection;
use errors::BigNeonError;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AddToCartRequest {
    pub ticket_type_id: Uuid,
    pub quantity: i64,
}

#[derive(Serialize)]
pub struct AddToCartResponse {
    pub cart_id: Uuid,
}

pub fn add(
    (connection, json, user): (Connection, Json<AddToCartRequest>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    // Find the current cart of the user, if it exists.
    let current_cart = Order::find_cart_for_user(user.id(), connection).optional()?;
    let cart: Order;

    // Create it if there isn't one
    if current_cart.is_none() {
        cart = Order::create(user.id(), OrderTypes::Cart).commit(connection)?;
    } else {
        cart = current_cart.unwrap();
    }

    // Add the item
    cart.add_tickets(json.ticket_type_id, json.quantity, connection)?;

    Ok(HttpResponse::Created().json(&AddToCartResponse { cart_id: cart.id }))
}
