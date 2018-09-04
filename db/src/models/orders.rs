use chrono::NaiveDateTime;
use db::Connectable;
use diesel;
use diesel::prelude::*;
use models::{OrderItemTypes, OrderStatus, OrderTypes, TicketType, User};
use schema::{order_items, orders};
use utils::errors;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Associations, Identifiable, Queryable)]
#[belongs_to(User)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    status: String,
    #[allow(dead_code)]
    order_type: String,
    #[allow(dead_code)]
    created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "orders"]
pub struct NewOrder {
    user_id: Uuid,
    status: String,
    order_type: String,
}

impl NewOrder {
    pub fn commit(&self, conn: &Connectable) -> Result<Order, DatabaseError> {
        use schema::orders;
        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new order",
            diesel::insert_into(orders::table)
                .values(self)
                .get_result(conn.get_connection()),
        )
    }
}

impl Order {
    pub fn create(user_id: Uuid, order_type: OrderTypes) -> NewOrder {
        NewOrder {
            user_id,
            status: OrderStatus::Draft.to_string(),
            order_type: order_type.to_string(),
        }
    }

    pub fn status(&self) -> OrderStatus {
        return OrderStatus::parse(&self.status).unwrap();
    }

    pub fn find_cart_for_user(user_id: Uuid, conn: &Connectable) -> Result<Order, DatabaseError> {
        orders::table
            .filter(orders::user_id.eq(user_id))
            .filter(orders::status.eq("Draft"))
            .filter(orders::order_type.eq("Cart"))
            .first(conn.get_connection())
            .to_db_error(
                errors::ErrorCode::QueryError,
                "Could not load cart for user",
            )
    }

    pub fn add_tickets(
        &self,
        ticket_type_id: Uuid,
        quantity: i64,
        conn: &Connectable,
    ) -> Result<(), DatabaseError> {
        let item = OrderItem::find(self.id, ticket_type_id, conn)?;
        if item.is_none() {
            if quantity <= 0 {
                return Ok(());
            }

            OrderItem::create_tickets(self.id, ticket_type_id, quantity as u32).commit(conn)?;
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

    pub fn items(&self, conn: &Connectable) -> Result<Vec<OrderItem>, DatabaseError> {
        OrderItem::find_for_order(self.id, conn)
    }

    pub fn checkout(&mut self, conn: &Connectable) -> Result<(), DatabaseError> {
        self.status = OrderStatus::PendingPayment.to_string();
        diesel::update(&*self)
            .set(orders::status.eq(&self.status))
            .execute(conn.get_connection())
            .to_db_error(ErrorCode::UpdateError, "Could not update order")?;
        Ok(())
    }
}

#[derive(Identifiable, Associations, Queryable, AsChangeset)]
#[belongs_to(TicketType)]
#[belongs_to(Order)]
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct OrderItem {
    pub id: Uuid,
    order_id: Uuid,
    item_type: String,
    ticket_type_id: Uuid,
    quantity: i64,
    created_at: NaiveDateTime,
}

impl OrderItem {
    fn create_tickets(order_id: Uuid, ticket_type_id: Uuid, quantity: u32) -> NewTicketsOrderItem {
        NewTicketsOrderItem {
            order_id,
            ticket_type_id,
            item_type: OrderItemTypes::Tickets.to_string(),
            quantity: quantity as i64,
        }
    }

    fn find(
        order_id: Uuid,
        ticket_type_id: Uuid,
        conn: &Connectable,
    ) -> Result<Option<OrderItem>, errors::DatabaseError> {
        order_items::table
            .filter(order_items::order_id.eq(order_id))
            .filter(order_items::ticket_type_id.eq(ticket_type_id))
            .first(conn.get_connection())
            .optional()
            .to_db_error(
                errors::ErrorCode::QueryError,
                "Could not retrieve order item",
            )
    }

    fn update(&self, conn: &Connectable) -> Result<(), DatabaseError> {
        diesel::update(self)
            .set(order_items::quantity.eq(self.quantity))
            .execute(conn.get_connection())
            .map(|_| ())
            .to_db_error(
                errors::ErrorCode::UpdateError,
                "Could not update order item",
            )
    }

    fn delete(self, conn: &Connectable) -> Result<(), DatabaseError> {
        diesel::delete(&self)
            .execute(conn.get_connection())
            .map(|_| ())
            .to_db_error(
                errors::ErrorCode::DeleteError,
                "Could not delete order item",
            )
    }

    fn find_for_order(order_id: Uuid, conn: &Connectable) -> Result<Vec<OrderItem>, DatabaseError> {
        order_items::table
            .filter(order_items::order_id.eq(order_id))
            .load(conn.get_connection())
            .to_db_error(errors::ErrorCode::QueryError, "Could not load order items")
    }
}

#[derive(Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "order_items"]
struct NewTicketsOrderItem {
    order_id: Uuid,
    item_type: String,
    ticket_type_id: Uuid,
    quantity: i64,
}

impl NewTicketsOrderItem {
    fn commit(self, conn: &Connectable) -> Result<OrderItem, DatabaseError> {
        diesel::insert_into(order_items::table)
            .values(self)
            .get_result(conn.get_connection())
            .to_db_error(
                errors::ErrorCode::InsertError,
                "Could not create order item",
            )
    }
}
