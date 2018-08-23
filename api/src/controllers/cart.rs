use actix_web::Error;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Json;
use actix_web::Responder;
use actix_web::State;
use auth::user::User;
use bigneon_db::models::Cart;
use errors::ConvertToWebError;
use serde_json;
use server::AppState;
use uuid::Uuid;

use bigneon_db::utils::errors::DatabaseError;

#[derive(Deserialize)]
pub struct AddToCartRequest {
    pub ticket_allocation_id: Uuid,
    pub quantity: i64,
}

#[derive(Serialize)]
pub struct AddToCartResponse {
    pub cart_id: Uuid,
}

pub fn add((state, json, user): (State<AppState>, Json<AddToCartRequest>, User)) -> HttpResponse {
    let conn = state.database.get_connection();
    // Find the current cart of the user, if it exists.
    let current_cart = match Cart::find_for_user(user.id(), &*conn) {
        Ok(c) => Some(c),
        Err(e) => match e.code {
            2000 => None,
            _ => return e.to_response(),
        },
    };

    let cart: Cart;

    // Create it if there isn't one
    if current_cart.is_none() {
        cart = match Cart::create(user.id()).commit(&*conn) {
            Ok(o) => o,
            Err(e) => return e.to_response(),
        };
    } else {
        cart = current_cart.unwrap();
    }

    let data = json.into_inner();

    // Add the item
    match cart.add_item(data.ticket_allocation_id, data.quantity, &*conn) {
        Ok(o) => HttpResponse::Ok().json(&AddToCartResponse { cart_id: cart.id }),
        Err(e) => return e.to_response(),
    }
}
