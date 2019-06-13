use chrono::prelude::*;
use diesel;
use diesel::dsl::{count, exists, select, sql};
use diesel::expression::dsl;
use diesel::prelude::*;
use diesel::sql_types::{Array, BigInt, Nullable, Uuid as dUuid};
use models::*;
use schema::{
    assets, events, order_transfers, orders, ticket_instances, ticket_types, transfer_tickets,
    transfers,
};
use serde_json::Value;
use std::cmp::Ordering;
use tari_client::*;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;
use validator::*;
use validators::{self, *};

pub static TRANSFER_DRIP_NOTIFICATION_DAYS_PRIOR_TO_EVENT: &'static [i64] = &[7, 1, 0];
pub const TRANSFER_DRIP_NOTIFICATION_HOURS_PRIOR_TO_EVENT: i64 = 3;

#[derive(Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "transfers"]
pub struct NewTransfer {
    pub source_user_id: Uuid,
    pub transfer_key: Uuid,
    pub status: TransferStatus,
    pub transfer_message_type: Option<TransferMessageType>,
    pub transfer_address: Option<String>,
}

#[derive(Clone, Queryable, Identifiable, Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "transfers"]
pub struct Transfer {
    pub id: Uuid,
    pub source_user_id: Uuid,
    pub destination_user_id: Option<Uuid>,
    pub transfer_key: Uuid,
    pub status: TransferStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub transfer_message_type: Option<TransferMessageType>,
    pub transfer_address: Option<String>,
}

#[derive(AsChangeset, Default, Deserialize)]
#[table_name = "transfers"]
pub struct TransferEditableAttributes {
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub status: Option<TransferStatus>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub destination_user_id: Option<Uuid>,
}

