use chrono::prelude::*;
use diesel;
use diesel::dsl::{exists, select};
use diesel::expression::dsl;
use diesel::prelude::*;
use diesel::sql_types;
use diesel::sql_types::{BigInt, Integer, Nullable, Uuid as dUuid};
use itertools::Itertools;
use log::Level;
use models::*;
use schema::{order_items, orders, users};
use serde_json;
use std::borrow::Cow;
use std::collections::HashMap;
use time::Duration;
use utils::errors;
use utils::errors::*;
use uuid::Uuid;
use validator::ValidationErrors;
use validators::*;

#[derive(Associations, Debug, Identifiable, PartialEq, Queryable)]
#[belongs_to(User)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    order_type: String,
    order_date: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    pub version: i64,
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
    pub fn destroy(&self, conn: &PgConnection) -> Result<usize, DatabaseError> {
        let cart_user: Option<User> = users::table
            .filter(users::last_cart_id.eq(self.id))
            .get_result(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not find user attached to this cart",
            ).optional()?;
        if cart_user.is_some() {
            cart_user.unwrap().update_last_cart(None, conn)?;
        }

        DatabaseError::wrap(
            ErrorCode::DeleteError,
            "Failed to delete order record",
            diesel::delete(self).execute(conn),
        )
    }

    pub fn status(&self) -> OrderStatus {
        self.status.parse::<OrderStatus>().unwrap()
    }

    pub fn find_or_create_cart(user: &User, conn: &PgConnection) -> Result<Order, DatabaseError> {
        // Do a quick check to find the cart linked to the user.
        let cart = Order::find_cart_for_user(user.id, conn)?;

        if cart.is_some() {
            return Ok(cart.unwrap());
        }

        // Cart either does not exist, expired or was paid up.
        // A number of threads might reach here at the same time, so we
        // need to do a bit of concurrency checking.

        let query = r#"
            INSERT INTO Orders (user_id, status, expires_at, order_type)
            SELECT $1 as user_id, 'Draft' as status, $2 as expires_at, 'Cart' as order_type
            WHERE NOT EXISTS
            ( SELECT o.id FROM orders o
                WHERE o.user_id = $1
                AND o.status = 'Draft'
                AND o.order_type = 'Cart'
                AND o.expires_at > now())
            RETURNING id;
        "#;

        #[derive(QueryableByName)]
        struct R {
            #[sql_type = "Nullable<dUuid>"]
            id: Option<Uuid>,
        }

        let cart_id: Vec<R> = diesel::sql_query(query)
            .bind::<sql_types::Uuid, _>(user.id)
            .bind::<sql_types::Timestamp, _>(Utc::now().naive_utc() + Duration::minutes(15))
            .get_results(conn)
            .to_db_error(ErrorCode::QueryError, "Could not find or create cart")?;

        if cart_id.is_empty() || cart_id[0].id.is_none() || cart_id.len() > 1 {
            // Another thread has created a cart
            return DatabaseError::concurrency_error(&format!(
                "Possible race condition when creating a cart for a user. Number of carts returned: {}",
                cart_id.len()
            ));
        }

        let cart_id = cart_id[0].id;

        // This will also row lock the user row to detect that another thread has not
        // created another cart in the mean time
        user.update_last_cart(cart_id, conn)?;

        // Finally return the actual order
        Order::find(cart_id.unwrap(), conn)
    }

    pub fn find_cart_for_user(
        user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Option<Order>, DatabaseError> {
        users::table
            .inner_join(orders::table.on(users::last_cart_id.eq(orders::id.nullable())))
            .filter(users::id.eq(user_id))
            .filter(orders::user_id.eq(user_id))
            .filter(orders::status.eq("Draft"))
            .filter(orders::order_type.eq("Cart"))
            .filter(orders::expires_at.ge(dsl::now))
            .select(orders::all_columns)
            .first(conn)
            .to_db_error(
                errors::ErrorCode::QueryError,
                "Could not load cart for user",
            ).optional()
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<Order, DatabaseError> {
        orders::table
            .filter(orders::id.eq(id))
            .first(conn)
            .to_db_error(errors::ErrorCode::QueryError, "Could not find order")
    }

    pub fn update_quantities(
        &mut self,
        items: &[UpdateOrderItem],
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        self.lock_version(conn)?;

        #[derive(Debug)]
        struct LimitCheck {
            ticket_type_id: Uuid,
            event_id: Uuid,
            limit_per_person: i32,
        }
        let mut check_ticket_limits: Vec<LimitCheck> = vec![];

        let mut mapped = vec![];
        for (index, item) in items.iter().enumerate() {
            mapped.push(match &item.redemption_code {
                Some(r) => {
                    let hold = Hold::find_by_redemption_code(r, conn)
                        .optional()?
                        .map(|h| h.id);
                    let comp = Comp::find_by_redemption_code(r, conn)
                        .optional()?
                        .map(|c| c.id);
                    (index, hold, comp, item)
                }
                None => (index, None, None, item),
            });
        }

        for mut current_line in self
            .items(conn)?
            .into_iter()
            .filter(|t| t.item_type() == OrderItemTypes::Tickets)
        {
            let mut index_to_remove: Option<usize> = None;
            {
                let matching_result: Vec<&(
                    usize,
                    Option<Uuid>,
                    Option<Uuid>,
                    &UpdateOrderItem,
                )> = mapped
                    .iter()
                    .filter(|i| {
                        Some(i.3.ticket_type_id) == current_line.ticket_type_id
                            && i.1 == current_line.hold_id
                            && i.2 == current_line.comp_id
                    }).collect();
                let matching_result = matching_result.first();

                if matching_result.is_some() {
                    jlog!(Level::Debug, "Found an existing cart item, replacing");
                    let (index, hold_id, comp_id, mut matching_line) = matching_result.unwrap();
                    index_to_remove = Some(*index);
                    if current_line.quantity as u32 > matching_line.quantity {
                        jlog!(Level::Debug, "Reducing quantity of cart item");
                        TicketInstance::release_tickets(
                            &current_line,
                            current_line.quantity as u32 - matching_line.quantity,
                            conn,
                        )?;
                        current_line.quantity = matching_line.quantity as i64;
                        current_line.update(conn)?;
                        if current_line.quantity == 0 {
                            jlog!(Level::Debug, "Cart item has 0 quantity, deleting it");
                            OrderItem::destroy(current_line.id, conn)?;
                        }
                    } else if (current_line.quantity as u32) < matching_line.quantity {
                        jlog!(Level::Debug, "Increasing quantity of cart item");
                        // Ticket pricing might have changed since we added the previous item.
                        // In future we may want to use the ticket pricing at the time the order was created.

                        // TODO: Fetch the ticket type and pricing in one go.
                        let ticket_type_id = current_line.ticket_type_id.unwrap();
                        let ticket_pricing =
                            TicketPricing::get_current_ticket_pricing(ticket_type_id, conn)?;
                        let ticket_type = TicketType::find(ticket_type_id, conn)?;

                        check_ticket_limits.push(LimitCheck {
                            limit_per_person: ticket_type.limit_per_person.clone(),
                            ticket_type_id: ticket_type.id.clone(),
                            event_id: ticket_type.event_id.clone(),
                        });

                        // TODO: Move this to an external processer
                        let fee_schedule_range = ticket_type
                            .fee_schedule(conn)?
                            .get_range(ticket_pricing.price_in_cents, conn)?;
                        if Some(ticket_pricing.id) != current_line.ticket_pricing_id {
                            let order_item = NewTicketsOrderItem {
                                order_id: self.id,
                                item_type: OrderItemTypes::Tickets.to_string(),
                                quantity: matching_line.quantity as i64,
                                ticket_type_id: ticket_type.id,
                                ticket_pricing_id: ticket_pricing.id,
                                event_id: Some(ticket_type.event_id),
                                fee_schedule_range_id: fee_schedule_range.id,
                                unit_price_in_cents: ticket_pricing.price_in_cents,
                                comp_id: *comp_id,
                                hold_id: *hold_id,
                                code_id: None,
                            }.commit(conn)?;
                            TicketInstance::reserve_tickets(
                                &order_item,
                                self.expires_at,
                                ticket_type_id,
                                *hold_id,
                                matching_line.quantity - current_line.quantity as u32,
                                conn,
                            )?;
                        } else {
                            TicketInstance::reserve_tickets(
                                &current_line,
                                self.expires_at,
                                ticket_type_id,
                                *hold_id,
                                matching_line.quantity - current_line.quantity as u32,
                                conn,
                            )?;
                            current_line.quantity = matching_line.quantity as i64;
                            current_line.update(conn)?;
                        }
                    }
                }
            }
            if index_to_remove.is_some() {
                mapped.remove(index_to_remove.unwrap());
            }
        }

        for (_, hold_id, comp_id, new_line) in mapped {
            jlog!(Level::Debug, "Adding new cart items");
            let ticket_pricing =
                TicketPricing::get_current_ticket_pricing(new_line.ticket_type_id, conn)?;
            let ticket_type = TicketType::find(new_line.ticket_type_id, conn)?;

            check_ticket_limits.push(LimitCheck {
                limit_per_person: ticket_type.limit_per_person.clone(),
                ticket_type_id: ticket_type.id.clone(),
                event_id: ticket_type.event_id.clone(),
            });

            // TODO: Move this to an external processer
            let fee_schedule_range = ticket_type
                .fee_schedule(conn)?
                .get_range(ticket_pricing.price_in_cents, conn)?;
            let order_item = NewTicketsOrderItem {
                order_id: self.id,
                item_type: OrderItemTypes::Tickets.to_string(),
                quantity: new_line.quantity as i64,
                ticket_type_id: ticket_type.id,
                ticket_pricing_id: ticket_pricing.id,
                event_id: Some(ticket_type.event_id),
                fee_schedule_range_id: fee_schedule_range.id,
                unit_price_in_cents: ticket_pricing.price_in_cents,
                comp_id,
                hold_id,
                code_id: None,
            }.commit(conn)?;

            TicketInstance::reserve_tickets(
                &order_item,
                self.expires_at,
                new_line.ticket_type_id,
                hold_id,
                new_line.quantity,
                conn,
            )?;
        }

        for limit_check in check_ticket_limits {
            let quantities_ordered =
                Order::quantity_for_user_for_event(&self.user_id, &limit_check.event_id, &conn)?;

            if &limit_check.limit_per_person > &0
                && quantities_ordered.contains_key(&limit_check.ticket_type_id)
            {
                match quantities_ordered.get(&limit_check.ticket_type_id) {
                    Some(ordered_quantity) => {
                        if ordered_quantity > &limit_check.limit_per_person {
                            let mut error = create_validation_error(
                                "limit_per_person_exceeded",
                                "Exceeded limit per person per event",
                            );
                            error.add_param(
                                Cow::from("limit_per_person"),
                                &limit_check.limit_per_person,
                            );
                            error.add_param(
                                Cow::from("ticket_type_id"),
                                &limit_check.ticket_type_id,
                            );
                            error.add_param(Cow::from("attempted_quantity"), ordered_quantity);
                            let mut errors = ValidationErrors::new();
                            errors.add("quantity", error);
                            return Err(errors.into());
                        }
                    }
                    None => {}
                };
            }
        }

        self.update_fees(conn)?;

        Ok(())
    }

    pub fn has_items(&self, conn: &PgConnection) -> Result<bool, DatabaseError> {
        select(exists(
            order_items::table.filter(order_items::order_id.eq(self.id)),
        )).get_result(conn)
        .to_db_error(
            errors::ErrorCode::QueryError,
            "Could not check if order items exist",
        )
    }

    pub fn update_fees(&self, conn: &PgConnection) -> Result<(), DatabaseError> {
        let items = self.items(conn)?;

        for o in items {
            match o.item_type() {
                OrderItemTypes::EventFees => OrderItem::destroy(o.id, conn)?,
                _ => {}
            }
        }
        for (event_id, items) in self
            .items(conn)?
            .iter()
            .group_by(|i| i.event_id)
            .into_iter()
        {
            if event_id.is_none() {
                continue;
            }
            let event = Event::find(event_id.unwrap(), conn)?;
            let organization = Organization::find(event.organization_id, conn)?;
            for o in items {
                match o.item_type() {
                    OrderItemTypes::Tickets => o.update_fees(conn)?,
                    _ => {}
                }
            }

            let mut new_event_fee = NewFeesOrderItem {
                order_id: self.id,
                item_type: OrderItemTypes::EventFees.to_string(),
                event_id: Some(event.id),
                unit_price_in_cents: 0,
                quantity: 1,
                parent_id: None,
            };
            if event.fee_in_cents.is_some() {
                new_event_fee.unit_price_in_cents = event.fee_in_cents.unwrap();
                new_event_fee.commit(conn)?;
            } else if organization.event_fee_in_cents.is_some() {
                new_event_fee.unit_price_in_cents = organization.event_fee_in_cents.unwrap();
                new_event_fee.commit(conn)?;
            }
        }

        Ok(())
    }

    pub fn quantity_for_user_for_event(
        user_id: &Uuid,
        event_id: &Uuid,
        conn: &PgConnection,
    ) -> Result<HashMap<Uuid, i32>, DatabaseError> {
        let mut ticket_type_totals: HashMap<Uuid, i32> = HashMap::new();

        let query = include_str!("../queries/quantity_of_tickets_per_user_per_event.sql");
        let order_items_for_user: Vec<ResultForTicketTypeTotal> = diesel::sql_query(query)
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .bind::<diesel::sql_types::Uuid, _>(event_id)
            .load::<ResultForTicketTypeTotal>(conn)
            .to_db_error(ErrorCode::QueryError, "Could not load orders")?;

        for result_for_ticket in &order_items_for_user {
            ticket_type_totals.insert(
                result_for_ticket.ticket_type_id.unwrap(),
                result_for_ticket.total_quantity,
            );
        }

        Ok(ticket_type_totals)
    }

    pub fn find_for_user_for_display(
        user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<DisplayOrder>, DatabaseError> {
        use schema::*;
        let orders: Vec<Order> = orders::table
            .filter(orders::user_id.eq(user_id))
            .filter(orders::status.ne(OrderStatus::Draft.to_string()))
            .order_by(orders::order_date.desc())
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Could not load orders")?;
        let mut r = Vec::<DisplayOrder>::new();
        for order in orders {
            r.push(order.for_display(conn)?);
        }
        Ok(r)
    }

    pub fn items(&self, conn: &PgConnection) -> Result<Vec<OrderItem>, DatabaseError> {
        OrderItem::find_for_order(self.id, conn)
    }

    pub fn tickets(
        &self,
        ticket_type_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<TicketInstance>, DatabaseError> {
        let items = self.items(conn)?;
        let tickets: Vec<OrderItem> = items
            .into_iter()
            .filter(|ci| {
                ci.item_type() == OrderItemTypes::Tickets
                    && ci.ticket_type_id == Some(ticket_type_id)
            }).collect();

        let mut result: Vec<TicketInstance> = vec![];
        for t in tickets {
            let mut instances = TicketInstance::find_for_order_item(t.id, conn)?;
            result.append(&mut instances);
        }

        Ok(result)
    }

    pub fn for_display(&self, conn: &PgConnection) -> Result<DisplayOrder, DatabaseError> {
        let now = Utc::now().naive_utc();
        let seconds_until_expiry = if self.expires_at >= now {
            let duration = self.expires_at.signed_duration_since(now);
            duration.num_seconds() as u32
        } else {
            0
        };

        Ok(DisplayOrder {
            id: self.id,
            status: self.status.clone(),
            date: self.order_date,
            expires_at: self.expires_at,
            items: self.items_for_display(conn)?,
            total_in_cents: self.calculate_total(conn)?,
            seconds_until_expiry,
        })
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
        let payment = Payment::create(
            self.id,
            current_user_id,
            PaymentStatus::Completed,
            PaymentMethods::External,
            "External".to_string(),
            external_reference,
            amount,
            None,
        );
        self.add_payment(payment, conn)
    }

    pub fn add_credit_card_payment(
        &mut self,
        current_user_id: Uuid,
        amount: i64,
        provider: String,
        external_reference: String,
        status: PaymentStatus,
        provider_data: serde_json::Value,
        conn: &PgConnection,
    ) -> Result<Payment, DatabaseError> {
        let payment = Payment::create(
            self.id,
            current_user_id,
            status,
            PaymentMethods::CreditCard,
            provider,
            external_reference,
            amount,
            Some(provider_data),
        );

        self.add_payment(payment, conn)
    }

    fn add_payment(
        &mut self,
        payment: NewPayment,
        conn: &PgConnection,
    ) -> Result<Payment, DatabaseError> {
        if self.status() == OrderStatus::Paid {
            return DatabaseError::business_process_error("This order has already been paid");
        }
        // orders can only expire if the order is in draft
        if self.status() == OrderStatus::Draft {
            self.mark_partially_paid(conn)?;
        } else if self.status() != OrderStatus::PartiallyPaid {
            return DatabaseError::business_process_error(&format!(
                "Order was in unexpected state when trying to make a payment: {}",
                self.status()
            ));
        }

        let p = payment.commit(conn)?;

        self.complete_if_fully_paid(conn)?;
        Ok(p)
    }

    fn mark_partially_paid(&mut self, conn: &PgConnection) -> Result<(), DatabaseError> {
        // TODO: The multiple queries in this method could probably be combined into a single query
        let now_plus_one_day = Utc::now().naive_utc() + Duration::days(1);

        let result = diesel::update(
            orders::table.filter(
                orders::id
                    .eq(self.id)
                    .and(orders::version.eq(self.version))
                    .and(orders::expires_at.gt(dsl::now)),
            ),
        ).set((
            orders::status.eq(OrderStatus::PartiallyPaid.to_string()),
            orders::expires_at.eq(now_plus_one_day),
            orders::updated_at.eq(dsl::now),
        )).execute(conn)
        .to_db_error(ErrorCode::UpdateError, "Could not update order status")?;

        let db_record = Order::find(self.id, conn)?;

        if result == 0 {
            if db_record.version != self.version {
                return DatabaseError::concurrency_error(
                    "Could not update order because it has been updated by another process",
                );
            }

            // Unfortunately, it's quite hard to work out what the current dsl::now() time is
            // So assume that the order has expired.
            return DatabaseError::business_process_error(
                "Could not update order because it has expired",
            );
        }

        self.updated_at = db_record.updated_at;
        self.status = db_record.status;

        //Extend the reserved_until time for tickets associated with this order
        let order_items = OrderItem::find_for_order(db_record.id, conn)?;

        for item in &order_items {
            TicketInstance::update_reserved_time(item, now_plus_one_day, conn)?;
        }

        Ok(())
    }

    pub(crate) fn complete_if_fully_paid(
        &mut self,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        if self.total_paid(conn)? >= self.calculate_total(conn)? {
            self.update_status(OrderStatus::Paid, conn)?;
            //Mark tickets as Purchased
            let order_items = OrderItem::find_for_order(self.id, conn)?;
            for item in &order_items {
                TicketInstance::mark_as_purchased(item, self.user_id, conn)?;
            }
            let cart_user: Option<User> = users::table
                .filter(users::last_cart_id.eq(self.id))
                .get_result(conn)
                .to_db_error(
                    ErrorCode::QueryError,
                    "Could not find user attached to this cart",
                ).optional()?;

            if cart_user.is_some() {
                cart_user.unwrap().update_last_cart(None, conn)?;
            }
        }
        Ok(())
    }

    pub fn total_paid(&self, conn: &PgConnection) -> Result<i64, DatabaseError> {
        #[derive(QueryableByName)]
        struct ResultForSum {
            #[sql_type = "Nullable<BigInt>"]
            s: Option<i64>,
        };
        let query = diesel::sql_query(
            "SELECT CAST(SUM(amount) as BigInt) as s FROM payments WHERE order_id = $1 AND status='Completed';",
        ).bind::<diesel::sql_types::Uuid, _>(self.id);

        let sum: ResultForSum = query.get_result(conn).to_db_error(
            ErrorCode::QueryError,
            "Could not get total payments for order",
        )?;
        Ok(sum.s.unwrap_or(0))
    }

    fn update_status(
        &mut self,
        status: OrderStatus,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        self.status = status.to_string();
        diesel::update(&*self)
            .set((
                orders::status.eq(&self.status),
                orders::updated_at.eq(dsl::now),
            )).execute(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not update order")?;

        Ok(())
    }

    pub fn calculate_total(&self, conn: &PgConnection) -> Result<i64, DatabaseError> {
        let order_items = self.items(conn)?;
        let mut total = 0;

        for item in &order_items {
            total += item.unit_price_in_cents * item.quantity;
        }

        Ok(total)
    }

    /// Updates the lock version in the database and forces a Concurrency error if
    /// another process has updated it
    pub fn lock_version(&mut self, conn: &PgConnection) -> Result<(), DatabaseError> {
        let rows_affected = diesel::update(
            orders::table
                .filter(orders::id.eq(self.id))
                .filter(orders::version.eq(self.version)),
        ).set((
            orders::version.eq(self.version + 1),
            orders::updated_at.eq(dsl::now),
        )).execute(conn)
        .to_db_error(ErrorCode::UpdateError, "Could not lock order")?;
        if rows_affected == 0 {
            return DatabaseError::concurrency_error(
                "Could not lock order, another process has updated it",
            );
        }
        self.version = self.version + 1;
        Ok(())
    }
}

#[derive(QueryableByName, Deserialize, Serialize, Debug)]
pub struct ResultForTicketTypeTotal {
    #[sql_type = "Nullable<dUuid>"]
    ticket_type_id: Option<Uuid>,
    #[sql_type = "Integer"]
    total_quantity: i32,
}

#[derive(Deserialize, Serialize)]
pub struct DisplayOrder {
    pub id: Uuid,
    pub date: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    pub seconds_until_expiry: u32,
    pub status: String,
    pub items: Vec<DisplayOrderItem>,
    pub total_in_cents: i64,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct UpdateOrderItem {
    pub ticket_type_id: Uuid,
    pub quantity: u32,
    pub redemption_code: Option<String>,
}
