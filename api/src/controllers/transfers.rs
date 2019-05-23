use actix_web::{http::StatusCode, HttpResponse, Path, Query, State};
use auth::user::User;
use bigneon_db::models::{User as DbUser, *};
use chrono::prelude::*;
use communications::{mailers, smsers};
use db::Connection;
use errors::*;
use helpers::application;
use models::{OptionalPathParameters, PathParameters, WebPayload};
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize, Clone)]
pub struct TransferFilters {
    source_or_destination: Option<String>,
    start_utc: Option<NaiveDateTime>,
    end_utc: Option<NaiveDateTime>,
}

pub fn index(
    (connection, paging_query, filter_query, path, auth_user): (
        Connection,
        Query<PagingParameters>,
        Query<TransferFilters>,
        Path<OptionalPathParameters>,
        User,
    ),
) -> Result<WebPayload<DisplayTransfer>, BigNeonError> {
    let connection = connection.get();
    let mut lookup_user_id = auth_user.id();

    match path.id {
        Some(order_id) => {
            let order = Order::find(order_id, connection)?;
            lookup_user_id = order.on_behalf_of_user_id.unwrap_or(order.user_id);
            if order.on_behalf_of_user_id != Some(auth_user.id())
                && !(order.on_behalf_of_user_id.is_none() && order.user_id == auth_user.id())
            {
                auth_user.requires_scope_for_order(Scopes::TransferRead, &order, connection)?;
            } else {
                auth_user.requires_scope(Scopes::TransferReadOwn)?;
            }
        }
        None => {
            auth_user.requires_scope(Scopes::TransferReadOwn)?;
        }
    }

    let source_or_destination = match filter_query
        .source_or_destination
        .clone()
        .unwrap_or("source".to_string())
        .as_str()
    {
        "source" => SourceOrDestination::Source,
        _ => SourceOrDestination::Destination,
    };

    let mut payload = Transfer::find_for_user_for_display(
        lookup_user_id,
        path.id,
        source_or_destination,
        filter_query.start_utc,
        filter_query.end_utc,
        Some(paging_query.limit()),
        Some(paging_query.page()),
        connection,
    )?;
    payload
        .paging
        .tags
        .insert("start_utc".to_string(), json!(filter_query.start_utc));
    payload
        .paging
        .tags
        .insert("end_utc".to_string(), json!(filter_query.end_utc));
    payload.paging.tags.insert(
        "source_or_destination".to_string(),
        json!(filter_query.source_or_destination),
    );
    Ok(WebPayload::new(StatusCode::OK, payload))
}

pub fn cancel(
    (connection, path, auth_user, state): (Connection, Path<PathParameters>, User, State<AppState>),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let transfer = Transfer::find(path.id, connection)?;

    if transfer.source_user_id != auth_user.id() {
        let mut valid_for_cancelling = false;
        let orders = transfer.orders(connection)?;

        for order in orders {
            valid_for_cancelling =
                auth_user.has_scope_for_order(Scopes::TransferCancel, &order, connection)?;
        }

        if !valid_for_cancelling {
            return application::forbidden("You do not have access to this transfer");
        }
    } else {
        auth_user.requires_scope(Scopes::TransferCancelOwn)?;
    }

    let updated_transfer = transfer.cancel(auth_user.id(), None, connection)?;
    let source_user = DbUser::find(transfer.source_user_id, connection)?;

    if let Some(transfer_message_type) = transfer.transfer_message_type {
        if let Some(transfer_address) = &transfer.transfer_address {
            match transfer_message_type {
                TransferMessageType::Phone => {
                    smsers::tickets::transfer_cancelled(
                        &state.config,
                        transfer_address.clone(),
                        &auth_user.user,
                        connection,
                    )?;
                }
                TransferMessageType::Email => {
                    mailers::tickets::transfer_cancelled(
                        &state.config,
                        transfer_address.clone(),
                        &source_user,
                        connection,
                    )?;
                }
            }
        }
    }

    if let Some(source_email) = source_user.email.clone() {
        mailers::tickets::transfer_cancelled_receipt(
            &state.config,
            source_email,
            &source_user,
            transfer.created_at,
            &transfer
                .transfer_tickets(connection)?
                .iter()
                .map(|tt| tt.ticket_instance_id)
                .collect::<Vec<Uuid>>(),
            connection,
        )?;
    }

    Ok(HttpResponse::Ok().json(&updated_transfer.for_display(connection)?))
}