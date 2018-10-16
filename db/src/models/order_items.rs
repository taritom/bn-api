use chrono::prelude::*;
use diesel;
use diesel::dsl::{self, select};
use diesel::prelude::*;
use diesel::sql_types;
use diesel::sql_types::{BigInt, Nullable, Text, Uuid as dUuid};
use models::*;
use schema::{order_items, ticket_instances};
use utils::errors;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use uuid::Uuid;
use validator::*;
use validators;

sql_function!(fn order_items_quantity_in_increments(item_type: Text, quantity: BigInt, ticket_pricing_id: Nullable<dUuid>) -> Bool);

#[derive(Identifiable, Associations, Queryable, AsChangeset)]
#[belongs_to(Order)]
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub item_type: String,
    pub event_id: Option<Uuid>,
    pub quantity: i64,
    pub unit_price_in_cents: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub ticket_pricing_id: Option<Uuid>,
    pub fee_schedule_range_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
}

impl OrderItem {
    pub(crate) fn find(id: Uuid, conn: &PgConnection) -> Result<OrderItem, DatabaseError> {
        order_items::table
            .filter(order_items::id.eq(id))
            .first(conn)
            .to_db_error(errors::ErrorCode::QueryError, "Could not find order item")
    }

    pub fn item_type(&self) -> OrderItemTypes {
        self.item_type.parse::<OrderItemTypes>().unwrap()
    }

    pub fn find_fee_item(&self, conn: &PgConnection) -> Result<Option<OrderItem>, DatabaseError> {
        order_items::table
            .filter(order_items::parent_id.eq(self.id))
            .filter(order_items::item_type.eq(OrderItemTypes::PerUnitFees.to_string()))
            .first(conn)
            .optional()
            .to_db_error(
                errors::ErrorCode::QueryError,
                "Could not retrieve order item fees",
            )
    }

    pub(crate) fn update_fees(&self, conn: &PgConnection) -> Result<(), DatabaseError> {
        let fee_item = self.find_fee_item(conn)?;
        let fee_schedule_range = FeeScheduleRange::find(self.fee_schedule_range_id.unwrap(), conn)?;

        match fee_item {
            Some(mut fee_item) => {
                fee_item.unit_price_in_cents = fee_schedule_range.fee_in_cents * self.quantity;
                fee_item.update(conn)
            }
            None => {
                NewFeesOrderItem {
                    order_id: self.order_id,
                    item_type: OrderItemTypes::PerUnitFees.to_string(),
                    event_id: self.event_id,
                    unit_price_in_cents: fee_schedule_range.fee_in_cents * self.quantity,
                    quantity: 1,
                    parent_id: Some(self.id),
                }.commit(conn)?;

                Ok(())
            }
        }
    }

    pub(crate) fn update(&self, conn: &PgConnection) -> Result<(), DatabaseError> {
        self.validate_record(conn)?;
        diesel::update(self)
            .set((
                order_items::unit_price_in_cents.eq(self.unit_price_in_cents),
                order_items::quantity.eq(self.quantity),
                order_items::updated_at.eq(dsl::now),
            )).execute(conn)
            .map(|_| ())
            .to_db_error(
                errors::ErrorCode::UpdateError,
                "Could not update order item",
            )
    }

    pub(crate) fn destroy(self, conn: &PgConnection) -> Result<(), DatabaseError> {
        diesel::delete(&self).execute(conn).map(|_| ()).to_db_error(
            errors::ErrorCode::DeleteError,
            "Could not delete order item",
        )
    }

    pub fn calculate_quantity(&self, conn: &PgConnection) -> Result<i64, DatabaseError> {
        ticket_instances::table
            .filter(ticket_instances::order_item_id.eq(self.id))
            //.filter(ticket_instances::reserved_until.ge(dsl::now.nullable()))
            .select(dsl::count(ticket_instances::id))
            .first(conn)
            .to_db_error(
                errors::ErrorCode::QueryError,
                "Could calculate order item quantity",
            )
    }

    pub fn validate_record(&self, conn: &PgConnection) -> Result<(), DatabaseError> {
        let quantity_valid_increment = OrderItem::quantity_valid_increment(
            false,
            self.item_type.clone(),
            self.quantity,
            self.ticket_pricing_id,
            conn,
        )?;
        Ok(validators::append_validation_error(
            Ok(()),
            "quantity",
            quantity_valid_increment,
        )?)
    }

