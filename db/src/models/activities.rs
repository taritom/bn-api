use chrono::prelude::*;
use diesel::expression::sql_literal::sql;
use diesel::pg::types::sql_types::Array;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Bool, Nullable, Text, Timestamp, Uuid as dUuid};
use itertools::Itertools;
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EventActivityItem {
    pub event_id: Uuid,
    pub name: String,
    pub code: Option<String>,
    pub code_discount_in_cents: Option<i64>,
    pub code_type: Option<String>,
    pub quantity: i64,
    pub total_in_cents: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Queryable)]
pub struct RefundActivityItem {
    pub id: Uuid,
    pub amount: i64,
    pub quantity: i64,
    pub item_type: OrderItemTypes,
    pub description: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UserActivityItem {
    pub id: Uuid,
    pub full_name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
        ticket_numbers: Vec<String>,
        destination_addresses: Option<String>,
        transfer_message_type: Option<TransferMessageType>,
        initiated_by: UserActivityItem,
        accepted_by: Option<UserActivityItem>,
        cancelled_by: Option<UserActivityItem>,
        occurred_at: NaiveDateTime,
        order_id: Option<Uuid>,
        order_number: Option<String>,
        transfer_key: Uuid,
        eligible_for_cancelling: bool,
    },
    CheckIn {
        ticket_instance_id: Uuid,
        ticket_number: String,
        redeemed_for: UserActivityItem,
        redeemed_by: UserActivityItem,
        occurred_at: NaiveDateTime,
        order_id: Option<Uuid>,
        order_number: Option<String>,
    },
    Refund {
        refund_id: Uuid,
        order_id: Uuid,
        order_number: String,
        refund_items: Vec<RefundActivityItem>,
        reason: Option<String>,
        total_in_cents: i64,
        manual_override: bool,
        refunded_by: UserActivityItem,
        occurred_at: NaiveDateTime,
    },
    Note {
        note_id: Uuid,
        order_id: Uuid,
        order_number: String,
        created_by: UserActivityItem,
        note: String,
        occurred_at: NaiveDateTime,
    },
}

impl ActivityItem {
    pub fn load_for_event(
        event_id: Uuid,
        user_id: Uuid,
        activity_type: Option<ActivityType>,
        conn: &PgConnection,
    ) -> Result<Vec<ActivityItem>, DatabaseError> {
        let mut activity_items: Vec<ActivityItem> = Vec::new();
        if activity_type.is_none() || activity_type == Some(ActivityType::Purchase) {
            activity_items.append(&mut ActivityItem::load_purchases(
                None,
                Some(event_id),
                Some(user_id),
                conn,
            )?);
        }
        if activity_type.is_none() || activity_type == Some(ActivityType::Transfer) {
            activity_items.append(&mut ActivityItem::load_transfers(
                None,
                Some(event_id),
                Some(user_id),
                false,
                conn,
            )?);
        }

        if activity_type.is_none() || activity_type == Some(ActivityType::CheckIn) {
            activity_items.append(&mut ActivityItem::load_check_ins(
                None,
                Some(event_id),
                Some(user_id),
                conn,
            )?);
        }

        if activity_type.is_none() || activity_type == Some(ActivityType::Refund) {
            activity_items.append(&mut ActivityItem::load_refunds(
                None,
                Some(event_id),
                Some(user_id),
                conn,
            )?);
        }
        if activity_type.is_none() || activity_type == Some(ActivityType::Note) {
            activity_items.append(&mut ActivityItem::load_notes(
                None,
                Some(event_id),
                Some(user_id),
                conn,
            )?);
        }
        activity_items.sort_by_key(|activity| Reverse(activity.occurred_at()));
        Ok(activity_items)
    }

