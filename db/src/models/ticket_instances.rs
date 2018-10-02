use chrono::prelude::*;
use diesel;
use diesel::prelude::*;
use diesel::sql_types;
use diesel::sql_types::{Bigint, Nullable, Text, Uuid as dUuid};
use itertools::Itertools;
use models::*;
use schema::{assets, events, order_items, orders, ticket_instances, ticket_types};
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Debug, Identifiable, PartialEq, Deserialize, Serialize, Queryable, QueryableByName)]
#[table_name = "ticket_instances"]
pub struct TicketInstance {
    pub id: Uuid,
    asset_id: Uuid,
    token_id: i32,
    ticket_holding_id: Option<Uuid>,
    pub order_item_id: Option<Uuid>,
    pub reserved_until: Option<NaiveDateTime>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl TicketInstance {
    pub fn find(
        id: Uuid,
        conn: &PgConnection,
    ) -> Result<(DisplayEvent, DisplayUser, DisplayTicket), DatabaseError> {
        let ticket_intermediary = ticket_instances::table
            .inner_join(assets::table.on(ticket_instances::asset_id.eq(assets::id)))
            .inner_join(ticket_types::table.on(assets::ticket_type_id.eq(ticket_types::id)))
            .inner_join(
                order_items::table
                    .on(ticket_instances::order_item_id.eq(order_items::id.nullable())),
            ).inner_join(orders::table.on(order_items::order_id.eq(orders::id)))
            .inner_join(events::table.on(ticket_types::event_id.eq(events::id)))
            .filter(ticket_instances::id.eq(id))
            .select((
                ticket_instances::id,
                ticket_types::name,
                orders::user_id,
                events::id,
                events::venue_id,
            )).first::<DisplayTicketIntermediary>(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load ticket")?;
        let event = Event::find(ticket_intermediary.event_id, conn)?.for_display(conn)?;
        let user: DisplayUser = User::find(ticket_intermediary.user_id, conn)?.into();
        Ok((event, user, ticket_intermediary.into()))
    }

    pub fn find_for_user(
        user_id: Uuid,
        event_id: Option<Uuid>,
        start_time: Option<NaiveDateTime>,
        end_time: Option<NaiveDateTime>,
        conn: &PgConnection,
    ) -> Result<Vec<(DisplayEvent, Vec<DisplayTicket>)>, DatabaseError> {
        let mut query =
            ticket_instances::table
                .inner_join(assets::table.on(ticket_instances::asset_id.eq(assets::id)))
                .inner_join(ticket_types::table.on(assets::ticket_type_id.eq(ticket_types::id)))
                .inner_join(
                    order_items::table
                        .on(ticket_instances::order_item_id.eq(order_items::id.nullable())),
                ).inner_join(orders::table.on(order_items::order_id.eq(orders::id)))
                .inner_join(events::table.on(ticket_types::event_id.eq(events::id)))
                .filter(events::event_start.gt(
                    start_time.unwrap_or_else(|| NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 0)),
                )).filter(events::event_start.lt(
                    end_time.unwrap_or_else(|| NaiveDate::from_ymd(3970, 1, 1).and_hms(0, 0, 0)),
                )).filter(
                    orders::user_id
                        .eq(user_id)
                        .and(orders::status.eq(OrderStatus::Paid.to_string())),
                ).into_boxed();

        if let Some(event_id) = event_id {
            query = query.filter(events::id.eq(event_id));
        }

        let tickets = query
            .select((
                ticket_instances::id,
                ticket_types::name,
                orders::user_id,
                events::id,
                events::venue_id,
            )).order_by(events::event_start.asc())
            .then_order_by(events::name.asc())
            .load::<DisplayTicketIntermediary>(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load user tickets")?;

        let mut grouped_display_tickets = Vec::new();
        for (key, group) in &tickets.into_iter().group_by(|ticket| ticket.event_id) {
            let event = Event::find(key, conn)?.for_display(conn)?;
            let display_tickets: Vec<DisplayTicket> =
                group.into_iter().map(|ticket| ticket.into()).collect();
            grouped_display_tickets.push((event, display_tickets));
        }

        Ok(grouped_display_tickets)
    }

    pub fn create_multiple(
        asset_id: Uuid,
        quantity: u32,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        let mut new_rows = Vec::<NewTicketInstance>::new();
        for x in 0..quantity {
            new_rows.push(NewTicketInstance {
                asset_id,
                token_id: x as i32,
            });
        }

        diesel::insert_into(ticket_instances::table)
            .values(new_rows)
            .execute(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create ticket instances")?;

        Ok(())
    }

    pub fn reserve_tickets(
        order_item: &OrderItem,
        order_expires_at: &NaiveDateTime,
        ticket_type_id: Uuid,
        ticket_holding_id: Option<Uuid>,
        quantity: i64,
        conn: &PgConnection,
    ) -> Result<Vec<TicketInstance>, DatabaseError> {
        let query = include_str!("../queries/reserve_tickets.sql");
        let q = diesel::sql_query(query)
            .bind::<sql_types::Uuid, _>(order_item.id)
            .bind::<sql_types::Timestamp, _>(order_expires_at)
            .bind::<sql_types::Uuid, _>(ticket_type_id)
            .bind::<sql_types::Nullable<sql_types::Uuid>, _>(ticket_holding_id)
            .bind::<Bigint, _>(quantity);
        let ids: Vec<TicketInstance> = q
            .get_results(conn)
            .to_db_error(ErrorCode::QueryError, "Could not reserve tickets")?;

        if ids.len() as i64 != quantity {
            return Err(DatabaseError::new(
                ErrorCode::QueryError,
                Some("Could not reserve the correct amount of tickets"),
            ));
        }

        Ok(ids)
    }

    pub fn release_tickets(
        order_item: &OrderItem,
        quantity: Option<i64>,
        conn: &PgConnection,
    ) -> Result<Vec<TicketInstance>, DatabaseError> {
        let query = include_str!("../queries/release_tickets.sql");
        let q = diesel::sql_query(query)
            .bind::<sql_types::Uuid, _>(order_item.id)
            .bind::<sql_types::Nullable<Bigint>, _>(quantity);
        let ids: Vec<TicketInstance> = q
            .get_results(conn)
            .to_db_error(ErrorCode::QueryError, "Could not release tickets")?;

        // Quantity was specified so number removed should equal amount returned
        if let Some(quantity) = quantity {
            if ids.len() as i64 != quantity {
                return Err(DatabaseError::new(
                    ErrorCode::QueryError,
                    Some("Could not release the correct amount of tickets"),
                ));
            }
        }

        Ok(ids)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DisplayTicket {
    pub id: Uuid,
    pub ticket_type_name: String,
}

#[derive(Queryable, QueryableByName)]
pub struct DisplayTicketIntermediary {
    #[sql_type = "dUuid"]
    pub id: Uuid,
    #[sql_type = "Text"]
    pub name: String,
    #[sql_type = "dUuid"]
    pub user_id: Uuid,
    #[sql_type = "dUuid"]
    pub event_id: Uuid,
    #[sql_type = "Nullable<dUuid>"]
    pub venue_id: Option<Uuid>,
}

impl From<DisplayTicketIntermediary> for DisplayTicket {
    fn from(ticket_intermediary: DisplayTicketIntermediary) -> Self {
        DisplayTicket {
            id: ticket_intermediary.id.clone(),
            ticket_type_name: ticket_intermediary.name.clone(),
        }
    }
}

#[derive(Insertable)]
#[table_name = "ticket_instances"]
struct NewTicketInstance {
    asset_id: Uuid,
    token_id: i32,
}
