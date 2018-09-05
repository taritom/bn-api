use chrono::NaiveDateTime;
use chrono::Utc;
use db::Connectable;
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use models::Event;
use schema::ticket_allocations;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Identifiable, Associations, Queryable)]
#[belongs_to(Event, foreign_key = "event_id")]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "ticket_allocations"]
pub struct TicketAllocation {
    pub id: Uuid,
    pub event_id: Uuid,
    tari_asset_id: Option<String>,
    pub created_at: NaiveDateTime,
    synced_at: Option<NaiveDateTime>,
    ticket_delta: i64,
    updated_at: NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "ticket_allocations"]
pub struct NewTicketAllocation {
    pub event_id: Uuid,
    pub ticket_delta: i64,
}

#[derive(AsChangeset, Default, Deserialize)]
#[table_name = "ticket_allocations"]
pub struct TicketAllocationEditableAttributes {
    tari_asset_id: Option<String>,
    synced_at: Option<NaiveDateTime>,
}

impl NewTicketAllocation {
    pub fn commit(&self, conn: &Connectable) -> Result<TicketAllocation, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new ticket allocation",
            diesel::insert_into(ticket_allocations::table)
                .values(self)
                .get_result(conn.get_connection()),
        )
    }
}

impl TicketAllocation {
    pub fn create(event_id: Uuid, ticket_delta: i64) -> NewTicketAllocation {
        NewTicketAllocation {
            event_id,
            ticket_delta,
        }
    }

    pub fn set_asset_id(&mut self, asset_id: String) {
        self.tari_asset_id = Some(asset_id);
        self.synced_at = Some(Utc::now().naive_utc())
    }

    pub fn update(self, conn: &Connectable) -> Result<TicketAllocation, DatabaseError> {
        let update_attr = TicketAllocationEditableAttributes {
            synced_at: self.synced_at,
            tari_asset_id: self.tari_asset_id.clone(),
        };

        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update organization",
            diesel::update(&self)
                .set((update_attr, ticket_allocations::updated_at.eq(dsl::now)))
                .get_result(conn.get_connection()),
        )
    }

    pub fn tari_asset_id(&self) -> Option<&String> {
        self.tari_asset_id.as_ref()
    }

    pub fn ticket_delta(&self) -> i64 {
        self.ticket_delta
    }

    pub fn find_by_event_id(
        event_id: Uuid,
        conn: &Connectable,
    ) -> Result<Vec<TicketAllocation>, DatabaseError> {
        ticket_allocations::table
            .filter(ticket_allocations::event_id.eq(event_id))
            .load(conn.get_connection())
            .to_db_error(
                ErrorCode::QueryError,
                "Could not find ticket allocations for event",
            )
    }
}
