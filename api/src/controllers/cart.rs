use actix_web::HttpResponse;
use actix_web::Json;
use actix_web::State;
use auth::user::User;
use bigneon_db::models::{Order, OrderTypes};
use errors::ConvertToWebError;
use server::AppState;
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

pub fn add((state, json, user): (State<AppState>, Json<AddToCartRequest>, User)) -> HttpResponse {
    let conn = state.database.get_connection();
    // Find the current cart of the user, if it exists.
    let current_cart = match Order::find_cart_for_user(user.id(), &*conn) {
        Ok(c) => Some(c),
        Err(e) => match e.code {
            2000 => None,
            _ => return e.to_response(),
        },
    };

    let cart: Order;

    // Create it if there isn't one
    if current_cart.is_none() {
        cart = match Order::create(user.id(), OrderTypes::Cart).commit(&*conn) {
            Ok(o) => o,
            Err(e) => return e.to_response(),
        };
    } else {
        cart = current_cart.unwrap();
    }

    let data = json.into_inner();

    // Add the item
    match cart.add_tickets(data.ticket_type_id, data.quantity, &*conn) {
        Ok(_o) => HttpResponse::Ok().json(&AddToCartResponse { cart_id: cart.id }),
        Err(e) => return e.to_response(),
    }
}
