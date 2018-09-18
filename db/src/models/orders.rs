use chrono::prelude::*;
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use models::payments::Payment;
use models::PaymentMethods;
use models::{
    Event, FeeSchedule, FeeScheduleRange, OrderItemTypes, OrderStatus, OrderTypes, Organization,
    TicketInstance, TicketPricing, TicketType, User,
};
use schema::{order_items, orders};
use time::Duration;
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
    order_type: String,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "orders"]
pub struct NewOrder {
    user_id: Uuid,
    status: String,
    expires_at: NaiveDateTime,
    order_type: String,
}

impl NewOrder {
    pub fn commit(&self, conn: &PgConnection) -> Result<Order, DatabaseError> {
        use schema::orders;
        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new order",
            diesel::insert_into(orders::table)
                .values(self)
                .get_result(conn),
        )
    }
}

impl Order {
    pub fn create(user_id: Uuid, order_type: OrderTypes) -> NewOrder {
        NewOrder {
            user_id,
            status: OrderStatus::Draft.to_string(),
            expires_at: Utc::now().naive_utc() + Duration::minutes(15),
            order_type: order_type.to_string(),
        }
    }

    pub fn status(&self) -> OrderStatus {
        return OrderStatus::parse(&self.status).unwrap();
    }

    pub fn find_cart_for_user(user_id: Uuid, conn: &PgConnection) -> Result<Order, DatabaseError> {
        orders::table
            .filter(orders::user_id.eq(user_id))
            .filter(orders::status.eq("Draft"))
            .filter(orders::order_type.eq("Cart"))
            .first(conn)
            .to_db_error(
                errors::ErrorCode::QueryError,
                "Could not load cart for user",
            )
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<Order, DatabaseError> {
        orders::table
            .filter(orders::id.eq(id))
            .first(conn)
            .to_db_error(errors::ErrorCode::QueryError, "Could not find order")
    }

    pub fn add_tickets(
        &self,
        ticket_type_id: Uuid,
        quantity: i64,
        conn: &PgConnection,
    ) -> Result<Vec<TicketInstance>, DatabaseError> {
        let ticket_pricing = TicketPricing::get_current_ticket_pricing(ticket_type_id, conn)?;

        let organization = Organization::find(
            Event::find(TicketType::find(ticket_type_id, conn)?.event_id, conn)?.organization_id,
            conn,
        )?;

        let fee_schedule_range = FeeSchedule::find(organization.fee_schedule_id, conn)?
            .get_range(ticket_pricing.price_in_cents, conn)?;

        let fee_schedule_range = fee_schedule_range.unwrap();

        let order_item = NewTicketsOrderItem {
            order_id: self.id,
            item_type: OrderItemTypes::Tickets.to_string(),
            ticket_pricing_id: ticket_pricing.id,
            fee_schedule_range_id: fee_schedule_range.id,
            cost: ticket_pricing.price_in_cents * quantity,
        }.commit(conn)?;

        let fee_item = NewFeesOrderItem {
            order_id: self.id,
            item_type: OrderItemTypes::Fees.to_string(),
            cost: fee_schedule_range.fee * quantity,
            parent_id: order_item.id,
        }.commit(conn)?;

        TicketInstance::reserve_tickets(
            &order_item,
            &self.expires_at,
            ticket_type_id,
            None,
            quantity,
            conn,
        )
    }

    pub fn items(&self, conn: &PgConnection) -> Result<Vec<OrderItem>, DatabaseError> {
        OrderItem::find_for_order(self.id, conn)
    }

    pub fn add_external_payment(
        &mut self,
        external_reference: String,
        current_user_id: Uuid,
        amount: i64,
        conn: &PgConnection,
    ) -> Result<Payment, DatabaseError> {
        diesel::update(&*self)
            .set((
                orders::status.eq(&self.status),
                orders::updated_at.eq(dsl::now),
            )).execute(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not update order")?;

        let payment = Payment::create(
            self.id,
            current_user_id,
            PaymentMethods::External,
            external_reference,
            amount,
        ).commit(conn)?;

        // TODO: Check if total paid is equal to total amount
        self.status = OrderStatus::Paid.to_string();

        // TODO: Move the tickets to the user's wallet.
        //        let target_wallet_id = self.target_wallet_id(conn)?;
        //        for order_item in self.order_items(conn)? {
        //            if order_item.item_type() == OrderItemTypes::Tickets {
        //                for ticket in order_item.tickets(conn)? {
        //                    ticket.move_to_wallet(target_wallet_id, conn)?;
        //                }
        //            }
        //        }

        Ok(payment)
    }

    pub fn calculate_total(&self, conn: &PgConnection) -> Result<u32, DatabaseError> {
        let order_items = self.items(conn)?;
        let mut total = 0;

        for item in &order_items {
            total += item.cost;
        }

        Ok(total as u32)
    }
}

#[derive(Identifiable, Associations, Queryable, AsChangeset)]
#[belongs_to(Order)]
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct OrderItem {
    pub id: Uuid,
    order_id: Uuid,
    item_type: String,
    pub cost: i64,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    ticket_pricing_id: Option<Uuid>,
    fee_schedule_range_id: Option<Uuid>,
    parent_id: Option<Uuid>,
}

impl OrderItem {
    fn create_tickets(order_id: Uuid, ticket_type_id: Uuid, quantity: u32) -> NewTicketsOrderItem {
        unimplemented!()

        //        NewTicketsOrderItem {
        //            order_id,
        //            ticket_type_id,
        //            item_type: OrderItemTypes::Tickets.to_string(),
        //            quantity: quantity as i64,
        //        }
    }

    fn item_type(&self) -> OrderItemTypes {
        OrderItemTypes::parse(&self.item_type).unwrap()
    }

    fn find(
        order_id: Uuid,
        ticket_type_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Option<OrderItem>, errors::DatabaseError> {
        unimplemented!()
        //        order_items::table
        //            .filter(order_items::order_id.eq(order_id))
        //            .filter(order_items::ticket_type_id.eq(ticket_type_id))
        //            .first(conn)
        //            .optional()
        //            .to_db_error(
        //                errors::ErrorCode::QueryError,
        //                "Could not retrieve order item",
        //            )
    }

    fn update(&self, conn: &PgConnection) -> Result<(), DatabaseError> {
        unimplemented!()
        //        diesel::update(self)
        //            .set((
        //                order_items::quantity.eq(self.quantity),
        //                order_items::updated_at.eq(dsl::now),
        //            ))
        //            .execute(conn)
        //            .map(|_| ())
        //            .to_db_error(
        //                errors::ErrorCode::UpdateError,
        //                "Could not update order item",
        //            )
    }

    //    fn delete(self, conn: &PgConnection) -> Result<(), DatabaseError> {
    //        diesel::delete(&self).execute(conn).map(|_| ()).to_db_error(
    //            errors::ErrorCode::DeleteError,
    //            "Could not delete order item",
    //        )
    //    }

    fn find_for_order(
        order_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<OrderItem>, DatabaseError> {
        order_items::table
            .filter(order_items::order_id.eq(order_id))
            .load(conn)
            .to_db_error(errors::ErrorCode::QueryError, "Could not load order items")
    }
}

#[derive(Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "order_items"]
struct NewTicketsOrderItem {
    order_id: Uuid,
    item_type: String,
    cost: i64,
    ticket_pricing_id: Uuid,
    fee_schedule_range_id: Uuid,
}

impl NewTicketsOrderItem {
    fn commit(self, conn: &PgConnection) -> Result<OrderItem, DatabaseError> {
        diesel::insert_into(order_items::table)
            .values(self)
            .get_result(conn)
            .to_db_error(
                errors::ErrorCode::InsertError,
                "Could not create order item",
            )
    }
}

#[derive(Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "order_items"]
struct NewFeesOrderItem {
    order_id: Uuid,
    item_type: String,
    cost: i64,
    parent_id: Uuid,
}

impl NewFeesOrderItem {
    fn commit(self, conn: &PgConnection) -> Result<OrderItem, DatabaseError> {
        diesel::insert_into(order_items::table)
            .values(self)
            .get_result(conn)
            .to_db_error(
                errors::ErrorCode::InsertError,
                "Could not create order item",
            )
    }
}
