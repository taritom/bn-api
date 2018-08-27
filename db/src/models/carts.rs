use chrono::NaiveDateTime;
use db::Connectable;
use diesel;
use diesel::prelude::*;
use models::TicketAllocation;
use models::User;
use schema::{cart_items, carts};
use utils::errors;
use utils::errors::*;
use uuid::Uuid;

#[derive(Associations, Identifiable, Queryable, AsChangeset)]
#[belongs_to(User)]
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "carts"]
pub struct Cart {
    pub id: Uuid,
    user_id: Uuid,
    created_at: NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "carts"]
pub struct NewCart {
    user_id: Uuid,
}

impl NewCart {
    pub fn commit(self, conn: &Connectable) -> Result<Cart, DatabaseError> {
        diesel::insert_into(carts::table)
            .values(self)
            .get_result(conn.get_connection())
            .to_db_error(errors::ErrorCode::InsertError, "Could not create cart")
    }
}

impl Cart {
    pub fn create(user_id: Uuid) -> NewCart {
        NewCart { user_id }
    }

    pub fn find_for_user(user_id: Uuid, conn: &Connectable) -> Result<Cart, DatabaseError> {
        carts::table
            .filter(carts::user_id.eq(user_id))
            .first(conn.get_connection())
            .to_db_error(
                errors::ErrorCode::QueryError,
                "Could not load cart for user",
            )
    }

    pub fn add_item(
        &self,
        ticket_allocation_id: Uuid,
        quantity: i64,
        conn: &Connectable,
    ) -> Result<(), DatabaseError> {
        let item = CartItem::find(self.id, ticket_allocation_id, conn)?;
        if item.is_none() {
            if quantity <= 0 {
                return Ok(());
            }

            CartItem::create(self.id, ticket_allocation_id, quantity as u32).commit(conn)?;
            Ok(())
        } else {
            let mut item = item.unwrap();
            item.quantity += quantity;
            if item.quantity <= 0 {
                item.delete(conn)
            } else {
                item.update(conn)
            }
        }
    }

    pub fn items(&self, conn: &Connectable) -> Result<Vec<DisplayCartItem>, DatabaseError> {
        let mut items = CartItem::find_for_cart(self.id, conn)?;

        let display_items: Vec<DisplayCartItem> =
            items.drain(0..).map(|i| DisplayCartItem::from(i)).collect();
        Ok(display_items)
    }
}

#[derive(PartialEq, Debug)]
pub struct DisplayCartItem {
    pub ticket_allocation_id: Uuid,
    pub quantity: u32,
}

impl From<CartItem> for DisplayCartItem {
    fn from(item: CartItem) -> Self {
        DisplayCartItem {
            ticket_allocation_id: item.ticket_allocation_id,
            quantity: item.quantity as u32,
        }
    }
}

#[derive(Identifiable, Associations, Queryable, AsChangeset)]
#[belongs_to(TicketAllocation)]
#[belongs_to(Cart)]
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "cart_items"]
pub struct CartItem {
    pub id: Uuid,
    cart_id: Uuid,
    created_at: NaiveDateTime,
    ticket_allocation_id: Uuid,
    quantity: i64,
}

impl CartItem {
    fn create(cart_id: Uuid, ticket_allocation_id: Uuid, quantity: u32) -> NewCartItem {
        NewCartItem {
            cart_id,
            ticket_allocation_id,
            quantity: quantity as i64,
        }
    }

    fn find(
        cart_id: Uuid,
        ticket_allocation_id: Uuid,
        conn: &Connectable,
    ) -> Result<Option<CartItem>, DatabaseError> {
        cart_items::table
            .filter(cart_items::cart_id.eq(cart_id))
            .filter(cart_items::ticket_allocation_id.eq(ticket_allocation_id))
            .first(conn.get_connection())
            .optional()
            .to_db_error(
                errors::ErrorCode::QueryError,
                "Could not retrieve cart item",
            )
    }

    fn update(&self, conn: &Connectable) -> Result<(), DatabaseError> {
        diesel::update(self)
            .set(cart_items::quantity.eq(self.quantity))
            .execute(conn.get_connection())
            .map(|_| ())
            .to_db_error(errors::ErrorCode::UpdateError, "Could not update cart item")
    }

    fn delete(self, conn: &Connectable) -> Result<(), DatabaseError> {
        diesel::delete(&self)
            .execute(conn.get_connection())
            .map(|_| ())
            .to_db_error(errors::ErrorCode::DeleteError, "Could not delete cart item")
    }

    fn find_for_cart(cart_id: Uuid, conn: &Connectable) -> Result<Vec<CartItem>, DatabaseError> {
        cart_items::table
            .filter(cart_items::cart_id.eq(cart_id))
            .load(conn.get_connection())
            .to_db_error(errors::ErrorCode::QueryError, "Could not load cart items")
    }
}

#[derive(Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "cart_items"]
struct NewCartItem {
    cart_id: Uuid,
    ticket_allocation_id: Uuid,
    quantity: i64,
}

impl NewCartItem {
    fn commit(self, conn: &Connectable) -> Result<CartItem, DatabaseError> {
        diesel::insert_into(cart_items::table)
            .values(self)
            .get_result(conn.get_connection())
            .to_db_error(errors::ErrorCode::InsertError, "Could not create cart item")
    }
}
