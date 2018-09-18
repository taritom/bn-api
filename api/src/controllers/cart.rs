use actix_web::HttpResponse;
use actix_web::Json;
use actix_web::Path;
use auth::user::User;
use bigneon_db::models::{Order, OrderStatus, OrderTypes, Scopes};
use bigneon_db::utils::errors::Optional;
use db::Connection;
use errors::BigNeonError;
use helpers::application;
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

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

#[derive(Deserialize)]
pub struct CheckoutCartRequest {
    pub amount: i64,
    pub method: PaymentRequest,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum PaymentRequest {
    External { reference: String },
}

pub fn show((connection, user): (Connection, User)) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let order = Order::find_cart_for_user(user.id(), connection).optional()?;
    if order.is_none() {
        return Ok(HttpResponse::Ok().json(json!({})));
    }

    let order = order.unwrap();

    #[derive(Serialize)]
    struct DisplayCart {
        id: Uuid,
        items: Vec<DisplayCartItem>,
        total: i64,
    }

    #[derive(Serialize)]
    struct DisplayCartItem {
        id: Uuid,
        item_type: String,
        quantity: u32,
        cost: i64,
    }

    let items: Vec<DisplayCartItem> = order
        .items(connection)?
        .iter()
        .map(|i| DisplayCartItem {
            id: i.id,
            item_type: i.item_type().to_string(),
            quantity: i.quantity.unwrap_or_default() as u32,
            cost: i.cost,
        }).collect();
    let r = DisplayCart {
        id: order.id,
        items,
        total: order.calculate_total(connection)?,
    };

    Ok(HttpResponse::Ok().json(r))
}

pub fn checkout(
    (connection, json, path, user): (
        Connection,
        Json<CheckoutCartRequest>,
        Path<PathParameters>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let req = json.into_inner();
    match &req.method {
        PaymentRequest::External { reference } => {
            checkout_external(connection, path.id, reference, &req, user)
        }
        _ => unimplemented!(),
    }
}

// TODO: This should actually probably move to an `orders` controller, since the
// user will not be calling this.
fn checkout_external(
    conn: Connection,
    order_id: Uuid,
    reference: &String,
    checkout_request: &CheckoutCartRequest,
    user: User,
) -> Result<HttpResponse, BigNeonError> {
    let connection = conn.get();
    if !user.has_scope(Scopes::OrderMakeExternalPayment, None, connection)? {
        return application::unauthorized();
    }

    let mut order = Order::find(order_id, connection)?;

    if order.status() != OrderStatus::Draft {
        return application::unprocessable(
            "Could not complete this cart because it is not in the correct status",
        );
    }

    let payment = order.add_external_payment(
        reference.to_string(),
        user.id(),
        checkout_request.amount,
        connection,
    )?;

    Ok(HttpResponse::Ok().json(json!({"payment_id": payment.id})))
}
