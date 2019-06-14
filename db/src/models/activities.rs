use chrono::prelude::*;
use diesel::expression::sql_literal::sql;
use diesel::pg::types::sql_types::Array;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Nullable, Text, Timestamp, Uuid as dUuid};
use models::*;
use std::cmp::Reverse;
use std::collections::HashMap;
use utils::errors::*;
use uuid::Uuid;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct ActivitySummary {
    pub event: DisplayEvent,
    pub activity_items: Vec<ActivityItem>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Queryable, QueryableByName)]
pub struct EventActivityItem {
    #[sql_type = "dUuid"]
    pub event_id: Uuid,
    #[sql_type = "Text"]
    pub name: String,
    #[sql_type = "Nullable<Text>"]
    pub code: Option<String>,
    #[sql_type = "Nullable<BigInt>"]
    pub code_discount_in_cents: Option<i64>,
    #[sql_type = "Nullable<Text>"]
    pub code_type: Option<String>,
    #[sql_type = "BigInt"]
    pub quantity: i64,
    #[sql_type = "BigInt"]
    pub total_in_cents: i64,
    #[serde(skip_serializing)]
    #[sql_type = "dUuid"]
    pub order_id: Uuid,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Queryable, QueryableByName)]
pub struct RefundActivityItem {
    #[sql_type = "dUuid"]
    pub id: Uuid,
    #[sql_type = "BigInt"]
    pub amount: i64,
    #[sql_type = "BigInt"]
    pub quantity: i64,
    #[sql_type = "Text"]
    pub item_type: OrderItemTypes,
    #[sql_type = "Nullable<Text>"]
    pub ticket_type_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UserActivityItem {
    pub id: Uuid,
    pub full_name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum ActivityItem {
    Purchase {
        order_id: Uuid,
        order_number: String,
        ticket_quantity: i64,
        total_in_cents: i64,
        events: Vec<EventActivityItem>,
        occurred_at: NaiveDateTime,
        purchased_by: UserActivityItem,
        user: UserActivityItem,
    },
    Transfer {
        transfer_id: Uuid,
        action: String,
        status: TransferStatus,
        ticket_ids: Vec<Uuid>,
        destination_addresses: Option<String>,
        transfer_message_type: Option<TransferMessageType>,
        initated_by: UserActivityItem,
        accepted_by: Option<UserActivityItem>,
        cancelled_by: Option<UserActivityItem>,
        occurred_at: NaiveDateTime,
    },
    CheckIn {
        ticket_instance_id: Uuid,
        redeemed_for: UserActivityItem,
        redeemed_by: UserActivityItem,
        occurred_at: NaiveDateTime,
    },
    Refund {
        refund_id: Uuid,
        refund_items: Vec<RefundActivityItem>,
        reason: Option<String>,
        total_in_cents: i64,
        refunded_by: UserActivityItem,
        occurred_at: NaiveDateTime,
    },
    Note {
        note_id: Uuid,
        order_id: Uuid,
        created_by: UserActivityItem,
        note: String,
        occurred_at: NaiveDateTime,
    },
}

impl ActivityItem {
    pub fn load_for_event(
        event_id: Uuid,
        user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<ActivityItem>, DatabaseError> {
        let mut activity_items: Vec<ActivityItem> = Vec::new();
        activity_items.append(&mut ActivityItem::load_purchases(
            None,
            Some(event_id),
            Some(user_id),
            conn,
        )?);
        activity_items.append(&mut ActivityItem::load_transfers(
            None,
            Some(event_id),
            Some(user_id),
            conn,
        )?);
        activity_items.append(&mut ActivityItem::load_check_ins(
            None,
            Some(event_id),
            Some(user_id),
            conn,
        )?);
        activity_items.append(&mut ActivityItem::load_refunds(
            None,
            Some(event_id),
            Some(user_id),
            conn,
        )?);
        activity_items.append(&mut ActivityItem::load_notes(
            None,
            Some(event_id),
            Some(user_id),
            conn,
        )?);
        activity_items.sort_by_key(|activity| Reverse(activity.occurred_at()));
        Ok(activity_items)
    }

    pub fn load_for_order(
        order: &Order,
        conn: &PgConnection,
    ) -> Result<Vec<ActivityItem>, DatabaseError> {
        let mut activity_items: Vec<ActivityItem> = Vec::new();
        activity_items.append(&mut ActivityItem::load_purchases(
            Some(order.id),
            None,
            None,
            conn,
        )?);
        activity_items.append(&mut ActivityItem::load_transfers(
            Some(order.id),
            None,
            None,
            conn,
        )?);
        activity_items.append(&mut ActivityItem::load_check_ins(
            Some(order.id),
            None,
            None,
            conn,
        )?);
        activity_items.append(&mut ActivityItem::load_refunds(
            Some(order.id),
            None,
            None,
            conn,
        )?);
        activity_items.append(&mut ActivityItem::load_notes(
            Some(order.id),
            None,
            None,
            conn,
        )?);
        activity_items.sort_by_key(|activity| Reverse(activity.occurred_at()));
        Ok(activity_items)
    }

    fn load_purchases(
        order_id: Option<Uuid>,
        event_id: Option<Uuid>,
        user_id: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<Vec<ActivityItem>, DatabaseError> {
        use schema::*;
        if order_id.is_none() && event_id.is_none() {
            return Err(DatabaseError::new(
                ErrorCode::BusinessProcessError,
                Some(
                    "Activity loading requires either order_id or event_id to be present"
                        .to_string(),
                ),
            ));
        }

        #[derive(Debug, Queryable, QueryableByName)]
        struct R {
            #[sql_type = "dUuid"]
            order_id: Uuid,
            #[sql_type = "BigInt"]
            ticket_quantity: i64,
            #[sql_type = "Timestamp"]
            occurred_at: NaiveDateTime,
            #[sql_type = "BigInt"]
            total_in_cents: i64,
            #[sql_type = "Array<dUuid>"]
            event_ids: Vec<Uuid>,
            #[sql_type = "dUuid"]
            purchased_by: Uuid,
            #[sql_type = "dUuid"]
            user_id: Uuid,
        }

        let mut query = orders::table
            .inner_join(order_items::table.on(order_items::order_id.eq(orders::id)))
            .inner_join(events::table.on(order_items::event_id.eq(events::id.nullable())))
            .filter(orders::status.eq(OrderStatus::Paid))
            .into_boxed();

        if let (Some(event_id), Some(user_id)) = (event_id, user_id) {
            query = query.filter(events::id.eq(event_id)).filter(
                orders::on_behalf_of_user_id
                    .eq(Some(user_id))
                    .or(orders::user_id.eq(user_id)),
            );
        } else if let Some(order_id) = order_id {
            query = query.filter(orders::id.eq(order_id));
        }
        let order_data: Vec<R> = query
            .select((
                orders::id,
                sql::<BigInt>(
                    "CAST(
                        (SELECT sum(quantity)
                        FROM order_items oi
                        WHERE oi.order_id = orders.id
                        AND oi.item_type = 'Tickets')
                    as BigInt) as ticket_quantity",
                ),
                orders::created_at,
                sql("CAST(SUM(order_items.unit_price_in_cents * order_items.quantity) as BigInt) as total_in_cents"),
                sql::<Array<dUuid>>(
                    "
                    ARRAY(
                        SELECT DISTINCT oi.event_id
                        FROM order_items oi
                        WHERE oi.order_id = orders.id
                    ) as event_ids
                ",
                ),
                orders::user_id,
                sql::<dUuid>(
                    "
                    COALESCE(orders.on_behalf_of_user_id, orders.user_id)
                ",
                ),
            ))
            .group_by((orders::id, orders::created_at, orders::user_id, orders::on_behalf_of_user_id, orders::user_id))
            .distinct()
            .load(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Unable to load purchases for organization fan",
            )?;

        let mut user_ids = Vec::new();
        for order_datum in &order_data {
            user_ids.push(order_datum.purchased_by);
            user_ids.push(order_datum.user_id);
        }
        user_ids.sort();
        user_ids.dedup();
        let users = User::find_by_ids(&user_ids, conn)?;
        let mut user_map: HashMap<Uuid, UserActivityItem> = HashMap::new();
        for user in users {
            user_map.insert(user.id, user.into());
        }

        let order_ids: Vec<Uuid> = order_data.iter().map(|d| d.order_id).collect();
        let order_event_data: Vec<EventActivityItem> = orders::table
            .inner_join(order_items::table.on(order_items::order_id.eq(orders::id)))
            .left_join(ticket_pricing::table.on(order_items::ticket_pricing_id.eq(ticket_pricing::id.nullable())))
            .left_join(codes::table.on(order_items::code_id.eq(codes::id.nullable())))
            .left_join(holds::table.on(order_items::hold_id.eq(holds::id.nullable())))
            .inner_join(events::table.on(order_items::event_id.eq(events::id.nullable())))
            .filter(orders::id.eq_any(order_ids))
            .filter(order_items::parent_id.is_null())
            .group_by((
                orders::id,
                events::id,
                events::name,
                holds::hold_type,
                codes::code_type,
                holds::redemption_code,
                codes::redemption_code,
                holds::discount_in_cents,
                codes::discount_in_cents,
            ))
            .select((
                events::id,
                events::name,
                sql("COALESCE(holds.redemption_code, codes.redemption_code) as code"),
                sql("COALESCE(holds.discount_in_cents, codes.discount_in_cents) as discount_in_cents"),
                sql("COALESCE(holds.hold_type, codes.code_type) as code_type"),
                sql("CAST(COALESCE(SUM(order_items.quantity) FILTER (WHERE order_items.item_type = 'Tickets'), 0) as BigInt) as quantity"),
                // Sum up all children in calculation, event fees are split off but children of holds/codes are combined
                sql("
                COALESCE((
                    SELECT CAST(SUM(oi2.unit_price_in_cents * oi2.quantity) as BigInt)
                    FROM order_items oi
                    JOIN order_items oi2 ON oi.id = oi2.parent_id OR oi.id = oi2.id
                    LEFT JOIN holds h ON h.id = oi.hold_id
                    LEFT JOIN codes c ON c.id = oi.code_id
                    WHERE COALESCE(holds.redemption_code, codes.redemption_code, '') = COALESCE(h.redemption_code, c.redemption_code, '')
                    AND oi.event_id = events.id
                    AND oi.order_id = orders.id
                    AND oi.parent_id is null
                ), 0) as total_in_cents"),
                orders::id
            ))
            .distinct()
            .load(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Unable to load purchase event data for organization fan",
            )?;

        let mut event_map: HashMap<Uuid, Vec<EventActivityItem>> = HashMap::new();
        for order_event_datum in order_event_data {
            event_map
                .entry(order_event_datum.order_id)
                .or_insert(Vec::new())
                .push(order_event_datum.clone());
        }

        let mut activity_items: Vec<ActivityItem> = Vec::new();
        for order_datum in order_data {
            let purchased_by = user_map
                .get(&order_datum.purchased_by)
                .map(|u| u.clone())
                .ok_or_else(|| {
                    DatabaseError::new(
                        ErrorCode::BusinessProcessError,
                        Some("Order can't load purchasing user".to_string()),
                    )
                })?;
            let user = user_map
                .get(&order_datum.user_id)
                .map(|u| u.clone())
                .ok_or_else(|| {
                    DatabaseError::new(
                        ErrorCode::BusinessProcessError,
                        Some("Order can't load user".to_string()),
                    )
                })?;

            activity_items.push(ActivityItem::Purchase {
                order_id: order_datum.order_id,
                order_number: Order::parse_order_number(order_datum.order_id),
                ticket_quantity: order_datum.ticket_quantity,
                events: event_map
                    .get(&order_datum.order_id)
                    .map(|e| e.clone())
                    .unwrap_or(Vec::new()),
                occurred_at: order_datum.occurred_at,
                purchased_by,
                total_in_cents: order_datum.total_in_cents,
                user,
            });
        }

        Ok(activity_items)
    }

    fn load_transfers(
        order_id: Option<Uuid>,
        event_id: Option<Uuid>,
        user_id: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<Vec<ActivityItem>, DatabaseError> {
        use schema::*;
        if order_id.is_none() && event_id.is_none() {
            return Err(DatabaseError::new(
                ErrorCode::BusinessProcessError,
                Some(
                    "Activity loading requires either order_id or event_id to be present"
                        .to_string(),
                ),
            ));
        }

        #[derive(Debug, Queryable, QueryableByName)]
        struct R {
            #[sql_type = "dUuid"]
            transfer_id: Uuid,
            #[sql_type = "Text"]
            action: String,
            #[sql_type = "Text"]
            status: TransferStatus,
            #[sql_type = "Array<dUuid>"]
            ticket_ids: Vec<Uuid>,
            #[sql_type = "Text"]
            destination_addresses: Option<String>,
            #[sql_type = "Text"]
            transfer_message_type: Option<TransferMessageType>,
            #[sql_type = "Timestamp"]
            occurred_at: NaiveDateTime,
            #[sql_type = "dUuid"]
            initated_by: Uuid,
            #[sql_type = "Nullable<dUuid>"]
            accepted_by: Option<Uuid>,
            #[sql_type = "Nullable<dUuid>"]
            cancelled_by: Option<Uuid>,
        }

        let mut query = transfers::table
            .inner_join(transfer_tickets::table.on(transfer_tickets::transfer_id.eq(transfers::id)))
            .inner_join(
                ticket_instances::table
                    .on(transfer_tickets::ticket_instance_id.eq(ticket_instances::id)),
            )
            .inner_join(assets::table.on(ticket_instances::asset_id.eq(assets::id)))
            .inner_join(ticket_types::table.on(assets::ticket_type_id.eq(ticket_types::id)))
            .left_join(
                order_items::table
                    .on(ticket_instances::order_item_id.eq(order_items::id.nullable())),
            )
            .left_join(orders::table.on(orders::id.eq(order_items::order_id)))
            .inner_join(
                domain_events::table.on(domain_events::main_id
                    .eq(transfers::id.nullable())
                    .and(domain_events::main_table.eq(Tables::Transfers))
                    .and(domain_events::event_type.eq_any(vec![
                        DomainEventTypes::TransferTicketCancelled,
                        DomainEventTypes::TransferTicketCompleted,
                        DomainEventTypes::TransferTicketStarted,
                    ]))),
            )
            .into_boxed();

        if let (Some(event_id), Some(user_id)) = (event_id, user_id) {
            query = query.filter(ticket_types::event_id.eq(event_id)).filter(
                transfers::source_user_id
                    .eq(user_id)
                    .or(transfers::cancelled_by_user_id.eq(user_id).and(
                        domain_events::event_type.eq(DomainEventTypes::TransferTicketCancelled),
                    ))
                    .or(transfers::destination_user_id.eq(user_id).and(
                        domain_events::event_type.eq(DomainEventTypes::TransferTicketCompleted),
                    )),
            );
        } else if let Some(order_id) = order_id {
            query = query.filter(orders::id.eq(order_id));
        }

        let transfer_data: Vec<R> = query
            .group_by((
                transfers::id,
                transfers::status,
                transfers::transfer_address,
                transfers::transfer_message_type,
                domain_events::created_at,
                domain_events::event_type,
                transfers::source_user_id,
                transfers::destination_user_id,
                transfers::cancelled_by_user_id,
            ))
            .select((
                transfers::id,
                sql::<Text>(
                    "
                    CASE domain_events.event_type
                    WHEN 'TransferTicketCancelled' THEN 'Cancelled'
                    WHEN 'TransferTicketCompleted' THEN 'Accepted'
                    WHEN 'TransferTicketStarted' THEN 'Started'
                    ELSE 'Unknown'
                    END
                ",
                ),
                transfers::status,
                sql::<Array<dUuid>>("array_agg(distinct ticket_instances.id)"),
                transfers::transfer_address,
                transfers::transfer_message_type,
                domain_events::created_at,
                transfers::source_user_id,
                transfers::destination_user_id,
                transfers::cancelled_by_user_id,
            ))
            .distinct()
            .load(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Unable to load transfers for organization fan",
            )?;

        let mut user_ids = Vec::new();
        for transfer_datum in &transfer_data {
            user_ids.push(transfer_datum.initated_by);

            if let Some(accepted_by) = transfer_datum.accepted_by {
                user_ids.push(accepted_by);
            }

            if let Some(cancelled_by) = transfer_datum.cancelled_by {
                user_ids.push(cancelled_by);
            }
        }
        user_ids.sort();
        user_ids.dedup();
        let users = User::find_by_ids(&user_ids, conn)?;
        let mut user_map: HashMap<Uuid, UserActivityItem> = HashMap::new();
        for user in users {
            user_map.insert(user.id, user.into());
        }

        let mut activity_items: Vec<ActivityItem> = Vec::new();
        for transfer_datum in &transfer_data {
            let initated_by = user_map
                .get(&transfer_datum.initated_by)
                .map(|u| u.clone())
                .ok_or_else(|| {
                    DatabaseError::new(
                        ErrorCode::BusinessProcessError,
                        Some("Unable to load initiating user".to_string()),
                    )
                })?;
            let accepted_by = if let Some(accepted_by) = transfer_datum.accepted_by {
                Some(
                    user_map
                        .get(&accepted_by)
                        .map(|u| u.clone())
                        .ok_or_else(|| {
                            DatabaseError::new(
                                ErrorCode::BusinessProcessError,
                                Some("Unable to load accepting user".to_string()),
                            )
                        })?,
                )
            } else {
                None
            };

            let cancelled_by = if let Some(cancelled_by) = transfer_datum.cancelled_by {
                Some(
                    user_map
                        .get(&cancelled_by)
                        .map(|u| u.clone())
                        .ok_or_else(|| {
                            DatabaseError::new(
                                ErrorCode::BusinessProcessError,
                                Some("Unable to load cancelled by user".to_string()),
                            )
                        })?,
                )
            } else {
                None
            };

            activity_items.push(ActivityItem::Transfer {
                transfer_id: transfer_datum.transfer_id,
                action: transfer_datum.action.clone(),
                status: transfer_datum.status,
                ticket_ids: transfer_datum.ticket_ids.clone(),
                destination_addresses: transfer_datum.destination_addresses.clone(),
                transfer_message_type: transfer_datum.transfer_message_type,
                initated_by,
                accepted_by,
                cancelled_by,
                occurred_at: transfer_datum.occurred_at,
            });
        }
        Ok(activity_items)
    }

    fn load_check_ins(
        order_id: Option<Uuid>,
        event_id: Option<Uuid>,
        user_id: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<Vec<ActivityItem>, DatabaseError> {
        use schema::*;
        if order_id.is_none() && event_id.is_none() {
            return Err(DatabaseError::new(
                ErrorCode::BusinessProcessError,
                Some(
                    "Activity loading requires either order_id or event_id to be present"
                        .to_string(),
                ),
            ));
        }

        #[derive(Debug, Queryable, QueryableByName)]
        struct R {
            #[sql_type = "dUuid"]
            ticket_instance_id: Uuid,
            #[sql_type = "Nullable<dUuid>"]
            redeemed_for: Option<Uuid>,
            #[sql_type = "Nullable<dUuid>"]
            redeemed_by: Option<Uuid>,
            #[sql_type = "Nullable<Timestamp>"]
            occurred_at: Option<NaiveDateTime>,
        }

        let mut query = ticket_instances::table
            .inner_join(wallets::table.on(ticket_instances::wallet_id.eq(wallets::id)))
            .inner_join(assets::table.on(ticket_instances::asset_id.eq(assets::id)))
            .inner_join(ticket_types::table.on(assets::ticket_type_id.eq(ticket_types::id)))
            .left_join(
                order_items::table
                    .on(ticket_instances::order_item_id.eq(order_items::id.nullable())),
            )
            .left_join(orders::table.on(orders::id.eq(order_items::order_id)))
            .into_boxed();

        if let (Some(event_id), Some(user_id)) = (event_id, user_id) {
            query = query.filter(ticket_types::event_id.eq(event_id)).filter(
                wallets::user_id
                    .eq(user_id)
                    .or(ticket_instances::redeemed_by_user_id.eq(Some(user_id))),
            );
        } else if let Some(order_id) = order_id {
            query = query.filter(orders::id.eq(order_id));
        }

        let check_in_data: Vec<R> = query
            .select((
                ticket_instances::id,
                wallets::user_id,
                ticket_instances::redeemed_by_user_id,
                ticket_instances::redeemed_at,
            ))
            .distinct()
            .load(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Unable to load check ins for organization fan",
            )?;

        let mut user_ids = Vec::new();
        for check_in_datum in &check_in_data {
            if let Some(redeemed_by) = check_in_datum.redeemed_by {
                user_ids.push(redeemed_by);
            }
            if let Some(redeemed_for) = check_in_datum.redeemed_for {
                user_ids.push(redeemed_for);
            }
        }
        user_ids.sort();
        user_ids.dedup();
        let users = User::find_by_ids(&user_ids, conn)?;
        let mut user_map: HashMap<Uuid, UserActivityItem> = HashMap::new();
        for user in users {
            user_map.insert(user.id, user.into());
        }

        let mut activity_items: Vec<ActivityItem> = Vec::new();
        for check_in_datum in check_in_data {
            if let (Some(redeemed_by), Some(redeemed_for), Some(occurred_at)) = (
                check_in_datum.redeemed_by,
                check_in_datum.redeemed_for,
                check_in_datum.occurred_at,
            ) {
                let redeemed_by =
                    user_map
                        .get(&redeemed_by)
                        .map(|u| u.clone())
                        .ok_or_else(|| {
                            DatabaseError::new(
                                ErrorCode::BusinessProcessError,
                                Some("Unable to load redeemed by user".to_string()),
                            )
                        })?;
                let redeemed_for =
                    user_map
                        .get(&redeemed_for)
                        .map(|u| u.clone())
                        .ok_or_else(|| {
                            DatabaseError::new(
                                ErrorCode::BusinessProcessError,
                                Some("Unable to load redeemed for user".to_string()),
                            )
                        })?;

                activity_items.push(ActivityItem::CheckIn {
                    ticket_instance_id: check_in_datum.ticket_instance_id,
                    redeemed_for,
                    occurred_at,
                    redeemed_by,
                });
            }
        }
        Ok(activity_items)
    }

    fn load_refunds(
        order_id: Option<Uuid>,
        event_id: Option<Uuid>,
        user_id: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<Vec<ActivityItem>, DatabaseError> {
        use schema::*;
        if order_id.is_none() && event_id.is_none() {
            return Err(DatabaseError::new(
                ErrorCode::BusinessProcessError,
                Some(
                    "Activity loading requires either order_id or event_id to be present"
                        .to_string(),
                ),
            ));
        }

        #[derive(Debug, Queryable, QueryableByName)]
        struct R {
            #[sql_type = "dUuid"]
            refund_id: Uuid,
            #[sql_type = "BigInt"]
            total_in_cents: i64,
            #[sql_type = "Nullable<Text>"]
            reason: Option<String>,
            #[sql_type = "dUuid"]
            refunded_by: Uuid,
            #[sql_type = "Timestamp"]
            occurred_at: NaiveDateTime,
        }

        let mut query = refunds::table
            .inner_join(refund_items::table.on(refund_items::refund_id.eq(refunds::id)))
            .inner_join(order_items::table.on(refund_items::order_item_id.eq(order_items::id)))
            .inner_join(orders::table.on(order_items::order_id.eq(orders::id)))
            .into_boxed();

        if let (Some(event_id), Some(user_id)) = (event_id, user_id) {
            query = query
                .filter(order_items::event_id.eq(Some(event_id)))
                .filter(
                    refunds::user_id
                        .eq(user_id)
                        .or(orders::on_behalf_of_user_id.eq(Some(user_id)))
                        .or(orders::on_behalf_of_user_id
                            .is_null()
                            .and(orders::user_id.eq(user_id))),
                );
        } else if let Some(order_id) = order_id {
            query = query.filter(orders::id.eq(order_id));
        }

        let refund_data: Vec<R> = query
            .group_by((
                refunds::id,
                refunds::reason,
                refunds::user_id,
                refunds::created_at,
            ))
            .select((
                refunds::id,
                sql("CAST(sum(refund_items.amount) as BigInt)"),
                refunds::reason,
                refunds::user_id,
                refunds::created_at,
            ))
            .distinct()
            .load(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Unable to load refund data for organization fan",
            )?;

        let mut user_ids = Vec::new();
        for refund_datum in &refund_data {
            user_ids.push(refund_datum.refunded_by);
        }
        user_ids.sort();
        user_ids.dedup();
        let users = User::find_by_ids(&user_ids, conn)?;
        let mut user_map: HashMap<Uuid, UserActivityItem> = HashMap::new();
        for user in users {
            user_map.insert(user.id, user.into());
        }

        let mut activity_items: Vec<ActivityItem> = Vec::new();
        for refund_datum in &refund_data {
            let refund_items: Vec<RefundActivityItem> = refunds::table
                .inner_join(refund_items::table.on(refund_items::refund_id.eq(refunds::id)))
                .inner_join(order_items::table.on(refund_items::order_item_id.eq(order_items::id)))
                .left_join(
                    ticket_types::table
                        .on(order_items::ticket_type_id.eq(ticket_types::id.nullable())),
                )
                .filter(refunds::id.eq(refund_datum.refund_id))
                .select((
                    refund_items::id,
                    refund_items::amount,
                    refund_items::quantity,
                    order_items::item_type,
                    sql("ticket_types.name"),
                ))
                .load(conn)
                .to_db_error(
                    ErrorCode::QueryError,
                    "Unable to load refund data for organization fan",
                )?;

            let refunded_by = user_map
                .get(&refund_datum.refunded_by)
                .map(|u| u.clone())
                .ok_or_else(|| {
                    DatabaseError::new(
                        ErrorCode::BusinessProcessError,
                        Some("Unable to load refunded by user".to_string()),
                    )
                })?;

            activity_items.push(ActivityItem::Refund {
                refund_id: refund_datum.refund_id,
                reason: refund_datum.reason.clone(),
                total_in_cents: refund_datum.total_in_cents,
                occurred_at: refund_datum.occurred_at,
                refund_items,
                refunded_by,
            });
        }
        Ok(activity_items)
    }

    fn load_notes(
        order_id: Option<Uuid>,
        event_id: Option<Uuid>,
        user_id: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<Vec<ActivityItem>, DatabaseError> {
        use schema::*;
        if order_id.is_none() && event_id.is_none() || (event_id.is_some() && user_id.is_none()) {
            return Err(DatabaseError::new(
                ErrorCode::BusinessProcessError,
                Some(
                    "Activity loading requires either order_id or event_id to be present"
                        .to_string(),
                ),
            ));
        }

        #[derive(Debug, Queryable, QueryableByName)]
        struct R {
            #[sql_type = "dUuid"]
            note_id: Uuid,
            #[sql_type = "dUuid"]
            order_id: Uuid,
            #[sql_type = "dUuid"]
            created_by: Uuid,
            #[sql_type = "Text"]
            note: String,
            #[sql_type = "Timestamp"]
            occurred_at: NaiveDateTime,
        }

        let mut query = orders::table
            .inner_join(order_items::table.on(order_items::order_id.eq(orders::id)))
            .inner_join(events::table.on(order_items::event_id.eq(events::id.nullable())))
            .inner_join(
                notes::table.on(notes::main_id
                    .eq(orders::id)
                    .and(notes::main_table.eq(Tables::Orders))),
            )
            .into_boxed();

        if let (Some(event_id), Some(user_id)) = (event_id, user_id) {
            query = query.filter(events::id.eq(event_id)).filter(
                orders::on_behalf_of_user_id
                    .eq(Some(user_id))
                    .or(orders::on_behalf_of_user_id
                        .is_null()
                        .and(orders::user_id.eq(user_id)))
                    .or(notes::created_by.eq(user_id)),
            );
        } else if let Some(order_id) = order_id {
            query = query.filter(orders::id.eq(order_id));
        }

        let note_data: Vec<R> = query
            .select((
                notes::id,
                notes::main_id,
                notes::created_by,
                notes::note,
                notes::created_at,
            ))
            .distinct()
            .load(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Unable to load notes for organization fan",
            )?;

        let mut user_ids = Vec::new();
        for note_datum in &note_data {
            user_ids.push(note_datum.created_by);
        }
        user_ids.sort();
        user_ids.dedup();
        let users = User::find_by_ids(&user_ids, conn)?;
        let mut user_map: HashMap<Uuid, UserActivityItem> = HashMap::new();
        for user in users {
            user_map.insert(user.id, user.into());
        }

        let mut activity_items: Vec<ActivityItem> = Vec::new();
        for note_datum in &note_data {
            let created_by = user_map
                .get(&note_datum.created_by)
                .map(|u| u.clone())
                .ok_or_else(|| {
                    DatabaseError::new(
                        ErrorCode::BusinessProcessError,
                        Some("Unable to load recorded by user".to_string()),
                    )
                })?;

            activity_items.push(ActivityItem::Note {
                note_id: note_datum.note_id,
                order_id: note_datum.order_id,
                note: note_datum.note.clone(),
                occurred_at: note_datum.occurred_at,
                created_by,
            });
        }
        Ok(activity_items)
    }

    pub fn occurred_at(&self) -> NaiveDateTime {
        match *self {
            ActivityItem::Purchase { occurred_at, .. } => occurred_at,
            ActivityItem::Transfer { occurred_at, .. } => occurred_at,
            ActivityItem::CheckIn { occurred_at, .. } => occurred_at,
            ActivityItem::Refund { occurred_at, .. } => occurred_at,
            ActivityItem::Note { occurred_at, .. } => occurred_at,
        }
    }
}

impl From<User> for UserActivityItem {
    fn from(user: User) -> Self {
        UserActivityItem {
            id: user.id,
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            full_name: user.full_name(),
        }
    }
}