    fn quantity_valid_increment(
        new_record: bool,
        item_type: String,
        quantity: i64,
        ticket_pricing_id: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<Result<(), ValidationError>, DatabaseError> {
        if item_type != OrderItemTypes::Tickets.to_string() {
            return Ok(Ok(()));
        }
        let result = select(order_items_quantity_in_increments(
            item_type,
            quantity,
            ticket_pricing_id,
        )).get_result::<bool>(conn)
        .to_db_error(
            if new_record {
                errors::ErrorCode::InsertError
            } else {
                errors::ErrorCode::UpdateError
            },
            "Could not confirm quantity increment valid",
        )?;
        if !result {
            return Ok(Err(ValidationError::new(&"quantity_invalid_increment")));
        }
        Ok(Ok(()))
    }

    pub(crate) fn find_for_display(
        order_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<DisplayOrderItem>, DatabaseError> {
        diesel::sql_query(
            r#"
        SELECT oi.id,
           oi.parent_id,
           tt.id                      AS ticket_type_id,
           tp.id                      AS ticket_pricing_id,
           oi.quantity,
           oi.unit_price_in_cents,
           oi.item_type,
           CASE
             WHEN item_type = 'PerUnitFees' THEN 'Ticket Fees'
             WHEN item_type = 'EventFees' THEN 'Event Fees - ' || e.name
             ELSE e.name || ' - ' || tt.name END AS description
        FROM order_items oi
           LEFT JOIN events e ON event_id = e.id
           LEFT JOIN ticket_pricing tp
           INNER JOIN ticket_types tt
            ON tp.ticket_type_id = tt.id
            ON oi.ticket_pricing_id = tp.id
        WHERE oi.order_id = $1
        ORDER BY oi.item_type DESC
        "#,
        ).bind::<sql_types::Uuid, _>(order_id)
        .load(conn)
        .to_db_error(errors::ErrorCode::QueryError, "Could not load order items")
    }

    pub fn find_for_order(
        order_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<OrderItem>, DatabaseError> {
        order_items::table
            .filter(order_items::order_id.eq(order_id))
            .load(conn)
            .to_db_error(errors::ErrorCode::QueryError, "Could not load order items")
    }

    pub(crate) fn find_in_order(
        order_id: Uuid,
        order_item_id: Uuid,
        conn: &PgConnection,
    ) -> Result<OrderItem, errors::DatabaseError> {
        order_items::table
            .filter(order_items::order_id.eq(order_id))
            .filter(order_items::id.eq(order_item_id))
            .filter(order_items::item_type.eq(OrderItemTypes::Tickets.to_string()))
            .first(conn)
            .to_db_error(
                errors::ErrorCode::QueryError,
                "Could not retrieve order item",
            )
    }

    pub(crate) fn find_for_ticket_pricing(
        order_id: Uuid,
        ticket_pricing_id: Uuid,
        conn: &PgConnection,
    ) -> Result<OrderItem, DatabaseError> {
        order_items::table
            .filter(order_items::order_id.eq(order_id))
            .filter(order_items::ticket_pricing_id.eq(ticket_pricing_id))
            .filter(order_items::item_type.eq(OrderItemTypes::Tickets.to_string()))
            .first(conn)
            .to_db_error(
                errors::ErrorCode::QueryError,
                "Could not retrieve order item",
            )
    }
}

#[derive(Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "order_items"]
pub(crate) struct NewTicketsOrderItem {
    pub order_id: Uuid,
    pub item_type: String,
    pub event_id: Option<Uuid>,
    pub quantity: i64,
    pub unit_price_in_cents: i64,
    pub ticket_pricing_id: Uuid,
    pub fee_schedule_range_id: Uuid,
}

impl NewTicketsOrderItem {
    pub(crate) fn commit(self, conn: &PgConnection) -> Result<OrderItem, DatabaseError> {
        self.validate_record(conn)?;
        diesel::insert_into(order_items::table)
            .values(self)
            .get_result(conn)
            .to_db_error(
                errors::ErrorCode::InsertError,
                "Could not create order item",
            )
    }

    pub fn validate_record(&self, conn: &PgConnection) -> Result<(), DatabaseError> {
        let quantity_valid_increment = OrderItem::quantity_valid_increment(
            true,
            self.item_type.clone(),
            self.quantity,
            Some(self.ticket_pricing_id),
            conn,
        )?;
        Ok(validators::append_validation_error(
            Ok(()),
            "quantity",
            quantity_valid_increment,
        )?)
    }
}

#[derive(Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "order_items"]
pub(crate) struct NewFeesOrderItem {
    pub order_id: Uuid,
    pub item_type: String,
    pub event_id: Option<Uuid>,
    pub quantity: i64,
    pub unit_price_in_cents: i64,
    pub parent_id: Option<Uuid>,
}

impl NewFeesOrderItem {
    pub(crate) fn commit(self, conn: &PgConnection) -> Result<OrderItem, DatabaseError> {
        diesel::insert_into(order_items::table)
            .values(self)
            .get_result(conn)
            .to_db_error(
                errors::ErrorCode::InsertError,
                "Could not create order item",
            )
    }
}

#[derive(Deserialize, Queryable, QueryableByName, Serialize)]
pub struct DisplayOrderItem {
    #[sql_type = "dUuid"]
    pub id: Uuid,
    #[sql_type = "Nullable<dUuid>"]
    pub parent_id: Option<Uuid>,
    #[sql_type = "Nullable<dUuid>"]
    pub ticket_type_id: Option<Uuid>,
    #[sql_type = "Nullable<dUuid>"]
    pub ticket_pricing_id: Option<Uuid>,
    #[sql_type = "BigInt"]
    pub quantity: i64,
    #[sql_type = "BigInt"]
    pub unit_price_in_cents: i64,
    #[sql_type = "Text"]
    pub item_type: String,
    #[sql_type = "Text"]
    pub description: String,
}
