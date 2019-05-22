use actix_web::{http::StatusCode, FromRequest, Path, Query};
use bigneon_api::controllers::transfers::{self, *};
use bigneon_api::errors::BigNeonError;
use bigneon_api::models::*;
use bigneon_db::prelude::*;
use bigneon_db::utils::dates;
use chrono::prelude::*;
use functional::base;
use serde_json::Value;
use std::collections::HashMap;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

#[cfg(test)]
mod index_tests {
    use super::*;
    #[test]
    fn index_org_member() {
        base::transfers::index(Roles::OrgMember, false, true);
    }
    #[test]
    fn index_admin() {
        base::transfers::index(Roles::Admin, false, true);
    }
    #[test]
    fn index_user() {
        base::transfers::index(Roles::User, false, false);
    }
    #[test]
    fn index_org_owner() {
        base::transfers::index(Roles::OrgOwner, false, true);
    }
    #[test]
    fn index_door_person() {
        base::transfers::index(Roles::DoorPerson, false, false);
    }
    #[test]
    fn index_promoter() {
        base::transfers::index(Roles::Promoter, false, true);
    }
    #[test]
    fn index_promoter_read_only() {
        base::transfers::index(Roles::PromoterReadOnly, false, true);
    }
    #[test]
    fn index_org_admin() {
        base::transfers::index(Roles::OrgAdmin, false, true);
    }
    #[test]
    fn index_box_office() {
        base::transfers::index(Roles::OrgBoxOffice, true, true);
    }
    #[test]
    fn index_owns_order_org_member() {
        base::transfers::index(Roles::OrgMember, true, true);
    }
    #[test]
    fn index_owns_order_admin() {
        base::transfers::index(Roles::Admin, true, true);
    }
    #[test]
    fn index_owns_order_user() {
        base::transfers::index(Roles::User, true, true);
    }
    #[test]
    fn index_owns_order_org_owner() {
        base::transfers::index(Roles::OrgOwner, true, true);
    }
    #[test]
    fn index_owns_order_door_person() {
        base::transfers::index(Roles::DoorPerson, true, true);
    }
    #[test]
    fn index_owns_order_promoter() {
        base::transfers::index(Roles::Promoter, true, true);
    }
    #[test]
    fn index_owns_order_promoter_read_only() {
        base::transfers::index(Roles::PromoterReadOnly, true, true);
    }
    #[test]
    fn index_owns_order_org_admin() {
        base::transfers::index(Roles::OrgAdmin, true, true);
    }
    #[test]
    fn index_owns_order_box_office() {
        base::transfers::index(Roles::OrgBoxOffice, true, true);
    }
}

#[cfg(test)]
mod cancel_tests {
    use super::*;
    #[test]
    fn cancel_org_member() {
        base::transfers::cancel(Roles::OrgMember, false, true);
    }
    #[test]
    fn cancel_admin() {
        base::transfers::cancel(Roles::Admin, false, true);
    }
    #[test]
    fn cancel_user() {
        base::transfers::cancel(Roles::User, false, false);
    }
    #[test]
    fn cancel_org_owner() {
        base::transfers::cancel(Roles::OrgOwner, false, true);
    }
    #[test]
    fn cancel_door_person() {
        base::transfers::cancel(Roles::DoorPerson, false, false);
    }
    #[test]
    fn cancel_promoter() {
        base::transfers::cancel(Roles::Promoter, false, true);
    }
    #[test]
    fn cancel_promoter_read_only() {
        base::transfers::cancel(Roles::PromoterReadOnly, false, false);
    }
    #[test]
    fn cancel_org_admin() {
        base::transfers::cancel(Roles::OrgAdmin, false, true);
    }
    #[test]
    fn cancel_box_office() {
        base::transfers::cancel(Roles::OrgBoxOffice, false, false);
    }
    #[test]
    fn cancel_owns_order_org_member() {
        base::transfers::cancel(Roles::OrgMember, true, true);
    }
    #[test]
    fn cancel_owns_order_admin() {
        base::transfers::cancel(Roles::Admin, true, true);
    }
    #[test]
    fn cancel_owns_order_user() {
        base::transfers::cancel(Roles::User, true, true);
    }
    #[test]
    fn cancel_owns_order_org_owner() {
        base::transfers::cancel(Roles::OrgOwner, true, true);
    }
    #[test]
    fn cancel_owns_order_door_person() {
        base::transfers::cancel(Roles::DoorPerson, true, true);
    }
    #[test]
    fn cancel_owns_order_promoter() {
        base::transfers::cancel(Roles::Promoter, true, true);
    }
    #[test]
    fn cancel_owns_order_promoter_read_only() {
        base::transfers::cancel(Roles::PromoterReadOnly, true, true);
    }
    #[test]
    fn cancel_owns_order_org_admin() {
        base::transfers::cancel(Roles::OrgAdmin, true, true);
    }
    #[test]
    fn cancel_owns_order_box_office() {
        base::transfers::cancel(Roles::OrgBoxOffice, true, true);
    }
}

