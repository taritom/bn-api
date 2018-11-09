use chrono::NaiveDateTime;
use diesel;
use diesel::dsl;
use diesel::prelude::*;
use models::*;
use schema::{
    assets, events, fee_schedules, organizations, ticket_instances, ticket_pricing,
    ticket_type_codes, ticket_types,
};
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;
use validator::*;
use validators;

#[derive(Identifiable, Associations, Queryable, PartialEq, Debug)]
#[table_name = "ticket_types"]
#[belongs_to(Event)]
pub struct TicketType {
    pub id: Uuid,
    pub event_id: Uuid,
    pub name: String,
    status: String,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub increment: i32,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(AsChangeset, Default, Deserialize)]
#[table_name = "ticket_types"]
pub struct TicketTypeEditableAttributes {
    pub name: Option<String>,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
    pub increment: Option<i32>,
}

impl TicketType {
    // Properties at the top

    pub fn fee_schedule(&self, conn: &PgConnection) -> Result<FeeSchedule, DatabaseError> {
        ticket_types::table
            .inner_join(
                events::table.inner_join(organizations::table.inner_join(fee_schedules::table)),
            ).filter(ticket_types::id.eq(self.id))
            .select(fee_schedules::all_columns)
            .first(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not retrieve fee schedule for ticket type",
            )
    }

    pub fn create(
        event_id: Uuid,
        name: String,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
        increment: Option<i32>,
    ) -> NewTicketType {
        NewTicketType {
            event_id,
            name,
            status: TicketTypeStatus::Published.to_string(),
            start_date,
            end_date,
            increment,
        }
    }

    pub fn update(
        &self,
        attributes: TicketTypeEditableAttributes,
        conn: &PgConnection,
    ) -> Result<TicketType, DatabaseError> {
        diesel::update(self)
            .set((attributes, ticket_types::updated_at.eq(dsl::now)))
            .get_result(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not update ticket_types")
    }

    pub fn find_for_code(
        code_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<TicketType>, DatabaseError> {
        ticket_types::table
            .inner_join(
                ticket_type_codes::table.on(ticket_type_codes::ticket_type_id.eq(ticket_types::id)),
            ).filter(ticket_type_codes::code_id.eq(code_id))
            .select(ticket_types::all_columns)
            .get_results(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not find ticket types for code",
            )
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<TicketType, DatabaseError> {
        ticket_types::table
            .filter(ticket_types::id.eq(id))
            .get_result(conn)
            .to_db_error(ErrorCode::QueryError, "Could not find ticket type")
    }

    pub fn validate_record(&self, conn: &PgConnection) -> Result<(), DatabaseError> {
        let mut validation_errors: Result<(), ValidationErrors> = Ok(());
        for ticket_pricing in self.ticket_pricing(conn)? {
            validation_errors = validators::append_validation_error(
                validation_errors,
                "ticket_pricing",
                TicketPricing::ticket_pricing_no_overlapping_periods(
                    ticket_pricing.id,
                    self.id,
                    ticket_pricing.start_date,
                    ticket_pricing.end_date,
                    conn,
                )?,
            );
        }

        Ok(validation_errors?)
    }

    pub fn find_by_event_id(
        event_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<TicketType>, DatabaseError> {
        ticket_types::table
            .filter(ticket_types::event_id.eq(event_id))
            .order_by(ticket_types::name)
            .load(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not load ticket types for event",
            )
    }

    pub fn ticket_count(&self, conn: &PgConnection) -> Result<u32, DatabaseError> {
        let valid_ticket_count: i64 = ticket_instances::table
            .inner_join(assets::table)
            .filter(assets::ticket_type_id.eq(self.id))
            .select(dsl::count(ticket_instances::id))
            .first(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not load ticket pricing for ticket type",
            )?;
        Ok(valid_ticket_count as u32)
    }

    pub fn remaining_ticket_count(&self, conn: &PgConnection) -> Result<u32, DatabaseError> {
        let remaining_ticket_count: i64 = ticket_instances::table
            .inner_join(assets::table)
            .filter(assets::ticket_type_id.eq(self.id))
            .filter(
                ticket_instances::reserved_until
                    .lt(dsl::now.nullable())
                    .or(ticket_instances::reserved_until.is_null()),
            ).select(dsl::count(ticket_instances::id))
            .first(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not load ticket pricing for ticket type",
            )?;
        Ok(remaining_ticket_count as u32)
    }

    pub fn valid_ticket_count(&self, conn: &PgConnection) -> Result<u32, DatabaseError> {
        let valid_ticket_count: i64 = ticket_instances::table
            .inner_join(assets::table)
            .filter(assets::ticket_type_id.eq(self.id))
            .filter(ticket_instances::status.ne(TicketInstanceStatus::Nullified.to_string()))
            .select(dsl::count(ticket_instances::id))
            .first(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not load ticket pricing for ticket type",
            )?;
        Ok(valid_ticket_count as u32)
    }

    pub fn current_ticket_pricing(
        &self,
        conn: &PgConnection,
    ) -> Result<TicketPricing, DatabaseError> {
        TicketPricing::get_current_ticket_pricing(self.id, conn)
    }

    pub fn ticket_pricing(&self, conn: &PgConnection) -> Result<Vec<TicketPricing>, DatabaseError> {
        ticket_pricing::table
            .filter(ticket_pricing::ticket_type_id.eq(self.id))
            .order_by(ticket_pricing::name)
            .load(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not load ticket pricing for ticket type",
            )
    }

    pub fn valid_ticket_pricing(
        &self,
        conn: &PgConnection,
    ) -> Result<Vec<TicketPricing>, DatabaseError> {
        ticket_pricing::table
            .filter(ticket_pricing::ticket_type_id.eq(self.id))
            .filter(ticket_pricing::status.ne(TicketPricingStatus::Deleted.to_string()))
            .order_by(ticket_pricing::name)
            .load(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not load ticket pricing for ticket type",
            )
    }

    pub fn ticket_capacity(&self, conn: &PgConnection) -> Result<u32, DatabaseError> {
        //Calculate capacity by counting the number of ticket instances for event
        let ticket_capacity: i64 = assets::table
            .filter(assets::ticket_type_id.eq(self.id))
            .inner_join(ticket_instances::table)
            .select(dsl::count(ticket_instances::id))
            .first(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load ticket instances")?;
        Ok(ticket_capacity as u32)
    }

    pub fn add_ticket_pricing(
        &self,
        name: String,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
        price_in_cents: i64,
        conn: &PgConnection,
    ) -> Result<TicketPricing, DatabaseError> {
        TicketPricing::create(self.id, name, start_date, end_date, price_in_cents).commit(conn)
    }

    pub fn status(&self) -> TicketTypeStatus {
        self.status.parse::<TicketTypeStatus>().unwrap()
    }
}

#[derive(Insertable)]
#[table_name = "ticket_types"]
pub struct NewTicketType {
    event_id: Uuid,
    name: String,
    status: String,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
    increment: Option<i32>,
}

impl NewTicketType {
    pub fn commit(self, conn: &PgConnection) -> Result<TicketType, DatabaseError> {
        diesel::insert_into(ticket_types::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create ticket type")
    }
}
