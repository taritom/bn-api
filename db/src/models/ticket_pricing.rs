use chrono::NaiveDateTime;
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use models::{TicketPricingStatus, TicketType};
use schema::{order_items, ticket_pricing};
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Identifiable, Associations, Queryable, PartialEq, Debug)]
#[belongs_to(TicketType)]
#[table_name = "ticket_pricing"]
pub struct TicketPricing {
    pub id: Uuid,
    ticket_type_id: Uuid,
    pub name: String,
    status: String,
    pub price_in_cents: i64,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(AsChangeset, Default, Deserialize)]
#[table_name = "ticket_pricing"]
pub struct TicketPricingEditableAttributes {
    pub name: Option<String>,
    pub price_in_cents: Option<i64>,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
}

impl TicketPricing {
    pub fn create(
        ticket_type_id: Uuid,
        name: String,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
        price_in_cents: i64,
    ) -> NewTicketPricing {
        NewTicketPricing {
            ticket_type_id,
            name,
            status: TicketPricingStatus::Published.to_string(),
            start_date,
            end_date,
            price_in_cents,
        }
    }

    pub fn update(
        &self,
        attributes: TicketPricingEditableAttributes,
        conn: &PgConnection,
    ) -> Result<TicketPricing, DatabaseError> {
        diesel::update(self)
            .set((attributes, ticket_pricing::updated_at.eq(dsl::now)))
            .get_result(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not update ticket_pricing")
    }

    pub fn destroy(&self, conn: &PgConnection) -> Result<usize, DatabaseError> {
        //Check if there is any order items linked to this ticket pricing
        let affected_order_count: i64 = order_items::table
            .filter(order_items::ticket_pricing_id.eq(self.id))
            .select(dsl::count(order_items::id))
            .first(conn)
            .to_db_error(ErrorCode::QueryError, "Could not load order_items")?;
        if affected_order_count == 0 as i64 {
            //Ticket pricing is unused -> delete
            diesel::delete(self)
                .execute(conn)
                .to_db_error(ErrorCode::DeleteError, "Error removing ticket pricing")
        } else {
            //Ticket pricing is used -> mark status for deletion
            #[derive(AsChangeset)]
            #[table_name = "ticket_pricing"]
            struct R {
                pub status: String,
            }
            let status_attribute = R {
                status: TicketPricingStatus::Deleted.to_string(),
            };
            diesel::update(self)
                .set((status_attribute, ticket_pricing::updated_at.eq(dsl::now)))
                //.get_result(conn)
                .execute(conn)
                .to_db_error(ErrorCode::UpdateError, "Could not update ticket_pricing status")?;
            Ok(0 as usize)
        }
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<TicketPricing, DatabaseError> {
        ticket_pricing::table
            .find(id)
            .first::<TicketPricing>(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading ticket pricing")
    }

    pub fn status(&self) -> TicketPricingStatus {
        TicketPricingStatus::parse(&self.status).unwrap()
    }

    pub fn get_current_ticket_pricing(
        ticket_type_id: Uuid,
        conn: &PgConnection,
    ) -> Result<TicketPricing, DatabaseError> {
        let mut price_points = ticket_pricing::table
            .filter(ticket_pricing::ticket_type_id.eq(ticket_type_id))
            .filter(ticket_pricing::start_date.le(dsl::now))
            .filter(ticket_pricing::end_date.gt(dsl::now))
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Could not load Ticket Pricing")?;

        if price_points.len() > 1 {
            return Err(DatabaseError::new(
                ErrorCode::MultipleResultsWhenOneExpected,
                Some("Expected a single ticket pricing period but multiple were found"),
            ));
        } else if price_points.len() == 0 {
            return Err(DatabaseError::new(
                ErrorCode::NoResults,
                Some("No ticket pricing found"),
            ));
        }

        price_points.pop().ok_or(DatabaseError::new(
            ErrorCode::NoResults,
            Some("No ticket pricing found"),
        ))
    }
}

#[derive(Insertable)]
#[table_name = "ticket_pricing"]
pub struct NewTicketPricing {
    ticket_type_id: Uuid,
    name: String,
    status: String,
    price_in_cents: i64,
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
}

impl NewTicketPricing {
    pub fn commit(self, conn: &PgConnection) -> Result<TicketPricing, DatabaseError> {
        diesel::insert_into(ticket_pricing::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create ticket pricing")
    }
}
