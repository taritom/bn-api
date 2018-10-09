use chrono::prelude::*;
use diesel;
use diesel::prelude::*;
use models::enums::*;
use schema::*;
use serde_json;
use utils::errors::*;
use uuid::Uuid;

#[derive(Identifiable, Queryable)]
pub struct DomainEvent {
    pub id: Uuid,
    pub event_type: String,
    pub display_text: String,
    pub event_data: Option<serde_json::Value>,
    pub main_table: String,
    pub main_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl DomainEvent {
    pub fn create(
        event_type: DomainEventTypes,
        display_text: String,
        main_table: Tables,
        main_id: Option<Uuid>,
        event_data: Option<serde_json::Value>,
    ) -> NewDomainEvent {
        NewDomainEvent {
            event_type: event_type.to_string(),
            display_text,
            event_data,
            main_table: main_table.to_string(),
            main_id,
        }
    }
}

#[derive(Insertable)]
#[table_name = "domain_events"]
pub struct NewDomainEvent {
    pub event_type: String,
    pub display_text: String,
    pub event_data: Option<serde_json::Value>,
    pub main_table: String,
    pub main_id: Option<Uuid>,
}

impl NewDomainEvent {
    pub fn commit(self, conn: &PgConnection) -> Result<DomainEvent, DatabaseError> {
        diesel::insert_into(domain_events::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not insert domain event")
    }
}