#[derive(Clone, Queryable, Deserialize, Serialize, PartialEq, Debug)]
pub struct DisplayTransfer {
    pub id: Uuid,
    pub source_user_id: Uuid,
    pub destination_user_id: Option<Uuid>,
    pub transfer_key: Uuid,
    pub status: TransferStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub transfer_message_type: Option<TransferMessageType>,
    pub transfer_address: Option<String>,
    pub ticket_ids: Vec<Uuid>,
    pub event_ids: Vec<Uuid>,
    #[serde(skip_serializing)]
    pub total: Option<i64>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct ProcessTransferDripPayload {
    pub source_or_destination: SourceOrDestination,
    pub event_id: Uuid,
}

impl PartialOrd for Transfer {
    fn partial_cmp(&self, other: &Transfer) -> Option<Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl Transfer {
    pub fn receive_url(
        &self,
        front_end_url: String,
        conn: &PgConnection,
    ) -> Result<String, DatabaseError> {
        Ok(format!(
            "{}/tickets/transfers/receive?sender_user_id={}&transfer_key={}&num_tickets={}&signature={}",
            front_end_url,
            self.source_user_id,
            self.transfer_key,
            self.transfer_tickets(conn)?.len(),
            self.signature(conn)?
        )
        .to_string())
    }

    pub fn into_authorization(
        &self,
        conn: &PgConnection,
    ) -> Result<TransferAuthorization, DatabaseError> {
        Ok(TransferAuthorization {
            transfer_key: self.transfer_key,
            sender_user_id: self.source_user_id,
            num_tickets: self.transfer_tickets(conn)?.len() as u32,
            signature: self.signature(conn)?,
        })
    }

    pub fn drip_header(
        &self,
        event: &Event,
        source_or_destination: SourceOrDestination,
        include_links: bool,
        conn: &PgConnection,
    ) -> Result<String, DatabaseError> {
        if self.transfer_address.is_none() {
            return DatabaseError::business_process_error(
                "Cannot build drip header for transfer missing destination address",
            );
        }

        Ok(match event.days_until_event() {
            Some(days_until_event) => match source_or_destination {
                SourceOrDestination::Source => {
                    let mut destination_address = self.transfer_address.clone().unwrap();
                    if include_links
                        && self.transfer_message_type == Some(TransferMessageType::Email)
                    {
                        destination_address = format!(
                            "<a href='mailto:{}'>{}</a>",
                            destination_address, destination_address
                        );
                    }
                    match days_until_event {
                            0 => {
                                let (is_pm, hour) = event.get_all_localized_times(&event.venue(conn)?).event_start.unwrap().hour12();
                                let time_of_day_text = (if !is_pm || hour < 5 { "today" } else { "tonight" }).to_string();
                                format!("Time to take action! The show is {} and those tickets you sent to {} still haven't been claimed. Give them a nudge!", time_of_day_text, destination_address)
                            },
                            1 => format!("Uh oh! The show is tomorrow and those tickets you sent to {} still haven't been claimed. Give them a nudge!", destination_address),
                            _ => format!("Those tickets you sent to {} still haven't been claimed. Give them a nudge!", destination_address),
                        }
                }
                SourceOrDestination::Destination => {
                    let source_user = User::find(self.source_user_id, conn)?;
                    let mut name = if let (Some(first_name), Some(last_name)) =
                        (source_user.first_name, source_user.last_name)
                    {
                        vec![
                            first_name.clone(),
                            last_name
                                .chars()
                                .next()
                                .map(|c| format!("{}.", c))
                                .unwrap_or("".to_string()),
                        ]
                        .join(" ")
                    } else {
                        "another user".to_string()
                    };

                    if include_links && source_user.email.is_some() {
                        name = format!(
                            "<a href='mailto:{}'>{}</a>",
                            source_user.email.unwrap(),
                            name
                        );
                    }
                    match days_until_event {
                            0 => {
                                let (is_pm, hour) = event.get_all_localized_times(&event.venue(conn)?).event_start.unwrap().hour12();
                                let time_of_day_text = (if !is_pm || hour < 5 { "today" } else { "tonight" }).to_string();
                                format!("Time to take action! The event is {} and the tickets {} sent you are still waiting!", time_of_day_text, name)
                            },
                            1 => format!("Get your tickets! The event is TOMORROW and you still need to get the tickets that {} sent you!", name),
                            7 => format!("The event is only one week away and you still need to get the tickets that {} sent you!", name),
                            _ => format!("You still need to get the tickets that {} sent you!", name)
                        }
                }
            },
            None => "".to_string(),
        })
    }

    pub fn log_drip_domain_event(
        &self,
        source_or_destination: SourceOrDestination,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        let domain_event_type = match source_or_destination {
            SourceOrDestination::Source => DomainEventTypes::TransferTicketDripSourceSent,
            SourceOrDestination::Destination => DomainEventTypes::TransferTicketDripDestinationSent,
        };

        DomainEvent::create(
            domain_event_type,
            "Transfer drip sent".to_string(),
            Tables::Transfers,
            Some(self.id),
            None,
            None,
        )
        .commit(conn)?;

        Ok(())
    }

    pub fn transfer_ticket_count(&self, conn: &PgConnection) -> Result<i64, DatabaseError> {
        transfer_tickets::table
            .filter(transfer_tickets::transfer_id.eq(self.id))
            .select(count(transfer_tickets::id))
            .first(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not check transfer ticket count",
            )
    }

    pub fn signature(&self, conn: &PgConnection) -> Result<String, DatabaseError> {
        let mut message: String = self.transfer_key.to_string();
        message.push_str(self.source_user_id.to_string().as_str());
        message.push_str(self.transfer_ticket_count(conn)?.to_string().as_str());
        let secret_key = Wallet::find_default_for_user(self.source_user_id, conn)?.secret_key;
        Ok(convert_bytes_to_hexstring(&cryptographic_signature(
            &message,
            &convert_hexstring_to_bytes(&secret_key),
        )?))
    }

    pub fn create_drip_actions(
        &self,
        event: &Event,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        for source_or_destination in vec![
            SourceOrDestination::Destination,
            SourceOrDestination::Source,
        ] {
            DomainAction::create(
                None,
                DomainActionTypes::ProcessTransferDrip,
                None,
                json!(ProcessTransferDripPayload {
                    event_id: event.id,
                    source_or_destination,
                }),
                Some(Tables::Transfers.to_string()),
                Some(self.id),
            )
            .commit(conn)?;
        }
        Ok(())
    }

    pub fn can_process_drips(&self, conn: &PgConnection) -> Result<bool, DatabaseError> {
        Ok(self.status == TransferStatus::Pending
            && self.events_have_not_ended(conn)?
            && self.transfer_address.is_some())
    }

    pub fn events_have_not_ended(&self, conn: &PgConnection) -> Result<bool, DatabaseError> {
        select(exists(
            transfer_tickets::table
                .inner_join(
                    ticket_instances::table
                        .on(ticket_instances::id.eq(transfer_tickets::ticket_instance_id)),
                )
                .inner_join(assets::table.on(assets::id.eq(ticket_instances::asset_id)))
                .inner_join(ticket_types::table.on(ticket_types::id.eq(assets::ticket_type_id)))
                .inner_join(events::table.on(events::id.eq(ticket_types::event_id)))
                .filter(transfer_tickets::transfer_id.eq(self.id))
                .filter(events::event_end.gt(dsl::now.nullable())),
        ))
        .get_result(conn)
        .to_db_error(
            ErrorCode::QueryError,
            "Could not confirm if transfer has active events",
        )
    }

    pub fn find_by_transfer_key(
        transfer_key: Uuid,
        conn: &PgConnection,
    ) -> Result<Transfer, DatabaseError> {
        transfers::table
            .filter(transfers::transfer_key.eq(transfer_key))
            .select(transfers::all_columns)
            .distinct()
            .get_result(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading transfers")
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<Transfer, DatabaseError> {
        transfers::table
            .filter(transfers::id.eq(id))
            .first(conn)
            .to_db_error(ErrorCode::QueryError, "Could not find transfer")
    }

    pub fn find_for_user_for_display(
        user_id: Uuid,
        order_id: Option<Uuid>,
        source_or_destination: SourceOrDestination,
        start_time: Option<NaiveDateTime>,
        end_time: Option<NaiveDateTime>,
        limit: Option<u32>,
        page: Option<u32>,
        conn: &PgConnection,
    ) -> Result<Payload<DisplayTransfer>, DatabaseError> {
        let limit = limit.unwrap_or(100);
        let page = page.unwrap_or(0);

        let mut query = transfers::table
            .left_join(order_transfers::table.on(order_transfers::transfer_id.eq(transfers::id)))
            .then_order_by(transfers::created_at.desc())
            .into_boxed();

        match source_or_destination {
            SourceOrDestination::Source => {
                query = query.filter(transfers::source_user_id.eq(user_id));
            }
            SourceOrDestination::Destination => {
                query = query.filter(transfers::destination_user_id.eq(Some(user_id)));
            }
        }

        if let Some(order_id) = order_id {
            query = query.filter(order_transfers::order_id.eq(order_id));
        }

        if let Some(start_time) = start_time {
            query = query.filter(transfers::created_at.ge(start_time));
        }

        if let Some(end_time) = end_time {
            query = query.filter(transfers::created_at.le(end_time));
        }

        let transfers: Vec<DisplayTransfer> = query
            .select(transfers::all_columns)
            .limit(limit as i64)
            .offset(limit as i64 * page as i64)
            .then_order_by(transfers::created_at.desc())
            .select((
                transfers::id,
                transfers::source_user_id,
                transfers::destination_user_id,
                transfers::transfer_key,
                transfers::status,
                transfers::created_at,
                transfers::updated_at,
                transfers::transfer_message_type,
                transfers::transfer_address,
                sql::<Array<dUuid>>(
                    "
                    ARRAY(
                        SELECT ticket_instance_id
                        FROM transfer_tickets
                        WHERE transfer_tickets.transfer_id = transfers.id
                    ) as ticket_ids
                ",
                ),
                sql::<Array<dUuid>>(
                    "
                    ARRAY(
                        SELECT DISTINCT event_id
                        FROM transfer_tickets tt
                        JOIN ticket_instances ti ON tt.ticket_instance_id = ti.id
                        JOIN assets a ON a.id = ti.asset_id
                        JOIN ticket_types tt2 ON tt2.id = a.ticket_type_id
                        WHERE tt.transfer_id = transfers.id
                    ) as event_ids
                ",
                ),
                sql::<Nullable<BigInt>>("count(*) over() as total"),
            ))
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load transfers")?;

        let mut paging = Paging::new(page, limit);
        paging.total = transfers.first().map(|t| t.total.unwrap_or(0)).unwrap_or(0) as u64;
        Ok(Payload::new(transfers, paging))
    }

    pub fn for_display(&self, conn: &PgConnection) -> Result<DisplayTransfer, DatabaseError> {
        let ticket_ids: Vec<Uuid> = self
            .transfer_tickets(conn)?
            .iter()
            .map(|tt| tt.ticket_instance_id)
            .collect();
        let event_ids = self.events(conn)?.iter().map(|e| e.id).collect();

        Ok(DisplayTransfer {
            id: self.id,
            source_user_id: self.source_user_id,
            destination_user_id: self.destination_user_id,
            transfer_key: self.transfer_key,
            status: self.status,
            created_at: self.created_at,
            updated_at: self.updated_at,
            transfer_message_type: self.transfer_message_type,
            transfer_address: self.transfer_address.clone(),
            ticket_ids,
            event_ids,
            total: None,
        })
    }

    pub fn find_pending_by_ticket_instance_ids(
        ticket_instance_ids: &[Uuid],
        conn: &PgConnection,
    ) -> Result<Vec<Transfer>, DatabaseError> {
        transfers::table
            .inner_join(transfer_tickets::table.on(transfer_tickets::transfer_id.eq(transfers::id)))
            .filter(transfer_tickets::ticket_instance_id.eq_any(ticket_instance_ids))
            .filter(transfers::status.eq(TransferStatus::Pending))
            .select(transfers::all_columns)
            .distinct()
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading transfers")
    }

    pub fn transfer_tickets(
        &self,
        conn: &PgConnection,
    ) -> Result<Vec<TransferTicket>, DatabaseError> {
        transfer_tickets::table
            .filter(transfer_tickets::transfer_id.eq(self.id))
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Could not load transfer tickets")
    }

    pub fn cancel(
        &self,
        user_id: Uuid,
        new_transfer_key: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<Transfer, DatabaseError> {
        if self.status != TransferStatus::Pending {
            return Err(DatabaseError::new(
                ErrorCode::UpdateError,
                Some("Transfer cannot be cancelled as it is no longer pending".to_string()),
            ));
        }

        let transfer = self.update(
            TransferEditableAttributes {
                status: Some(TransferStatus::Cancelled),
                ..Default::default()
            },
            conn,
        )?;

        let mut domain_event_data = vec![(self.id, Tables::Transfers)];
        for transfer_ticket in transfer.transfer_tickets(conn)? {
            domain_event_data.push((transfer_ticket.ticket_instance_id, Tables::TicketInstances));
        }

        for (id, table) in domain_event_data {
            DomainEvent::create(
                DomainEventTypes::TransferTicketCancelled,
                "Ticket transfer was cancelled".to_string(),
                table,
                Some(id),
                Some(user_id),
                Some(json!({"old_transfer_key": self.transfer_key, "new_transfer_key": &new_transfer_key })),
            )
            .commit(conn)?;
        }

        Ok(transfer)
    }

    pub fn complete(
        &self,
        destination_user_id: Uuid,
        additional_data: Option<Value>,
        conn: &PgConnection,
    ) -> Result<Transfer, DatabaseError> {
        if self.status != TransferStatus::Pending {
            return Err(DatabaseError::new(
                ErrorCode::UpdateError,
                Some("Transfer cannot be completed as it is no longer pending".to_string()),
            ));
        }

        let transfer = self.update(
            TransferEditableAttributes {
                status: Some(TransferStatus::Completed),
                destination_user_id: Some(destination_user_id),
                ..Default::default()
            },
            conn,
        )?;

        let mut domain_event_data = vec![(self.id, Tables::Transfers)];
        for transfer_ticket in transfer.transfer_tickets(conn)? {
            domain_event_data.push((transfer_ticket.ticket_instance_id, Tables::TicketInstances));
        }

        for (id, table) in domain_event_data {
            DomainEvent::create(
                DomainEventTypes::TransferTicketCompleted,
                "Transfer ticket completed".to_string(),
                table,
                Some(id),
                None,
                additional_data.clone(),
            )
            .commit(conn)?;
        }

        User::find(self.source_user_id, conn)?.update_genre_info(conn)?;
        User::find(destination_user_id, conn)?.update_genre_info(conn)?;

        Ok(transfer)
    }

    fn transfer_key_unique(
        transfer_key: Uuid,
        conn: &PgConnection,
    ) -> Result<Result<(), ValidationError>, DatabaseError> {
        let transfer_key_in_use = select(exists(
            transfers::table.filter(transfers::transfer_key.eq(transfer_key)),
        ))
        .get_result(conn)
        .to_db_error(
            ErrorCode::QueryError,
            "Could not check if transfer transfer_key is unique",
        )?;

        if transfer_key_in_use {
            let validation_error =
                create_validation_error("uniqueness", "Transfer key is already in use");
            return Ok(Err(validation_error));
        }

        Ok(Ok(()))
    }

    pub fn add_transfer_ticket(
        &self,
        ticket_instance_id: Uuid,
        user_id: Uuid,
        additional_info: &Option<Value>,
        conn: &PgConnection,
    ) -> Result<TransferTicket, DatabaseError> {
        TransferTicket::create(ticket_instance_id, self.id).commit(user_id, &additional_info, conn)
    }

    pub fn find_pending(conn: &PgConnection) -> Result<Vec<Transfer>, DatabaseError> {
        transfers::table
            .filter(transfers::status.eq(TransferStatus::Pending))
            .select(transfers::all_columns)
            .distinct()
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading transfers")
    }

    pub fn create(
        source_user_id: Uuid,
        transfer_key: Uuid,
        transfer_message_type: Option<TransferMessageType>,
        transfer_address: Option<String>,
    ) -> NewTransfer {
        NewTransfer {
            transfer_address,
            transfer_message_type,
            source_user_id,
            transfer_key,
            status: TransferStatus::Pending,
        }
    }

    pub fn events(&self, conn: &PgConnection) -> Result<Vec<Event>, DatabaseError> {
        transfer_tickets::table
            .inner_join(
                ticket_instances::table
                    .on(ticket_instances::id.eq(transfer_tickets::ticket_instance_id)),
            )
            .inner_join(assets::table.on(assets::id.eq(ticket_instances::asset_id)))
            .inner_join(ticket_types::table.on(ticket_types::id.eq(assets::ticket_type_id)))
            .inner_join(events::table.on(events::id.eq(ticket_types::event_id)))
            .filter(transfer_tickets::transfer_id.eq(self.id))
            .select(events::all_columns)
            .distinct()
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Could not load transfer events")
    }

    pub fn orders(&self, conn: &PgConnection) -> Result<Vec<Order>, DatabaseError> {
        order_transfers::table
            .inner_join(orders::table.on(orders::id.eq(order_transfers::order_id)))
            .filter(order_transfers::transfer_id.eq(self.id))
            .select(orders::all_columns)
            .then_order_by(orders::created_at.desc())
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Could not load transfer orders")
    }

    pub fn update_associated_orders(&self, conn: &PgConnection) -> Result<(), DatabaseError> {
        if self.status != TransferStatus::Pending {
            return Err(DatabaseError::new(
                ErrorCode::UpdateError,
                Some("Transfer cannot be updated as it is no longer pending".to_string()),
            ));
        }
        let query = r#"
            INSERT INTO order_transfers (order_id, transfer_id)
            SELECT DISTINCT o.id, t.id
            FROM transfers t
            JOIN transfer_tickets tt ON t.id = tt.transfer_id
            JOIN ticket_instances ti ON ti.id = tt.ticket_instance_id
            JOIN order_items oi ON ti.order_item_id = oi.id
            JOIN orders o ON o.id = oi.order_id
            WHERE t.id = $1
            AND COALESCE(o.on_behalf_of_user_id, o.user_id) = $2
        "#;

        diesel::sql_query(query)
            .bind::<dUuid, _>(self.id)
            .bind::<dUuid, _>(self.source_user_id)
            .execute(conn)
            .to_db_error(ErrorCode::QueryError, "Could not update associated orders")?;
        Ok(())
    }

    pub fn update(
        &self,
        attributes: TransferEditableAttributes,
        conn: &PgConnection,
    ) -> Result<Transfer, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update transfer",
            diesel::update(self)
                .set((attributes, transfers::updated_at.eq(dsl::now)))
                .get_result(conn),
        )
    }
}

impl NewTransfer {
    pub fn commit(
        &self,
        additional_data: &Option<Value>,
        conn: &PgConnection,
    ) -> Result<Transfer, DatabaseError> {
        self.validate_record(conn)?;
        let result: Transfer = DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new transfer",
            diesel::insert_into(transfers::table)
                .values(self)
                .get_result(conn),
        )?;

        DomainEvent::create(
            DomainEventTypes::TransferTicketStarted,
            "Transfer ticket started".to_string(),
            Tables::Transfers,
            Some(result.id),
            Some(self.source_user_id),
            additional_data.clone(),
        )
        .commit(conn)?;

        Ok(result)
    }

    fn validate_record(&self, conn: &PgConnection) -> Result<(), DatabaseError> {
        let validation_errors = validators::append_validation_error(
            Ok(()),
            "transfer_key",
            Transfer::transfer_key_unique(self.transfer_key, conn)?,
        );

        Ok(validation_errors?)
    }
}
