use chrono::NaiveDateTime;
use diesel;
use diesel::expression::dsl;
use diesel::pg::expression::dsl::any;
use diesel::prelude::*;
use itertools::Itertools;
use models::*;
use schema::{domain_event_published, domain_event_publishers};
use serde_json::Value;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use utils::errors::*;
use uuid::Uuid;
use validator::Validate;

pub static SUPPORTED_DOMAIN_EVENT_TYPES_FOR_PUBLISHING: &'static [DomainEventTypes] = &[
    DomainEventTypes::TransferTicketStarted,
    DomainEventTypes::TransferTicketCancelled,
    DomainEventTypes::TransferTicketCompleted,
    DomainEventTypes::UserCreated,
    DomainEventTypes::OrderCompleted,
    DomainEventTypes::OrderRefund,
    DomainEventTypes::OrderResendConfirmationTriggered,
    DomainEventTypes::TemporaryUserCreated,
    DomainEventTypes::PushNotificationTokenCreated,
];

#[derive(Clone, Debug, Serialize, Identifiable, Queryable, QueryableByName)]
#[table_name = "domain_event_publishers"]
pub struct DomainEventPublisher {
    pub id: Uuid,
    pub organization_id: Option<Uuid>,
    pub event_types: Vec<DomainEventTypes>,
    pub webhook_url: String,
    pub import_historic_events: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub last_domain_event_seq: Option<i64>,
    pub deleted_at: Option<NaiveDateTime>,
    pub adapter: Option<WebhookAdapters>,
    pub adapter_config: Option<Value>,
}

impl Eq for DomainEventPublisher {}

impl PartialEq for DomainEventPublisher {
    fn eq(&self, other: &DomainEventPublisher) -> bool {
        self.id.eq(&other.id)
    }
}

impl Hash for DomainEventPublisher {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

#[derive(AsChangeset, Deserialize, Validate)]
#[table_name = "domain_event_publishers"]
pub struct DomainEventPublisherEditableAttributes {
    #[validate(url(message = "Webhook URL is invalid"))]
    pub webhook_url: Option<String>,
    pub import_historic_events: Option<bool>,
}

impl DomainEventPublisher {
    pub fn find_all(conn: &PgConnection) -> Result<Vec<DomainEventPublisher>, DatabaseError> {
        domain_event_publishers::table
            .filter(domain_event_publishers::deleted_at.is_null())
            .order_by(domain_event_publishers::last_domain_event_seq.asc())
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Could not load Domain Event Publishers")
    }