#[test]
fn index() {
    let database = TestDatabase::new();
    let connection = database.connection.get();
    let user = database.create_user().finish();
    let user2 = database.create_user().finish();
    let user3 = database.create_user().finish();
    database
        .create_order()
        .for_user(&user)
        .quantity(1)
        .is_paid()
        .finish();
    database
        .create_order()
        .for_user(&user2)
        .quantity(1)
        .is_paid()
        .finish();
    let ticket = &TicketInstance::find_for_user(user.id, connection).unwrap()[0];
    let ticket2 = &TicketInstance::find_for_user(user2.id, connection).unwrap()[0];

    let transfer = Transfer::create(
        user.id,
        Uuid::new_v4(),
        dates::now().add_seconds(40).finish(),
        None,
        None,
    )
    .commit(&None, connection)
    .unwrap();
    transfer
        .add_transfer_ticket(ticket.id, user.id, &None, connection)
        .unwrap();
    transfer.update_associated_orders(connection).unwrap();
    transfer
        .update(
            TransferEditableAttributes {
                destination_user_id: Some(user3.id),
                ..Default::default()
            },
            connection,
        )
        .unwrap();
    let transfer2 = Transfer::create(
        user2.id,
        Uuid::new_v4(),
        dates::now().add_seconds(40).finish(),
        None,
        None,
    )
    .commit(&None, connection)
    .unwrap();
    transfer2
        .add_transfer_ticket(ticket2.id, user2.id, &None, connection)
        .unwrap();
    transfer2.update_associated_orders(connection).unwrap();
    transfer2
        .update(
            TransferEditableAttributes {
                destination_user_id: Some(user.id),
                ..Default::default()
            },
            connection,
        )
        .unwrap();

    // Outgoing
    let expected_transfers = Transfer::find_for_user_for_display(
        user.id,
        None,
        SourceOrDestination::Source,
        None,
        None,
        None,
        None,
        connection,
    )
    .unwrap()
    .data;
    assert_eq!(expected_transfers.len(), 1);
    assert_eq!(expected_transfers[0].ticket_ids, vec![ticket.id]);

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
    let test_request = TestRequest::create_with_uri("/transfers?source_or_destination=source");
    let paging_parameters = Query::<PagingParameters>::extract(&test_request.request).unwrap();
    let filter_parameters = Query::<TransferFilters>::extract(&test_request.request).unwrap();
    let mut path = Path::<OptionalPathParameters>::extract(&test_request.request).unwrap();
    path.id = None;
    let response: Result<WebPayload<DisplayTransfer>, BigNeonError> = transfers::index((
        database.connection.clone().into(),
        paging_parameters,
        filter_parameters,
        path,
        auth_user.clone(),
    ));

    let response = response.unwrap();
    let mut expected_tags: HashMap<String, Value> = HashMap::new();
    let no_datetime: Option<NaiveDateTime> = None;
    expected_tags.insert("source_or_destination".to_string(), json!("source"));
    expected_tags.insert("start_utc".to_string(), json!(no_datetime));
    expected_tags.insert("end_utc".to_string(), json!(no_datetime));
    let wrapped_expected_transfers = Payload {
        data: expected_transfers,
        paging: Paging {
            page: 0,
            limit: 100,
            sort: "".to_string(),
            dir: SortingDir::Asc,
            total: 1,
            tags: expected_tags,
        },
    };

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.payload(), &wrapped_expected_transfers);

    // Incoming
    let expected_transfers = Transfer::find_for_user_for_display(
        user.id,
        None,
        SourceOrDestination::Destination,
        None,
        None,
        None,
        None,
        connection,
    )
    .unwrap()
    .data;
    assert_eq!(expected_transfers.len(), 1);
    assert_eq!(expected_transfers[0].ticket_ids, vec![ticket2.id]);

    let test_request = TestRequest::create_with_uri("/transfers?source_or_destination=destination");
    let paging_parameters = Query::<PagingParameters>::extract(&test_request.request).unwrap();
    let filter_parameters = Query::<TransferFilters>::extract(&test_request.request).unwrap();
    let mut path = Path::<OptionalPathParameters>::extract(&test_request.request).unwrap();
    path.id = None;
    let response: Result<WebPayload<DisplayTransfer>, BigNeonError> = transfers::index((
        database.connection.clone().into(),
        paging_parameters,
        filter_parameters,
        path,
        auth_user,
    ));

    let response = response.unwrap();
    let mut expected_tags: HashMap<String, Value> = HashMap::new();
    let no_datetime: Option<NaiveDateTime> = None;
    expected_tags.insert("source_or_destination".to_string(), json!("destination"));
    expected_tags.insert("start_utc".to_string(), json!(no_datetime));
    expected_tags.insert("end_utc".to_string(), json!(no_datetime));
    let wrapped_expected_transfers = Payload {
        data: expected_transfers,
        paging: Paging {
            page: 0,
            limit: 100,
            sort: "".to_string(),
            dir: SortingDir::Asc,
            total: 1,
            tags: expected_tags,
        },
    };

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.payload(), &wrapped_expected_transfers);
}
