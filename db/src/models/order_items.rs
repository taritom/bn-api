use chrono::prelude::*;
use diesel;
use diesel::dsl;
use diesel::prelude::*;
use diesel::sql_types;
use diesel::sql_types::{BigInt, Nullable, Text, Uuid as dUuid};
use models::*;
use schema::{order_items, ticket_instances};
use utils::errors;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use uuid::Uuid;

#[derive(Identifiable, Associations, Queryable, AsChangeset)]
#[belongs_to(Order)]
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub item_type: String,
    pub quantity: i64,
    pub unit_price_in_cents: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub ticket_pricing_id: Option<Uuid>,
    pub fee_schedule_range_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
}

impl OrderItem {
    pub(crate) fn create_tickets(
        _order_id: Uuid,
        _ticket_type_id: Uuid,
        _quantity: u32,
    ) -> NewTicketsOrderItem {
        unimplemented!()

        //        NewTicketsOrderItem {
        //            order_id,
        //            ticket_type_id,
        //            item_type: OrderItemTypes::Tickets.to_string(),
        //            quantity: quantity as i64,
        //        }
    }

    pub fn item_type(&self) -> OrderItemTypes {
        self.item_type.parse::<OrderItemTypes>().unwrap()
    }

    pub fn find_fee_item(&self, conn: &PgConnection) -> Result<Option<OrderItem>, DatabaseError> {
        order_items::table
            .filter(order_items::parent_id.eq(self.id))
            .filter(order_items::item_type.eq(OrderItemTypes::Fees.to_string()))
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
                    item_type: OrderItemTypes::Fees.to_string(),
                    unit_price_in_cents: fee_schedule_range.fee_in_cents * self.quantity,
                    quantity: 1,
                    parent_id: self.id,
                }.commit(conn)?;

                Ok(())
            }
        }
    }

    pub(crate) fn update(&self, conn: &PgConnection) -> Result<(), DatabaseError> {
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
             WHEN item_type = 'Fees' THEN 'Fees'
             ELSE e.name || ' - ' || tt.name END AS description
        FROM order_items oi
           LEFT JOIN ticket_pricing tp
           INNER JOIN ticket_types tt
           INNER JOIN events e ON tt.event_id = e.id
            ON tp.ticket_type_id = tt.id
            ON oi.ticket_pricing_id = tp.id
        WHERE oi.order_id = $1
        "#,
        ).bind::<sql_types::Uuid, _>(order_id)
        .load(conn)
        .to_db_error(errors::ErrorCode::QueryError, "Could not load order items")
    }

    pub(crate) fn find_for_order(
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
}

#[derive(Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "order_items"]
pub(crate) struct NewTicketsOrderItem {
    pub order_id: Uuid,
    pub item_type: String,
    pub quantity: i64,
    pub unit_price_in_cents: i64,
    pub ticket_pricing_id: Uuid,
    pub fee_schedule_range_id: Uuid,
}

impl NewTicketsOrderItem {
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

#[derive(Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "order_items"]
pub(crate) struct NewFeesOrderItem {
    pub order_id: Uuid,
    pub item_type: String,
    pub quantity: i64,
    pub unit_price_in_cents: i64,
    pub parent_id: Uuid,
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

#[derive(Queryable, QueryableByName, Serialize)]
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
