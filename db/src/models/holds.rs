use chrono::prelude::*;
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use models::*;
use schema::holds;
use utils::errors::*;
use uuid::Uuid;

#[derive(Serialize, Queryable, Identifiable)]
pub struct Hold {
    pub id: Uuid,
    pub name: String,
    pub event_id: Uuid,
    pub redemption_code: String,
    pub discount_in_cents: i64,
    pub end_at: Option<NaiveDateTime>,
    pub max_per_order: Option<i64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Deserialize, AsChangeset)]
#[table_name = "holds"]
pub struct UpdateHoldAttributes {
    pub name: Option<String>,
    pub discount_in_cents: Option<i64>,
    pub end_at: Option<Option<NaiveDateTime>>,
    pub max_per_order: Option<Option<i64>>,
}

impl Hold {
    pub fn create(
        name: String,
        event_id: Uuid,
        redemption_code: String,
        discount_in_cents: u32,
        end_at: Option<NaiveDateTime>,
        max_per_order: Option<u32>,
    ) -> NewHold {
        NewHold {
            name,
            event_id,
            redemption_code,
            discount_in_cents: discount_in_cents as i64,
            end_at,
            max_per_order: max_per_order.map(|m| m as i64),
        }
    }

    pub fn update(
        &self,
        update_attrs: UpdateHoldAttributes,
        conn: &PgConnection,
    ) -> Result<Hold, DatabaseError> {
        diesel::update(
            holds::table
                .filter(holds::id.eq(self.id))
                .filter(holds::updated_at.eq(self.updated_at)),
        ).set((update_attrs, holds::updated_at.eq(dsl::now)))
        .get_result(conn)
        .to_db_error(ErrorCode::UpdateError, "Could not update hold")
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<Hold, DatabaseError> {
        holds::table
            .filter(holds::id.eq(id))
            .first(conn)
            .to_db_error(ErrorCode::QueryError, "Could not retrieve hold")
    }

    pub fn set_quantity(
        &self,
        ticket_type_id: Uuid,
        quantity: u32,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        let count = self.quantity(ticket_type_id, conn)?;
        if count < quantity {
            TicketInstance::add_to_hold(self.id, ticket_type_id, quantity - count, conn)?;
        }
        if count > quantity {
            TicketInstance::release_from_hold(self.id, ticket_type_id, count - quantity, conn)?;
        }
        Ok(())
    }

    pub fn quantity(
        &self,
        ticket_type_id: Uuid,
        conn: &PgConnection,
    ) -> Result<u32, DatabaseError> {
        TicketInstance::count_for_hold(self.id, ticket_type_id, conn)
    }

    pub fn organization(&self, conn: &PgConnection) -> Result<Organization, DatabaseError> {
        use schema::*;
        events::table
            .inner_join(organizations::table)
            .filter(events::id.eq(self.event_id))
            .select(organizations::all_columns)
            .first(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not load organization for hold",
            )
    }
}

#[derive(Insertable)]
#[table_name = "holds"]
pub struct NewHold {
    pub name: String,
    pub event_id: Uuid,
    pub redemption_code: String,
    pub discount_in_cents: i64,
    pub end_at: Option<NaiveDateTime>,
    pub max_per_order: Option<i64>,
}

impl NewHold {
    pub fn commit(self, conn: &PgConnection) -> Result<Hold, DatabaseError> {
        diesel::insert_into(holds::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create hold")
    }
}