    pub fn load_for_order(order: &Order, conn: &PgConnection) -> Result<Vec<ActivityItem>, DatabaseError> {
        let mut activity_items: Vec<ActivityItem> = Vec::new();
        activity_items.append(&mut ActivityItem::load_purchases(Some(order.id), None, None, conn)?);
        activity_items.append(&mut ActivityItem::load_transfers(
            Some(order.id),
            None,
            None,
            false,
            conn,
        )?);
        activity_items.append(&mut ActivityItem::load_check_ins(Some(order.id), None, None, conn)?);
        activity_items.append(&mut ActivityItem::load_refunds(Some(order.id), None, None, conn)?);
        activity_items.append(&mut ActivityItem::load_notes(Some(order.id), None, None, conn)?);
        activity_items.sort_by_key(|activity| Reverse(activity.occurred_at()));
        Ok(activity_items)
    }

    fn load_purchases(
        order_id: Option<Uuid>,
        event_id: Option<Uuid>,
        user_id: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<Vec<ActivityItem>, DatabaseError> {
        if order_id.is_none() && event_id.is_none() {
            return Err(DatabaseError::new(
                ErrorCode::BusinessProcessError,
                Some("Activity loading requires either order_id or event_id to be present".to_string()),
            ));
        }

        #[derive(Debug, Queryable, QueryableByName)]
        struct R {
            #[sql_type = "dUuid"]
            order_id: Uuid,
            #[sql_type = "Timestamp"]
            occurred_at: NaiveDateTime,
            #[sql_type = "Nullable<Timestamp>"]
            paid_at: Option<NaiveDateTime>,
            #[sql_type = "dUuid"]
            purchased_by: Uuid,
            #[sql_type = "dUuid"]
            user_id: Uuid,
            #[sql_type = "BigInt"]
            ticket_quantity: i64,
            #[sql_type = "BigInt"]
            total_in_cents: i64,
            #[sql_type = "dUuid"]
            event_id: Uuid,
            #[sql_type = "Text"]
            event_name: String,
            #[sql_type = "Nullable<Text>"]
            code: Option<String>,
            #[sql_type = "Nullable<BigInt>"]
            code_discount_in_cents: Option<i64>,
            #[sql_type = "Nullable<Text>"]
            code_type: Option<String>,
        }
        let mut query = sql_query(
            r#"
        SELECT DISTINCT
            o.id as order_id,
            o.created_at as occurred_at,
            o.paid_at AS paid_at,
            o.user_id as purchased_by,
            COALESCE(o.on_behalf_of_user_id, o.user_id) as user_id,
            CAST(
                COALESCE(SUM(oi.quantity) FILTER (WHERE oi.item_type = 'Tickets'), 0) as BigInt
            ) as ticket_quantity,
            CAST(COALESCE(SUM(oi.unit_price_in_cents * oi.quantity), 0) as BigInt) as total_in_cents,
            e.id as event_id,
            e.name as event_name,
            COALESCE(h.redemption_code, c.redemption_code) as code,
            COALESCE(h.discount_in_cents, c.discount_in_cents) as code_discount_in_cents,
            COALESCE(h.hold_type, c.code_type) as code_type
        FROM orders o
        JOIN order_items oi ON oi.order_id = o.id
        JOIN events e ON e.id = oi.event_id
        LEFT JOIN order_items parent_oi ON parent_oi.id = oi.parent_id
        -- Associate children with parent's code or hold for grouping
        LEFT JOIN codes c on c.id = COALESCE(parent_oi.code_id, oi.code_id)
        LEFT JOIN holds h on h.id = COALESCE(parent_oi.hold_id, oi.hold_id)
        WHERE
            o.status = 'Paid'
        "#,
        )
        .into_boxed();

        if let (Some(event_id), Some(user_id)) = (event_id, user_id) {
            query = query.sql(format!(" and e.id = $1 ")).bind::<dUuid, _>(event_id);
            query = query
                .sql(format!(" and COALESCE(o.on_behalf_of_user_id, o.user_id) = $2 "))
                .bind::<dUuid, _>(user_id);
        } else if let Some(order_id) = order_id {
            query = query.sql(format!(" AND o.id = $1 ")).bind::<dUuid, _>(order_id);
        }

        let order_data: Vec<R> = query
            .sql(
                "
                GROUP BY
                    o.id,
                    o.created_at,
                    o.user_id,
                    o.on_behalf_of_user_id,
                    e.id,
                    e.name,
                    h.hold_type,
                    c.code_type,
                    h.redemption_code,
                    c.redemption_code,
                    h.discount_in_cents,
                    c.discount_in_cents
                ORDER BY
                    o.created_at desc
            ",
            )
            .load(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Unable to load purchase event data for organization fan",
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

        let mut order_activities: HashMap<Uuid, ActivityItem> = HashMap::new();
        for order_datum in &order_data {
            let event_activity_item = EventActivityItem {
                event_id: order_datum.event_id,
                name: order_datum.event_name.clone(),
                code: order_datum.code.clone(),
                code_discount_in_cents: order_datum.code_discount_in_cents,
                code_type: order_datum.code_type.clone(),
                quantity: order_datum.ticket_quantity,
                total_in_cents: order_datum.total_in_cents,
            };

            let purchased_by = user_map.get(&order_datum.purchased_by).ok_or_else(|| {
                DatabaseError::new(
                    ErrorCode::BusinessProcessError,
                    Some("Order can't load purchasing user".to_string()),
                )
            })?;
            let user = user_map.get(&order_datum.user_id).ok_or_else(|| {
                DatabaseError::new(
                    ErrorCode::BusinessProcessError,
                    Some("Order can't load user".to_string()),
                )
            })?;

            order_activities
                .entry(order_datum.order_id)
                .and_modify(|activity_item| {
                    if let ActivityItem::Purchase {
                        ref mut ticket_quantity,
                        ref mut total_in_cents,
                        ref mut events,
                        ..
                    } = activity_item
                    {
                        *ticket_quantity += order_datum.ticket_quantity;
                        *total_in_cents += order_datum.total_in_cents;
                        events.push(event_activity_item.clone());
                    }
                })
                .or_insert_with(|| ActivityItem::Purchase {
                    order_id: order_datum.order_id,
                    order_number: Order::parse_order_number(order_datum.order_id),
                    ticket_quantity: order_datum.ticket_quantity,
                    events: vec![event_activity_item],
                    occurred_at: order_datum.occurred_at,
                    purchased_by: purchased_by.clone(),
                    total_in_cents: order_datum.total_in_cents,
                    user: user.clone(),
                });
        }

        Ok(order_activities.into_iter().map(|(_, v)| v).collect())
    }

    pub(crate) fn load_transfers(
        order_id: Option<Uuid>,
        event_id: Option<Uuid>,
        user_id: Option<Uuid>,
        only_source_user: bool,
        conn: &PgConnection,
    ) -> Result<Vec<ActivityItem>, DatabaseError> {
        use schema::*;
        if order_id.is_none() && event_id.is_none() {
            return Err(DatabaseError::new(
                ErrorCode::BusinessProcessError,
                Some("Activity loading requires either order_id or event_id to be present".to_string()),
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
            #[sql_type = "Nullable<dUuid>"]
            order_id: Option<Uuid>,
            #[sql_type = "dUuid"]
            transfer_key: Uuid,
            #[sql_type = "Bool"]
            eligible_for_cancelling: bool,
        }

        let mut query = transfers::table
            .inner_join(transfer_tickets::table.on(transfer_tickets::transfer_id.eq(transfers::id)))
            .inner_join(ticket_instances::table.on(transfer_tickets::ticket_instance_id.eq(ticket_instances::id)))
            .inner_join(assets::table.on(ticket_instances::asset_id.eq(assets::id)))
            .inner_join(ticket_types::table.on(assets::ticket_type_id.eq(ticket_types::id)))
            .left_join(order_items::table.on(ticket_instances::order_item_id.eq(order_items::id.nullable())))
            .left_join(orders::table.on(orders::id.eq(order_items::order_id)))
            .inner_join(
                domain_events::table.on(domain_events::main_table
                    .eq(Tables::Transfers)
                    .and(domain_events::main_id.eq(transfers::id.nullable()))),
            )
            .filter(domain_events::event_type.eq_any(vec![
                DomainEventTypes::TransferTicketCancelled,
                DomainEventTypes::TransferTicketCompleted,
                DomainEventTypes::TransferTicketStarted,
            ]))
            .into_boxed();

        if let (Some(event_id), Some(user_id)) = (event_id, user_id) {
            query = query.filter(ticket_types::event_id.eq(event_id));

            if only_source_user {
                query = query.filter(transfers::source_user_id.eq(user_id));
            } else {
                query = query.filter(
                    transfers::source_user_id.eq(user_id).or(transfers::destination_user_id
                        .eq(user_id)
                        .and(domain_events::event_type.eq(DomainEventTypes::TransferTicketCompleted))),
                );
            }
        } else if let Some(order_id) = order_id {
            query = query.filter(orders::id.eq(order_id));
        }

        let transfer_data: Vec<R> = query
            .order_by(domain_events::created_at.desc())
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
                orders::id,
                transfers::transfer_key,
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
                orders::id.nullable(),
                transfers::transfer_key,
                sql::<Bool>(
                    "
                    transfers.status <> 'Cancelled'
                    AND (SELECT NOT EXISTS (
                        SELECT 1
                        FROM transfer_tickets tt
                        JOIN ticket_instances ti ON tt.ticket_instance_id = ti.id
                        JOIN transfers t ON tt.transfer_id = t.id
                        LEFT JOIN transfer_tickets tt2 ON tt2.ticket_instance_id = tt.ticket_instance_id AND tt.id <> tt2.id
                        LEFT JOIN transfers t2 ON tt2.transfer_id = t2.id
                        WHERE tt.transfer_id = transfers.id
                        AND (
                            (t2.created_at >= t.created_at AND t2.status IN ('Pending', 'Completed'))
                            OR ti.redeemed_at IS NOT NULL
                        )
                    )
                )",
                ),
            ))
            .distinct()
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load transfers for organization fan")?;

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
                Some(user_map.get(&accepted_by).map(|u| u.clone()).ok_or_else(|| {
                    DatabaseError::new(
                        ErrorCode::BusinessProcessError,
                        Some("Unable to load accepting user".to_string()),
                    )
                })?)
            } else {
                None
            };

            let cancelled_by = if let Some(cancelled_by) = transfer_datum.cancelled_by {
                Some(user_map.get(&cancelled_by).map(|u| u.clone()).ok_or_else(|| {
                    DatabaseError::new(
                        ErrorCode::BusinessProcessError,
                        Some("Unable to load cancelled by user".to_string()),
                    )
                })?)
            } else {
                None
            };

            activity_items.push(ActivityItem::Transfer {
                transfer_id: transfer_datum.transfer_id,
                action: transfer_datum.action.clone(),
                status: transfer_datum.status,
                ticket_ids: transfer_datum.ticket_ids.clone(),
                ticket_numbers: transfer_datum
                    .ticket_ids
                    .iter()
                    .map(|id| TicketInstance::parse_ticket_number(*id))
                    .collect_vec(),
                destination_addresses: transfer_datum.destination_addresses.clone(),
                transfer_message_type: transfer_datum.transfer_message_type,
                initiated_by: initated_by,
                accepted_by,
                cancelled_by,
                occurred_at: transfer_datum.occurred_at,
                order_number: transfer_datum.order_id.map(|id| Order::parse_order_number(id)),
                order_id: transfer_datum.order_id,
                transfer_key: transfer_datum.transfer_key,
                eligible_for_cancelling: transfer_datum.eligible_for_cancelling,
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
                Some("Activity loading requires either order_id or event_id to be present".to_string()),
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
            #[sql_type = "Nullable<dUuid>"]
            order_id: Option<Uuid>,
        }

        let mut query = ticket_instances::table
            .inner_join(wallets::table.on(ticket_instances::wallet_id.eq(wallets::id)))
            .inner_join(assets::table.on(ticket_instances::asset_id.eq(assets::id)))
            .inner_join(ticket_types::table.on(assets::ticket_type_id.eq(ticket_types::id)))
            .left_join(order_items::table.on(ticket_instances::order_item_id.eq(order_items::id.nullable())))
            .left_join(orders::table.on(orders::id.eq(order_items::order_id)))
            .into_boxed();

        if let (Some(event_id), Some(user_id)) = (event_id, user_id) {
            query = query
                .filter(ticket_types::event_id.eq(event_id))
                .filter(wallets::user_id.eq(user_id));
        } else if let Some(order_id) = order_id {
            query = query.filter(orders::id.eq(order_id));
        }

        let check_in_data: Vec<R> = query
            .select((
                ticket_instances::id,
                wallets::user_id,
                ticket_instances::redeemed_by_user_id,
                ticket_instances::redeemed_at,
                orders::id.nullable(),
            ))
            .distinct()
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load check ins for organization fan")?;

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
                let redeemed_by = user_map.get(&redeemed_by).map(|u| u.clone()).ok_or_else(|| {
                    DatabaseError::new(
                        ErrorCode::BusinessProcessError,
                        Some("Unable to load redeemed by user".to_string()),
                    )
                })?;
                let redeemed_for = user_map.get(&redeemed_for).map(|u| u.clone()).ok_or_else(|| {
                    DatabaseError::new(
                        ErrorCode::BusinessProcessError,
                        Some("Unable to load redeemed for user".to_string()),
                    )
                })?;

                activity_items.push(ActivityItem::CheckIn {
                    ticket_instance_id: check_in_datum.ticket_instance_id,
                    ticket_number: TicketInstance::parse_ticket_number(check_in_datum.ticket_instance_id),
                    redeemed_for,
                    occurred_at,
                    redeemed_by,
                    order_id: check_in_datum.order_id,
                    order_number: check_in_datum.order_id.map(|id| Order::parse_order_number(id)),
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
                Some("Activity loading requires either order_id or event_id to be present".to_string()),
            ));
        }

        #[derive(Debug, Queryable, QueryableByName)]
        struct R {
            #[sql_type = "dUuid"]
            refund_id: Uuid,
            #[sql_type = "dUuid"]
            order_id: Uuid,
            #[sql_type = "BigInt"]
            total_in_cents: i64,
            #[sql_type = "Nullable<Text>"]
            reason: Option<String>,
            #[sql_type = "Bool"]
            manual_override: bool,
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
            query = query.filter(order_items::event_id.eq(Some(event_id))).filter(
                orders::on_behalf_of_user_id
                    .eq(Some(user_id))
                    .or(orders::on_behalf_of_user_id.is_null().and(orders::user_id.eq(user_id))),
            );
        } else if let Some(order_id) = order_id {
            query = query.filter(orders::id.eq(order_id));
        }

        let refund_data: Vec<R> = query
            .group_by((
                refunds::id,
                refunds::order_id,
                refunds::reason,
                refunds::user_id,
                refunds::created_at,
            ))
            .select((
                refunds::id,
                refunds::order_id,
                sql("CAST(sum(refund_items.amount) as BigInt)"),
                refunds::reason,
                refunds::manual_override,
                refunds::user_id,
                refunds::created_at,
            ))
            .distinct()
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load refund data for organization fan")?;

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

        #[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Queryable)]
        pub struct RI {
            pub refund_id: Uuid,
            pub id: Uuid,
            pub amount: i64,
            pub quantity: i64,
            pub order_item_id: Uuid,
            pub order_id: Uuid,
            pub item_type: OrderItemTypes,
            pub ticket_type_id: Option<Uuid>,
            pub event_id: Option<Uuid>,
            pub order_item_quantity: i64,
            pub unit_price_in_cents: i64,
            pub created_at: NaiveDateTime,
            pub updated_at: NaiveDateTime,
            pub ticket_pricing_id: Option<Uuid>,
            pub fee_schedule_range_id: Option<Uuid>,
            pub parent_id: Option<Uuid>,
            pub hold_id: Option<Uuid>,
            pub code_id: Option<Uuid>,
            pub company_fee_in_cents: i64,
            pub client_fee_in_cents: i64,
            pub refunded_quantity: i64,
        };

        let refund_ids: Vec<Uuid> = refund_data.iter().map(|r| r.refund_id).collect();
        let items: Vec<RI> = refunds::table
            .inner_join(refund_items::table.on(refund_items::refund_id.eq(refunds::id)))
            .inner_join(order_items::table.on(refund_items::order_item_id.eq(order_items::id)))
            .filter(refunds::id.eq_any(refund_ids))
            .select((
                refund_items::refund_id,
                refund_items::id,
                refund_items::amount,
                refund_items::quantity,
                order_items::id,
                order_items::order_id,
                order_items::item_type,
                order_items::ticket_type_id,
                order_items::event_id,
                order_items::quantity,
                order_items::unit_price_in_cents,
                order_items::created_at,
                order_items::updated_at,
                order_items::ticket_pricing_id,
                order_items::fee_schedule_range_id,
                order_items::parent_id,
                order_items::hold_id,
                order_items::code_id,
                order_items::company_fee_in_cents,
                order_items::client_fee_in_cents,
                order_items::refunded_quantity,
            ))
            .order_by(refunds::id)
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load refund data for organization fan")?;

        for refund_datum in refund_data {
            let mut refund_items = vec![];
            let refunded_by = user_map
                .get(&refund_datum.refunded_by)
                .map(|u| u.clone())
                .ok_or_else(|| {
                    DatabaseError::new(
                        ErrorCode::BusinessProcessError,
                        Some("Unable to load refunded by user".to_string()),
                    )
                })?;
            for item in items.iter().filter(|i| i.refund_id == refund_datum.refund_id) {
                let order_item = OrderItem {
                    id: item.order_item_id,
                    order_id: item.order_id,
                    item_type: item.item_type,
                    ticket_type_id: item.ticket_type_id,
                    event_id: item.event_id,
                    quantity: item.order_item_quantity,
                    unit_price_in_cents: item.unit_price_in_cents,
                    created_at: item.created_at,
                    updated_at: item.updated_at,
                    ticket_pricing_id: item.ticket_pricing_id,
                    fee_schedule_range_id: item.fee_schedule_range_id,
                    parent_id: item.parent_id,
                    hold_id: item.hold_id,
                    code_id: item.code_id,
                    company_fee_in_cents: item.company_fee_in_cents,
                    client_fee_in_cents: item.client_fee_in_cents,
                    refunded_quantity: item.refunded_quantity,
                };
                refund_items.push(RefundActivityItem {
                    id: item.id,
                    amount: item.amount,
                    quantity: item.quantity,
                    item_type: order_item.item_type,
                    description: order_item.description(conn)?,
                });
            }

            activity_items.push(ActivityItem::Refund {
                refund_id: refund_datum.refund_id,
                reason: refund_datum.reason.clone(),
                order_id: refund_datum.order_id,
                order_number: Order::parse_order_number(refund_datum.order_id),
                total_in_cents: refund_datum.total_in_cents,
                occurred_at: refund_datum.occurred_at,
                manual_override: refund_datum.manual_override,
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
                Some("Activity loading requires either order_id or event_id to be present".to_string()),
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
            .inner_join(notes::table.on(notes::main_table.eq(Tables::Orders).and(notes::main_id.eq(orders::id))))
            .into_boxed();

        if let (Some(event_id), Some(user_id)) = (event_id, user_id) {
            query = query.filter(events::id.eq(event_id)).filter(
                orders::on_behalf_of_user_id
                    .eq(Some(user_id))
                    .or(orders::on_behalf_of_user_id.is_null().and(orders::user_id.eq(user_id))),
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
            .to_db_error(ErrorCode::QueryError, "Unable to load notes for organization fan")?;

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
            let created_by = user_map.get(&note_datum.created_by).map(|u| u.clone()).ok_or_else(|| {
                DatabaseError::new(
                    ErrorCode::BusinessProcessError,
                    Some("Unable to load recorded by user".to_string()),
                )
            })?;

            activity_items.push(ActivityItem::Note {
                note_id: note_datum.note_id,
                order_id: note_datum.order_id,
                order_number: Order::parse_order_number(note_datum.order_id),
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
