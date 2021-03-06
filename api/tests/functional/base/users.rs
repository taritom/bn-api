use crate::support;
use crate::support::database::TestDatabase;
use crate::support::test_request::TestRequest;
use actix_web::{
    http::StatusCode,
    web::{Path, Query},
    FromRequest, HttpResponse,
};
use api::controllers::users::{self, *};
use api::errors::*;
use api::extractors::*;
use api::models::*;
use db::models::*;
use serde_json;
use std::collections::HashMap;
use uuid::Uuid;

pub async fn profile(role: Roles, should_test_true: bool) {
    let database = TestDatabase::new();
    let connection = database.connection.get();
    let user = database.create_user().finish();
    let user2 = database.create_user().finish();
    let organization = database.create_organization().with_fees().finish();
    let event = database
        .create_event()
        .with_organization(&organization)
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let ticket_type = &event.ticket_types(true, None, connection).unwrap()[0];
    let order = database
        .create_order()
        .for_user(&user2)
        .for_event(&event)
        .quantity(10)
        .is_paid()
        .finish();
    let auth_user = support::create_auth_user_from_user(&user, role, Some(&organization), &database);

    let items = order.items(&connection).unwrap();
    let order_item = items.iter().find(|i| i.ticket_type_id == Some(ticket_type.id)).unwrap();
    let tickets = TicketInstance::find_for_order_item(order_item.id, connection).unwrap();
    let ticket = &tickets[0];
    TicketInstance::redeem_ticket(
        ticket.id,
        ticket.redeem_key.clone().unwrap(),
        user.id,
        CheckInSource::GuestList,
        connection,
    )
    .unwrap();

    let test_request = TestRequest::create_with_uri_custom_params("/", vec!["id", "user_id"]);
    let mut path = Path::<OrganizationFanPathParameters>::extract(&test_request.request)
        .await
        .unwrap();
    path.id = organization.id;
    path.user_id = user2.id;
    let response: HttpResponse = users::profile((database.connection.clone().into(), path, auth_user.clone()))
        .await
        .into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_true {
        assert_eq!(response.status(), StatusCode::OK);
        let user_profile: FanProfile = serde_json::from_str(&body).unwrap();
        assert_eq!(
            user_profile,
            FanProfile {
                first_name: user2.first_name.clone(),
                last_name: user2.last_name.clone(),
                email: user2.email.clone(),
                facebook_linked: false,
                revenue_in_cents: 1700,
                ticket_sales: 10,
                tickets_owned: 10,
                profile_pic_url: user2.profile_pic_url.clone(),
                thumb_profile_pic_url: user2.thumb_profile_pic_url.clone(),
                cover_photo_url: user2.cover_photo_url.clone(),
                created_at: user2.created_at,
                attendance_information: vec![AttendanceInformation {
                    event_name: event.name,
                    event_id: event.id,
                    event_start: event.event_start
                }],
                deleted_at: None
            }
        );
    } else {
        support::expects_unauthorized(&response);
    }
}

