use chrono::NaiveDateTime;
use diesel;
use diesel::prelude::*;
use models::*;
use schema::event_report_subscribers;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use utils::errors::*;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Identifiable, PartialEq, Queryable, Serialize)]
#[table_name = "event_report_subscribers"]
pub struct EventReportSubscriber {
    pub id: Uuid,
    pub event_id: Uuid,
    pub email: String,
    pub report_type: ReportTypes,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Validate)]
#[table_name = "event_report_subscribers"]
pub struct NewEventReportSubscriber {
    pub event_id: Uuid,
    #[validate(email(message = "Email is invalid"))]
    pub email: String,
    pub report_type: ReportTypes,
}

impl NewEventReportSubscriber {
    pub fn commit(
        mut self,
        user_id: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<EventReportSubscriber, DatabaseError> {
        self.email = self.email.clone().to_lowercase();
        self.validate()?;
        let event_report_subscriber: EventReportSubscriber = diesel::insert_into(event_report_subscribers::table)
            .values(&self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create new event report subscriber")?;

        DomainEvent::create(
            DomainEventTypes::EventReportSubscriberCreated,
            "Event report subscriber created".to_string(),
            Tables::Events,
            Some(event_report_subscriber.event_id),
            user_id,
            Some(json!({"email": event_report_subscriber.email, "event_report_subscriber_id": event_report_subscriber.id })),
        )
        .commit(conn)?;

        Ok(event_report_subscriber)
    }
}

impl EventReportSubscriber {
    pub fn create(event_id: Uuid, report_type: ReportTypes, email: String) -> NewEventReportSubscriber {
        NewEventReportSubscriber {
            event_id,
            report_type,
            email,
        }
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<EventReportSubscriber, DatabaseError> {
        event_report_subscribers::table
            .find(id)
            .first(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading event report subscriber")
    }

    pub fn destroy(&self, user_id: Option<Uuid>, conn: &PgConnection) -> Result<(), DatabaseError> {
        diesel::delete(event_report_subscribers::table.filter(event_report_subscribers::id.eq(self.id)))
            .execute(conn)
            .to_db_error(ErrorCode::QueryError, "Error removing event report subscriber")?;

        DomainEvent::create(
            DomainEventTypes::EventReportSubscriberDeleted,
            "Event report subscriber deleted".to_string(),
            Tables::Events,
            Some(self.event_id),
            user_id,
            Some(json!({"email": self.email, "event_report_subscriber_id": self.id })),
        )
        .commit(conn)?;

        Ok(())
    }

    pub fn find_all(
        event_id: Uuid,
        report_type: ReportTypes,
        conn: &PgConnection,
    ) -> Result<Vec<EventReportSubscriber>, DatabaseError> {
        event_report_subscribers::table
            .filter(event_report_subscribers::event_id.eq(event_id))
            .filter(event_report_subscribers::report_type.eq(report_type))
            .order_by(event_report_subscribers::email)
            .load::<EventReportSubscriber>(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading event report subscribers")
    }
}