    pub fn delete(self, conn: &PgConnection) -> Result<(), DatabaseError> {
        diesel::update(&self)
            .set((
                domain_event_publishers::deleted_at.eq(dsl::now.nullable()),
                domain_event_publishers::updated_at.eq(dsl::now),
            ))
            .execute(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not delete domain event publisher")?;

        Ok(())
    }

    pub fn find_with_unpublished_domain_events(
        limit: i64,
        conn: &PgConnection,
    ) -> Result<HashMap<DomainEventPublisher, Vec<DomainEvent>>, DatabaseError> {
        use schema::*;
        let mut publisher_unpublished_domain_events: HashMap<DomainEventPublisher, Vec<DomainEvent>> = HashMap::new();

        #[derive(Queryable, Deserialize)]
        struct R {
            domain_event_publisher_id: Uuid,
            domain_event_id: Uuid,
            _created_at: NaiveDateTime,
        }
        let unpublished_domain_event_data: Vec<R> = domain_event_publishers::table
            .inner_join(
                domain_events::table.on(domain_events::event_type.eq(any(domain_event_publishers::event_types))),
            )
            .left_join(
                domain_event_published::table.on(domain_event_published::domain_event_id
                    .eq(domain_events::id)
                    .and(domain_event_published::domain_event_publisher_id.eq(domain_event_publishers::id))),
            )
            .left_join(
                organizations::table.on(domain_event_publishers::organization_id.eq(organizations::id.nullable())),
            )
            .left_join(events::table.on(events::organization_id.eq(organizations::id)))
            .left_join(order_items::table.on(order_items::event_id.eq(events::id.nullable())))
            .left_join(
                orders::table.on(order_items::order_id
                    .eq(orders::id)
                    .and(domain_events::main_id.eq(orders::id.nullable()))
                    .and(domain_events::main_table.eq(Tables::Orders))),
            )
            .left_join(ticket_types::table.on(ticket_types::event_id.eq(events::id)))
            .left_join(assets::table.on(assets::ticket_type_id.eq(ticket_types::id)))
            .left_join(ticket_instances::table.on(ticket_instances::asset_id.eq(assets::id)))
            .left_join(transfer_tickets::table.on(transfer_tickets::ticket_instance_id.eq(ticket_instances::id)))
            .left_join(
                transfers::table.on(transfer_tickets::transfer_id.eq(transfers::id).and(
                    domain_events::main_table
                        .eq(Tables::TemporaryUsers)
                        .or(domain_events::main_id
                            .eq(transfers::id.nullable())
                            .and(domain_events::main_table.eq(Tables::Transfers))),
                )),
            )
            .left_join(
                temporary_users::table.on(transfers::destination_temporary_user_id
                    .eq(temporary_users::id.nullable())
                    .and(domain_events::main_id.eq(transfers::id.nullable()))
                    .and(domain_events::main_table.eq(Tables::TemporaryUsers))),
            )
            .left_join(
                push_notification_tokens::table.on(domain_events::main_table
                    .eq(Tables::PushNotificationTokens)
                    .and(domain_events::main_id.eq(push_notification_tokens::id.nullable()))),
            )
            .left_join(
                users::table.on(domain_events::main_table.ne(Tables::TemporaryUsers).and(
                    users::id
                        .eq(transfers::source_user_id)
                        .or(push_notification_tokens::user_id.eq(users::id))
                        .or(transfers::destination_user_id.eq(users::id.nullable()))
                        .or(domain_events::main_id
                            .eq(transfers::id.nullable())
                            .and(domain_events::main_table.eq(Tables::Users)))
                        .or(orders::on_behalf_of_user_id.eq(users::id.nullable()))
                        .or(orders::on_behalf_of_user_id
                            .is_null()
                            .and(orders::user_id.eq(users::id))),
                )),
            )
            .left_join(
                organization_interactions::table.on(organization_interactions::organization_id
                    .eq(organizations::id)
                    .and(organization_interactions::user_id.eq(users::id))),
            )
            .filter(
                domain_event_publishers::import_historic_events
                    .eq(true)
                    .or(domain_events::created_at.ge(domain_event_publishers::created_at)),
            )
            .filter(domain_event_published::domain_event_id.is_null())
            .filter(
                // No filter for publisher
                domain_event_publishers::organization_id
                    .is_null()
                    // Found user connected to publisher organization
                    .or(organization_interactions::id.is_not_null())
                    // Found temporary user connected to publisher organization
                    .or(temporary_users::id.is_not_null()),
            )
            .select((
                domain_event_publishers::id,
                domain_events::id,
                domain_events::created_at,
            ))
            .distinct()
            .order_by(domain_event_publishers::id.asc())
            .then_order_by(domain_events::created_at.asc())
            .limit(limit)
            .get_results(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading domain event publishers")?;

        for (domain_event_publisher_id, domain_event_ids) in &unpublished_domain_event_data
            .into_iter()
            .group_by(|data| data.domain_event_publisher_id)
        {
            let domain_event_publisher = DomainEventPublisher::find(domain_event_publisher_id, conn)?;
            let domain_events = DomainEvent::find_by_ids(
                domain_event_ids
                    .into_iter()
                    .map(|data| data.domain_event_id)
                    .collect_vec(),
                conn,
            )?;
            publisher_unpublished_domain_events.insert(domain_event_publisher, domain_events);
        }

        Ok(publisher_unpublished_domain_events)
    }

    pub fn claim_for_publishing(&self, domain_event: &DomainEvent, conn: &PgConnection) -> Result<bool, DatabaseError> {
        // Mark domain event published for this publisher
        let rows_affected = diesel::insert_into(domain_event_published::table)
            .values((
                domain_event_published::domain_event_publisher_id.eq(self.id),
                domain_event_published::domain_event_id.eq(domain_event.id),
            ))
            .on_conflict_do_nothing()
            .execute(conn)
            .to_db_error(ErrorCode::InsertError, "Could not insert domain event published")?;

        if rows_affected == 0 {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn update_last_domain_event_seq(
        &mut self,
        last_domain_event_seq: i64,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        use diesel::dsl;
        use schema::*;
        let res: Option<DomainEventPublisher> = diesel::update(
            domain_event_publishers::table
                .filter(
                    domain_event_publishers::last_domain_event_seq
                        .is_null()
                        .or(domain_event_publishers::last_domain_event_seq.lt(last_domain_event_seq)),
                )
                .filter(domain_event_publishers::id.eq(self.id)),
        )
        .set((
            domain_event_publishers::last_domain_event_seq.eq(last_domain_event_seq),
            domain_event_publishers::updated_at.eq(dsl::now),
        ))
        .get_result(conn)
        .to_db_error(ErrorCode::UpdateError, "Could not update domain event publisher")
        .optional()?;

        let r = match res {
            Some(r) => Ok(r),
            None =>
            // A later event has already been published....
            {
                DomainEventPublisher::find(self.id, conn)
            }
        }?;

        self.last_domain_event_seq = r.last_domain_event_seq;
        Ok(())
    }

    pub fn create(
        organization_id: Option<Uuid>,
        event_types: Vec<DomainEventTypes>,
        webhook_url: String,
    ) -> NewDomainEventPublisher {
        NewDomainEventPublisher {
            organization_id,
            event_types,
            webhook_url,
            adapter: None,
            adapter_config: None,
        }
    }

    pub fn create_with_adapter(
        organization_id: Option<Uuid>,
        event_types: Vec<DomainEventTypes>,
        adapter: WebhookAdapters,
        adapter_config: Value,
    ) -> NewDomainEventPublisher {
        NewDomainEventPublisher {
            organization_id,
            event_types,
            webhook_url: "".to_string(),
            adapter: Some(adapter),
            adapter_config: Some(adapter_config),
        }
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<DomainEventPublisher, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading domain event publisher",
            domain_event_publishers::table
                .find(id)
                .first::<DomainEventPublisher>(conn),
        )
    }

    pub fn update(
        &self,
        attributes: &DomainEventPublisherEditableAttributes,
        conn: &PgConnection,
    ) -> Result<DomainEventPublisher, DatabaseError> {
        diesel::update(self)
            .set((attributes, domain_event_publishers::updated_at.eq(dsl::now)))
            .get_result(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not update domain event publisher")
    }
}

#[derive(Clone, Deserialize, Insertable, Validate)]
#[table_name = "domain_event_publishers"]
pub struct NewDomainEventPublisher {
    pub organization_id: Option<Uuid>,
    pub event_types: Vec<DomainEventTypes>,
    pub webhook_url: String,
    pub adapter: Option<WebhookAdapters>,
    pub adapter_config: Option<Value>,
}

impl NewDomainEventPublisher {
    pub fn commit(self, conn: &PgConnection) -> Result<DomainEventPublisher, DatabaseError> {
        diesel::insert_into(domain_event_publishers::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not insert domain event publisher")
    }
}