pub async fn activity(role: Roles, should_test_true: bool) {
    let database = TestDatabase::new();
    let connection = database.connection.get();
    let user = database.create_user().finish();
    let user2 = database.create_user().finish();
    let organization = database.create_organization().with_fees().finish();
    let event = database
        .create_event()
        .with_organization(&organization)
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let ticket_type = &event.ticket_types(true, None, connection).unwrap()[0];
    let mut order = Order::find_or_create_cart(&user2, connection).unwrap();
    order
        .update_quantities(
            user2.id,
            &[UpdateOrderItem {
                ticket_type_id: ticket_type.id,
                quantity: 10,
                redemption_code: None,
            }],
            false,
            false,
            &*connection,
        )
        .unwrap();
    assert_eq!(order.calculate_total(connection).unwrap(), 1700);
    order
        .add_external_payment(
            Some("test".to_string()),
            ExternalPaymentType::CreditCard,
            user.id,
            1700,
            connection,
        )
        .unwrap();
    assert_eq!(order.status, OrderStatus::Paid);
    let auth_user = support::create_auth_user_from_user(&user, role, Some(&organization), &database);

    let test_request = TestRequest::create_with_uri_custom_params("/", vec!["id", "user_id"]);
    let mut path = Path::<OrganizationFanPathParameters>::extract(&test_request.request)
        .await
        .unwrap();
    path.id = organization.id;
    path.user_id = user2.id;
    let query_parameters = Query::<PagingParameters>::extract(&test_request.request).await.unwrap();
    let activity_parameters = Query::<ActivityParameters>::extract(&test_request.request)
        .await
        .unwrap();
    let response: Result<WebPayload<ActivitySummary>, ApiError> = users::activity((
        database.connection.clone().into(),
        path,
        query_parameters,
        activity_parameters,
        auth_user.clone(),
    ))
    .await;

    if should_test_true {
        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let activity_payload = response.payload();
        let data = &activity_payload.data;
        assert_eq!(data.len(), 1);

        assert_eq!(data[0].event, event.for_display(connection).unwrap());
        assert_eq!(data[0].activity_items.len(), 1);
        if let ActivityItem::Purchase {
            order_id,
            order_number,
            ticket_quantity,
            purchased_by,
            user,
            ..
        } = &data[0].activity_items[0]
        {
            assert_eq!(order_id, &order.id);
            assert_eq!(order_number, &Order::order_number(&order));
            assert_eq!(ticket_quantity, &10);
            let expected_user: UserActivityItem = user2.clone().into();
            assert_eq!(purchased_by, &expected_user);
            assert_eq!(user, &expected_user);
        } else {
            panic!("Expected purchase activity item");
        }
    } else {
        support::expects_unauthorized(&response.unwrap_err().into_inner().to_response());
    }
}

pub async fn history(role: Roles, should_test_true: bool) {
    let database = TestDatabase::new();
    let connection = database.connection.get();
    let user = database.create_user().finish();
    let user2 = database.create_user().finish();
    let organization = database.create_organization().with_fees().finish();
    let event = database
        .create_event()
        .with_organization(&organization)
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let ticket_type = &event.ticket_types(true, None, connection).unwrap()[0];
    let mut cart = Order::find_or_create_cart(&user2, connection).unwrap();
    cart.update_quantities(
        user2.id,
        &[UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 10,
            redemption_code: None,
        }],
        false,
        false,
        &*connection,
    )
    .unwrap();
    assert_eq!(cart.calculate_total(connection).unwrap(), 1700);
    cart.add_external_payment(
        Some("test".to_string()),
        ExternalPaymentType::CreditCard,
        user.id,
        1700,
        connection,
    )
    .unwrap();
    assert_eq!(cart.status, OrderStatus::Paid);
    let auth_user = support::create_auth_user_from_user(&user, role, Some(&organization), &database);

    let test_request = TestRequest::create_with_uri_custom_params("/", vec!["id", "user_id"]);
    let mut path = Path::<OrganizationFanPathParameters>::extract(&test_request.request)
        .await
        .unwrap();
    path.id = organization.id;
    path.user_id = user2.id;
    let query_parameters = Query::<PagingParameters>::extract(&test_request.request).await.unwrap();
    let response: Result<WebPayload<HistoryItem>, ApiError> = users::history((
        database.connection.clone().into(),
        path,
        query_parameters,
        auth_user.clone(),
    ))
    .await;

    if should_test_true {
        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let history_payload = response.payload();

        let paging = Paging::new(0, 100);
        let mut payload = Payload::new(
            vec![HistoryItem::Purchase {
                order_id: cart.id,
                order_date: cart.order_date,
                event_name: event.name.clone(),
                ticket_sales: 10,
                revenue_in_cents: 1700,
            }],
            paging,
        );
        payload.paging.total = 1;
        payload.paging.dir = SortingDir::Desc;

        assert_eq!(history_payload, &payload);
    } else {
        support::expects_unauthorized(&response.unwrap_err().into_inner().to_response());
    }
}

