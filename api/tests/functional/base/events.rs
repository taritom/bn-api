use actix_web::{http::StatusCode, FromRequest, HttpResponse, Path, Query};
use bigneon_api::controllers::events;
use bigneon_api::controllers::events::*;
use bigneon_api::extractors::*;
use bigneon_api::models::PathParameters;
use bigneon_db::models::*;
use chrono::prelude::*;
use chrono::Duration;
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

pub fn create(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);
    let venue = database.create_venue().finish();

    let name = "event Example";
    let new_event = NewEvent {
        name: name.to_string(),
        organization_id: organization.id,
        venue_id: Some(venue.id),
        event_start: Some(NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11)),
        door_time: Some(NaiveDate::from_ymd(2016, 7, 8).and_hms(8, 11, 12)),
        publish_date: Some(NaiveDate::from_ymd(2016, 7, 1).and_hms(9, 10, 11)),
        ..Default::default()
    };
    // Emulate serialization for default serde behavior
    let new_event: NewEvent =
        serde_json::from_str(&serde_json::to_string(&new_event).unwrap()).unwrap();
    let json = Json(new_event);

    let response: HttpResponse =
        events::create((database.connection.into(), json, auth_user.clone())).into();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::CREATED);
        let body = support::unwrap_body_to_string(&response).unwrap();
        let event: Event = serde_json::from_str(&body).unwrap();
        assert_eq!(event.status, EventStatus::Draft);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn update(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);
    let event = database
        .create_event()
        .with_organization(&organization)
        .finish();

    let new_name = "New Event Name";
    let test_request = TestRequest::create();

    let json = Json(EventEditableAttributes {
        name: Some(new_name.to_string()),
        ..Default::default()
    });
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let response: HttpResponse =
        events::update((database.connection.into(), path, json, auth_user.clone())).into();
    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        let updated_event: Event = serde_json::from_str(&body).unwrap();
        assert_eq!(updated_event.name, new_name);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn cancel(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);

    let event = database
        .create_event()
        .with_organization(&organization)
        .finish();
    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let response: HttpResponse =
        events::cancel((database.connection.into(), path, auth_user)).into();
    if should_test_succeed {
        let body = support::unwrap_body_to_string(&response).unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let updated_event: Event = serde_json::from_str(&body).unwrap();
        assert!(!updated_event.cancelled_at.is_none());
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn add_artist(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);

    let event = database
        .create_event()
        .with_organization(&organization)
        .finish();
    let artist = database
        .create_artist()
        .with_organization(&organization)
        .finish();

    let test_request = TestRequest::create();

    let new_event_artist = AddArtistRequest {
        artist_id: artist.id,
        rank: 5,
        set_time: Some(NaiveDate::from_ymd(2016, 7, 1).and_hms(9, 10, 11)),
    };

    let json = Json(new_event_artist);

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let response: HttpResponse =
        events::add_artist((database.connection.into(), path, json, auth_user.clone())).into();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::CREATED);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn list_interested_users(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let event = database.create_event().finish();
    let primary_user = support::create_auth_user(role, None, &database);
    let conn = database.connection.get();
    EventInterest::create(event.id, primary_user.id())
        .commit(conn)
        .unwrap();
    let n_secondary_users = 5;
    let mut secondary_users: Vec<DisplayEventInterestedUser> = Vec::new();
    secondary_users.reserve(n_secondary_users);
    for _u_id in 0..n_secondary_users {
        let current_secondary_user = database.create_user().finish();
        EventInterest::create(event.id, current_secondary_user.id)
            .commit(conn)
            .unwrap();
        let current_user_entry = DisplayEventInterestedUser {
            user_id: current_secondary_user.id,
            first_name: current_secondary_user
                .first_name
                .clone()
                .unwrap_or("".to_string()),
            last_name: current_secondary_user
                .last_name
                .clone()
                .unwrap_or("".to_string()),
            thumb_profile_pic_url: None,
        };
        secondary_users.push(current_user_entry);
    }
    secondary_users.sort_by_key(|x| x.user_id); //Sort results for testing purposes
                                                //Construct api query
    let page: usize = 0;
    let limit: usize = 100;
    let test_request = TestRequest::create_with_uri(&format!(
        "/interest?page={}&limit={}",
        page.to_string(),
        limit.to_string()
    ));
    let query_parameters = Query::<PagingParameters>::extract(&test_request.request).unwrap();
    let mut path_parameters = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path_parameters.id = event.id;
    let response: HttpResponse = events::list_interested_users((
        database.connection.clone(),
        path_parameters,
        query_parameters,
        primary_user,
    ))
    .into();
    let response_body = support::unwrap_body_to_string(&response).unwrap();
    //Construct expected output
    let len = secondary_users.len() as u64;
    let wrapped_expected_date = Payload {
        data: secondary_users,
        paging: Paging {
            page: 0,
            limit: 100,
            sort: "".to_string(),
            dir: SortingDir::Asc,
            total: len,
            tags: HashMap::new(),
        },
    };
    let expected_json_body = serde_json::to_string(&wrapped_expected_date).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response_body, expected_json_body);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn add_interest(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let event = database.create_event().finish();

    let user = support::create_auth_user(role, None, &database);
    let test_request = TestRequest::create();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let response: HttpResponse =
        events::add_interest((database.connection.into(), path, user)).into();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::CREATED);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn remove_interest(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let event = database.create_event().finish();
    EventInterest::create(event.id, user.id)
        .commit(database.connection.get())
        .unwrap();

    let user = support::create_auth_user_from_user(&user, role, None, &database);
    let test_request = TestRequest::create();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let response: HttpResponse =
        events::remove_interest((database.connection.into(), path, user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body, "1");
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn update_artists(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);

    let event = database
        .create_event()
        .with_organization(&organization)
        .finish();
    let artist1 = database.create_artist().finish();
    let artist2 = database.create_artist().finish();
    let test_request = TestRequest::create();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let mut payload: UpdateArtistsRequestList = Default::default();
    payload.artists.push(UpdateArtistsRequest {
        artist_id: artist1.id,
        set_time: Some(NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11)),
    });
    payload.artists.push(UpdateArtistsRequest {
        artist_id: artist2.id,
        set_time: None,
    });
    let response: HttpResponse = events::update_artists((
        database.connection.into(),
        path,
        Json(payload),
        auth_user.clone(),
    ))
    .into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        let returned_event_artists: Vec<EventArtist> = serde_json::from_str(&body).unwrap();
        assert_eq!(returned_event_artists[0].artist_id, artist1.id);
        assert_eq!(returned_event_artists[1].set_time, None);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn dashboard(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.connection.get();
    let user = database.create_user().finish();
    let organization = database
        .create_organization()
        .with_fee_schedule(&database.create_fee_schedule().finish())
        .finish();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);
    let event = database
        .create_event()
        .with_organization(&organization)
        .with_event_start(NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11))
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let ticket_type = &event.ticket_types(connection).unwrap()[0];

    // user purchases 10 tickets
    let mut cart = Order::find_or_create_cart(&user, connection).unwrap();
    cart.update_quantities(
        &[UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 10,
            redemption_code: None,
        }],
        false,
        connection,
    )
    .unwrap();
    assert_eq!(cart.calculate_total(connection).unwrap(), 1700);
    cart.add_external_payment(Some("test".to_string()), user.id, 1700, connection)
        .unwrap();
    assert_eq!(cart.status, OrderStatus::Paid);

    let start_utc = Utc::now().naive_utc().date() - Duration::days(1);
    let end_utc = Utc::now().naive_utc().date();

    let test_request = TestRequest::create_with_uri(&format!(
        "/events/{}/dashboard?start_utc={:?}&end_utc={:?}",
        event.id, start_utc, end_utc
    ));
    let query_parameters = Query::<DashboardParameters>::extract(&test_request.request).unwrap();
    let mut path_parameters = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path_parameters.id = event.id;

    let response: HttpResponse = events::dashboard((
        database.connection.clone().into(),
        path_parameters,
        query_parameters,
        auth_user.clone(),
    ))
    .into();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        let body = support::unwrap_body_to_string(&response).unwrap();
        let dashboard_result: DashboardResult = serde_json::from_str(&body).unwrap();
        assert_eq!(dashboard_result.day_stats.len(), 2);
        assert_eq!(
            dashboard_result.day_stats,
            vec![
                DayStats {
                    date: start_utc,
                    revenue_in_cents: 0,
                    ticket_sales: 0,
                },
                DayStats {
                    date: end_utc,
                    revenue_in_cents: 1700,
                    ticket_sales: 10,
                }
            ]
        );
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn guest_list(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);

    let event = database
        .create_event()
        .with_organization(&organization)
        .with_ticket_pricing()
        .finish();
    database.create_order().for_event(&event).is_paid().finish();
    database.create_order().for_event(&event).is_paid().finish();

    let test_request = TestRequest::create_with_uri(&format!("/events/{}/guest?query=", event.id,));
    let query_parameters =
        Query::<GuestListQueryParameters>::extract(&test_request.request).unwrap();
    let mut path_parameters = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path_parameters.id = event.id;
    let response: HttpResponse = events::guest_list((
        database.connection.into(),
        query_parameters,
        path_parameters,
        auth_user,
    ))
    .into();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn codes(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.connection.get();
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);
    let event = database
        .create_event()
        .with_organization(&organization)
        .with_ticket_pricing()
        .finish();
    let code = database
        .create_code()
        .with_name("Discount 1".into())
        .with_event(&event)
        .with_code_type(CodeTypes::Discount)
        .finish()
        .for_display(connection)
        .unwrap();
    let code2 = database
        .create_code()
        .with_name("Discount 2".into())
        .with_event(&event)
        .with_code_type(CodeTypes::Discount)
        .finish()
        .for_display(connection)
        .unwrap();
    let _code3 = database
        .create_code()
        .with_name("Access".into())
        .with_event(&event)
        .with_code_type(CodeTypes::Access)
        .finish()
        .for_display(connection)
        .unwrap();

    let all_discounts = vec![code, code2];
    let test_request = TestRequest::create();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;
    let test_request = TestRequest::create_with_uri(&format!("/codes?type=Discount"));
    let query_parameters = Query::<PagingParameters>::extract(&test_request.request).unwrap();

    let response: HttpResponse = events::codes((
        database.connection.clone().into(),
        query_parameters,
        path,
        auth_user,
    ))
    .into();

    let mut expected_tags: HashMap<String, Value> = HashMap::new();
    expected_tags.insert("type".to_string(), json!("Discount"));

    let expected_discounts = Payload {
        data: all_discounts,
        paging: Paging {
            page: 0,
            limit: 100,
            sort: "".to_string(),
            dir: SortingDir::Asc,
            total: 2,
            tags: expected_tags,
        },
    };
    let expected_json = serde_json::to_string(&expected_discounts).unwrap();
    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body, expected_json);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn holds(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();

    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);
    let event = database
        .create_event()
        .with_organization(&organization)
        .with_ticket_pricing()
        .finish();

    let hold = database
        .create_hold()
        .with_name("Hold 1".to_string())
        .with_event(&event)
        .finish();
    let hold2 = database
        .create_hold()
        .with_name("Hold 2".to_string())
        .with_event(&event)
        .finish();

    #[derive(Serialize)]
    struct R {
        pub id: Uuid,
        pub name: String,
        pub event_id: Uuid,
        pub redemption_code: String,
        pub discount_in_cents: Option<i64>,
        pub end_at: Option<NaiveDateTime>,
        pub max_per_order: Option<i64>,
        pub hold_type: HoldTypes,
        pub ticket_type_id: Uuid,
        pub available: u32,
        pub quantity: u32,
    }

    let all_holds = vec![
        R {
            id: hold.id,
            name: hold.name,
            event_id: hold.event_id,
            redemption_code: hold.redemption_code,
            discount_in_cents: hold.discount_in_cents,
            end_at: hold.end_at,
            max_per_order: hold.max_per_order,
            hold_type: hold.hold_type,
            ticket_type_id: hold.ticket_type_id,
            available: 10,
            quantity: 10,
        },
        R {
            id: hold2.id,
            name: hold2.name,
            event_id: hold2.event_id,
            redemption_code: hold2.redemption_code,
            discount_in_cents: hold2.discount_in_cents,
            end_at: hold2.end_at,
            max_per_order: hold2.max_per_order,
            hold_type: hold2.hold_type,
            ticket_type_id: hold2.ticket_type_id,
            available: 10,
            quantity: 10,
        },
    ];

    let test_request = TestRequest::create();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;
    let test_request = TestRequest::create_with_uri(&format!("/holds"));
    let query_parameters = Query::<PagingParameters>::extract(&test_request.request).unwrap();

    let response: HttpResponse = events::holds((
        database.connection.into(),
        query_parameters,
        path,
        auth_user,
    ))
    .into();
    let expected_holds = Payload {
        data: all_holds,
        paging: Paging {
            page: 0,
            limit: 100,
            sort: "".to_string(),
            dir: SortingDir::Asc,
            total: 2,
            tags: HashMap::new(),
        },
    };
    let expected_json = serde_json::to_string(&expected_holds).unwrap();
    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body, expected_json);
    } else {
        support::expects_unauthorized(&response);
    }
}
