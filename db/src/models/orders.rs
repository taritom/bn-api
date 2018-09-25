use chrono::prelude::*;
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use models::*;
use schema::orders;
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
            .get_range(ticket_pricing.price_in_cents, conn)?
            .unwrap();

        let order_item = NewTicketsOrderItem {
            order_id: self.id,
            item_type: OrderItemTypes::Tickets.to_string(),
            quantity,
            ticket_pricing_id: ticket_pricing.id,
            fee_schedule_range_id: fee_schedule_range.id,
            unit_price_in_cents: ticket_pricing.price_in_cents,
        }.commit(conn)?;
        order_item.update_fees(conn)?;

        TicketInstance::reserve_tickets(
            &order_item,
            &self.expires_at,
            ticket_type_id,
            None,
            quantity,
            conn,
        )
    }

    pub fn remove_tickets(
        &self,
        mut order_item: OrderItem,
        quantity: Option<i64>,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        TicketInstance::release_tickets(&order_item, quantity, conn)?;
        let calculated_quantity = order_item.calculate_quantity(conn)?;

        if calculated_quantity == 0 {
            order_item.destroy(conn)
        } else {
            let ticket_pricing = TicketPricing::find(order_item.ticket_pricing_id.unwrap(), conn)?;
            order_item.quantity = calculated_quantity;
            order_item.unit_price_in_cents = ticket_pricing.price_in_cents;
            order_item.update(conn)?;

            order_item.update_fees(conn)
        }
    }

    pub fn items(&self, conn: &PgConnection) -> Result<Vec<OrderItem>, DatabaseError> {
        OrderItem::find_for_order(self.id, conn)
    }

    pub fn items_for_display(
        &self,
        conn: &PgConnection,
    ) -> Result<Vec<DisplayOrderItem>, DatabaseError> {
        OrderItem::find_for_display(self.id, conn)
    }

    pub fn find_item(
        &self,
        cart_item_id: Uuid,
        conn: &PgConnection,
    ) -> Result<OrderItem, DatabaseError> {
        OrderItem::find_in_order(self.id, cart_item_id, conn)
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

    pub fn calculate_total(&self, conn: &PgConnection) -> Result<i64, DatabaseError> {
        let order_items = self.items(conn)?;
        let mut total = 0;

        for item in &order_items {
            total += item.unit_price_in_cents * item.quantity;
        }

        Ok(total)
    }
}