pub async fn list_organizations(role: Roles, should_test_true: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let user2 = database.create_user().finish();
    let organization = database
        .create_organization()
        .with_member(&user2, Roles::OrgMember)
        .finish();
    let auth_user = support::create_auth_user_from_user(&user, role, Some(&organization), &database);

    let test_request = TestRequest::create_with_uri(&format!("/limits?"));
    let mut path = Path::<PathParameters>::extract(&test_request.request).await.unwrap();
    path.id = user2.id;
    let query_parameters = Query::<PagingParameters>::extract(&test_request.request).await.unwrap();
    let response: HttpResponse =
        users::list_organizations((database.connection.into(), path, query_parameters, auth_user.clone()))
            .await
            .into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_true {
        assert_eq!(response.status(), StatusCode::OK);
        #[derive(Serialize)]
        pub struct DisplayOrganizationLink {
            pub id: Uuid,
            pub name: String,
            pub role: Vec<String>,
        }
        let role_owner_string = vec![String::from("OrgMember")];
        let expected_data = DisplayOrganizationLink {
            id: organization.id,
            name: organization.name,
            role: role_owner_string,
        };
        let wrapped_expected_links = Payload {
            data: vec![expected_data],
            paging: Paging {
                page: 0,
                limit: 100,
                sort: "".to_string(),
                dir: SortingDir::Asc,
                total: 1,
                tags: HashMap::new(),
            },
        };
        let expected_json = serde_json::to_string(&wrapped_expected_links).unwrap();
        assert_eq!(body, expected_json);
    } else {
        support::expects_unauthorized(&response);
    }
    assert_eq!(true, true);
}

pub async fn show_push_notification_tokens_for_user_id(role: Roles, should_test_true: bool) {
    let database = TestDatabase::new();
    let connection = database.connection.get();
    let user = database.create_user().finish();
    let user2 = database.create_user().finish();
    let organization = database
        .create_organization()
        .with_member(&user2, Roles::OrgMember)
        .finish();
    let auth_user = support::create_auth_user_from_user(&user, role, Some(&organization), &database);
    //create push notification token for user2
    let created_token = NewPushNotificationToken {
        user_id: user2.id,
        token_source: "example_token_source".to_string(),
        token: "example_token".to_string(),
    };
    created_token.commit(user.id, &connection).unwrap();
    //Retrieve push notification token
    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).await.unwrap();
    path.id = user2.id;

    let response: HttpResponse =
        users::show_push_notification_tokens_for_user_id((database.connection.clone().into(), path, auth_user.clone()))
            .await
            .into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_true {
        assert_eq!(response.status(), StatusCode::OK);
        let retrieved_tokens: Vec<DisplayPushNotificationToken> = serde_json::from_str(&body).unwrap();
        assert_eq!(retrieved_tokens.len(), 1);
        if retrieved_tokens.len() >= 1 {
            assert_eq!(retrieved_tokens[0].token_source, created_token.token_source);
            assert_eq!(retrieved_tokens[0].token, created_token.token);
        }
    } else {
        support::expects_unauthorized(&response);
    }
}

pub async fn show_push_notification_tokens(role: Roles, should_test_true: bool) {
    let database = TestDatabase::new();
    let connection = database.connection.get();
    let user = database.create_user().finish();
    let organization = database
        .create_organization()
        .with_member(&user, Roles::OrgMember)
        .finish();
    let auth_user = support::create_auth_user_from_user(&user, role, Some(&organization), &database);
    //create push notification token
    let created_token = NewPushNotificationToken {
        user_id: user.id,
        token_source: "example_token_source".to_string(),
        token: "example_token".to_string(),
    };
    created_token.commit(user.id, &connection).unwrap();
    //Retrieve push notification tokens
    let response: HttpResponse =
        users::show_push_notification_tokens((database.connection.clone().into(), auth_user.clone()))
            .await
            .into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_true {
        assert_eq!(response.status(), StatusCode::OK);
        let retrieved_tokens: Vec<DisplayPushNotificationToken> = serde_json::from_str(&body).unwrap();
        assert_eq!(retrieved_tokens.len(), 1);
        if retrieved_tokens.len() >= 1 {
            assert_eq!(retrieved_tokens[0].token_source, created_token.token_source);
            assert_eq!(retrieved_tokens[0].token, created_token.token);
        }
    } else {
        support::expects_unauthorized(&response);
    }
}

