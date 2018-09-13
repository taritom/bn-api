use chrono::prelude::*;
use diesel;
use diesel::prelude::*;
use diesel::sql_types;
use diesel::sql_types::Bigint;
use models::orders::Order;
use schema::ticket_instances;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Queryable, QueryableByName)]
#[table_name = "ticket_instances"]
pub struct TicketInstance {
    id: Uuid,
    asset_id: Uuid,
    token_id: i32,
    ticket_holding_id: Option<Uuid>,
    order_id: Option<Uuid>,
    reserved_until: Option<NaiveDateTime>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl TicketInstance {
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
        order: &Order,
        ticket_type_id: Uuid,
        ticket_holding_id: Option<Uuid>,
        quantity: i64,
        conn: &PgConnection,
    ) -> Result<Vec<TicketInstance>, DatabaseError> {
        let query = include_str!("../queries/reserve_tickets.sql");
        let q = diesel::sql_query(query)
            .bind::<sql_types::Uuid, _>(order.id)
            .bind::<sql_types::Timestamp, _>(order.expires_at)
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
}

#[derive(Insertable)]
#[table_name = "ticket_instances"]
struct NewTicketInstance {
    asset_id: Uuid,
    token_id: i32,
}