pub async fn add_push_notification_token(role: Roles, should_test_true: bool) {
    let database = TestDatabase::new();
    let connection = database.connection.get();
    let user = database.create_user().finish();
    let organization = database
        .create_organization()
        .with_member(&user, Roles::OrgMember)
        .finish();
    let auth_user = support::create_auth_user_from_user(&user, role, Some(&organization), &database);
    //create push notification token for user
    let created_token = InputPushNotificationTokens {
        token_source: "example_token_source".to_string(),
        token: "example_token".to_string(),
    };
    let json = Json(created_token.clone());

    let response: HttpResponse =
        users::add_push_notification_token((database.connection.clone().into(), json, auth_user.clone()))
            .await
            .into();

    if should_test_true {
        assert_eq!(response.status(), StatusCode::OK);
        //Check if token was added to storage
        let retrieved_tokens = PushNotificationToken::find_by_user_id(user.id, connection).unwrap();
        assert_eq!(retrieved_tokens.len(), 1);
        if retrieved_tokens.len() >= 1 {
            assert_eq!(retrieved_tokens[0].user_id, user.id);
            assert_eq!(retrieved_tokens[0].token_source, created_token.token_source);
            assert_eq!(retrieved_tokens[0].token, created_token.token);
        }
    } else {
        support::expects_unauthorized(&response);
    }
}

pub async fn remove_push_notification_token(role: Roles, should_test_true: bool) {
    let database = TestDatabase::new();
    let connection = database.connection.get();
    let user = database.create_user().finish();
    let organization = database
        .create_organization()
        .with_member(&user, Roles::OrgMember)
        .finish();
    let auth_user = support::create_auth_user_from_user(&user, role, Some(&organization), &database);
    //create push notification token
    NewPushNotificationToken {
        user_id: user.id,
        token_source: "example_token_source".to_string(),
        token: "example_token".to_string(),
    }
    .commit(user.id, &connection)
    .unwrap();
    //check that it was created
    let stored_tokens = PushNotificationToken::find_by_user_id(user.id, connection).unwrap();
    assert_eq!(stored_tokens.len(), 1);
    //Remove push notification token
    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).await.unwrap();
    path.id = stored_tokens[0].id;

    let response: HttpResponse =
        users::remove_push_notification_token((database.connection.clone().into(), path, auth_user.clone()))
            .await
            .into();

    if should_test_true {
        assert_eq!(response.status(), StatusCode::OK);
        //Check that token was removed
        let stored_tokens = PushNotificationToken::find_by_user_id(user.id, connection).unwrap();
        assert_eq!(stored_tokens.len(), 0);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub async fn show(role: Roles, should_test_true: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let user2 = database.create_user().finish();

    let organization = database
        .create_organization()
        .with_member(&user2, Roles::OrgMember)
        .finish();
    let auth_user = support::create_auth_user_from_user(&user, role, Some(&organization), &database);

    let display_user = user2.for_display().unwrap();
    let test_request = TestRequest::create();

    let mut path = Path::<PathParameters>::extract(&test_request.request).await.unwrap();
    path.id = display_user.id;
    let response: HttpResponse = users::show((database.connection.into(), path, auth_user.clone()))
        .await
        .into();
    if should_test_true {
        let body = support::unwrap_body_to_string(&response).unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let user: DisplayUser = serde_json::from_str(&body).unwrap();
        assert_eq!(user, display_user);
    } else {
        support::expects_unauthorized(&response);
    }
}
