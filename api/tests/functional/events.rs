use actix_web::Query;
use actix_web::{http::StatusCode, FromRequest, HttpResponse, Path};
use bigneon_api::controllers::events;
use bigneon_api::controllers::events::*;
use bigneon_api::models::{PathParameters, UserDisplayTicketType};
use bigneon_db::models::*;
use chrono::prelude::*;
use diesel::PgConnection;
use functional::base;
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

#[test]
pub fn index() {
    let database = TestDatabase::new();
    let organization = database.create_organization().finish();
    let venue = database.create_venue().finish();
    let event = database
        .create_event()
        .with_name("NewEvent1".to_string())
        .with_organization(&organization)
        .with_venue(&venue)
        .finish();
    let event2 = database
        .create_event()
        .with_name("NewEvent2".to_string())
        .with_organization(&organization)
        .with_venue(&venue)
        .finish();

    let expected_results = vec![
        event_venue_entry(&event, &venue),
        event_venue_entry(&event2, &venue),
    ];

    let test_request = TestRequest::create_with_uri("/events?query=New");
    let parameters = Query::<SearchParameters>::from_request(&test_request.request, &()).unwrap();
    let response: HttpResponse =
        events::index((database.connection.into(), parameters, None)).into();

    let body = support::unwrap_body_to_string(&response).unwrap();

    let mut expected_tags: HashMap<String, Value> = HashMap::new();
    expected_tags.insert("query".to_string(), json!("New"));
    let wrapped_expected_events = Payload {
        data: expected_results,
        paging: Paging {
            page: 0,
            limit: 100,
            sort: "".to_string(),
            dir: SortingDir::Asc,
            total: 2,
            tags: expected_tags,
        },
    };
    let expected_json = serde_json::to_string(&wrapped_expected_events).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body, expected_json);
}

#[test]
pub fn index_with_draft_for_organization_user() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let auth_user = support::create_auth_user_from_user(
        &user,
        Roles::OrgMember,
        Some(&organization),
        &database,
    );
    let venue = database.create_venue().finish();
    let event = database
        .create_event()
        .with_name("NewEvent1".to_string())
        .with_organization(&organization)
        .with_venue(&venue)
        .finish();
    let event2 = database
        .create_event()
        .with_name("NewEvent2".to_string())
        .with_status(EventStatus::Draft)
        .with_organization(&organization)
        .with_venue(&venue)
        .finish();

    let expected_results = vec![
        event_venue_entry(&event, &venue),
        event_venue_entry(&event2, &venue),
    ];

    let test_request = TestRequest::create_with_uri("/events?query=New");
    let parameters = Query::<SearchParameters>::from_request(&test_request.request, &()).unwrap();
    let response: HttpResponse =
        events::index((database.connection.into(), parameters, Some(auth_user))).into();

    let body = support::unwrap_body_to_string(&response).unwrap();

    let mut expected_tags: HashMap<String, Value> = HashMap::new();
    expected_tags.insert("query".to_string(), json!("New"));
    let wrapped_expected_events = Payload {
        data: expected_results,
        paging: Paging {
            page: 0,
            limit: 100,
            sort: "".to_string(),
            dir: SortingDir::Asc,
            total: 2,
            tags: expected_tags,
        },
    };
    let expected_json = serde_json::to_string(&wrapped_expected_events).unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body, expected_json);
}

#[test]
pub fn index_with_draft_for_user_ignores_drafts() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
    let organization = database.create_organization().finish();
    let venue = database.create_venue().finish();
    let event = database
        .create_event()
        .with_name("NewEvent1".to_string())
        .with_organization(&organization)
        .with_venue(&venue)
        .finish();
    let _event2 = database
        .create_event()
        .with_name("NewEvent2".to_string())
        .with_status(EventStatus::Draft)
        .with_organization(&organization)
        .with_venue(&venue)
        .finish();

    let expected_results = vec![event_venue_entry(&event, &venue)];

    let test_request = TestRequest::create_with_uri("/events?query=New");
    let parameters = Query::<SearchParameters>::from_request(&test_request.request, &()).unwrap();
    let response: HttpResponse =
        events::index((database.connection.into(), parameters, Some(auth_user))).into();

    let body = support::unwrap_body_to_string(&response).unwrap();
    let mut expected_tags: HashMap<String, Value> = HashMap::new();
    expected_tags.insert("query".to_string(), json!("New"));
    let wrapped_expected_events = Payload {
        data: expected_results,
        paging: Paging {
            page: 0,
            limit: 100,
            sort: "".to_string(),
            dir: SortingDir::Asc,
            total: 1,
            tags: expected_tags,
        },
    };
    let expected_json = serde_json::to_string(&wrapped_expected_events).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body, expected_json);
}

#[test]
pub fn index_search_with_filter() {
    let database = TestDatabase::new();
    let organization = database.create_organization().finish();
    let event = database
        .create_event()
        .with_name("NewEvent1".to_string())
        .with_organization(&organization)
        .finish();
    database
        .create_event()
        .with_name("NewEvent2".to_string())
        .with_organization(&organization)
        .finish();

    let expected_events = vec![EventVenueEntry {
        id: event.id,
        name: event.name,
        organization_id: event.organization_id,
        venue_id: event.venue_id,
        created_at: event.created_at,
        event_start: event.event_start,
        door_time: event.door_time,
        status: event.status,
        publish_date: event.publish_date,
        promo_image_url: event.promo_image_url,
        additional_info: event.additional_info,
        top_line_info: event.top_line_info,
        age_limit: event.age_limit,
        cancelled_at: event.cancelled_at,
        venue: None,
        min_ticket_price: None,
        max_ticket_price: None,
    }];

    let test_request = TestRequest::create_with_uri("/events?query=NewEvent1");
    let parameters = Query::<SearchParameters>::from_request(&test_request.request, &()).unwrap();
    let response: HttpResponse =
        events::index((database.connection.into(), parameters, None)).into();

    let body = support::unwrap_body_to_string(&response).unwrap();
    let mut expected_tags: HashMap<String, Value> = HashMap::new();
    expected_tags.insert("query".to_string(), json!("NewEvent1"));
    let wrapped_expected_events = Payload {
        data: expected_events,
        paging: Paging {
            page: 0,
            limit: 100,
            sort: "".to_string(),
            dir: SortingDir::Asc,
            total: 1,
            tags: expected_tags,
        },
    };
    let expected_json = serde_json::to_string(&wrapped_expected_events).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body, expected_json);
}

#[test]
fn show() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);

    let organization = database.create_organization().finish();
    let venue = database.create_venue().finish();
    let event = database
        .create_event()
        .with_name("NewEvent".to_string())
        .with_organization(&organization)
        .with_venue(&venue)
        .with_ticket_pricing()
        .finish();
    let event_id = event.id;

    let artist1 = database.create_artist().finish();
    let artist2 = database.create_artist().finish();

    event.add_artist(artist1.id, &database.connection).unwrap();
    event.add_artist(artist2.id, &database.connection).unwrap();

    let _event_interest = EventInterest::create(event.id, user.id).commit(&database.connection);
    let event_expected_json = expected_show_json(event, organization, venue, &database.connection);

    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event_id;

    let response: HttpResponse =
        events::show((database.connection.into(), path, Some(auth_user))).into();
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body, event_expected_json);
}

#[cfg(test)]
mod dashboard_tests {
    use super::*;
    #[test]
    fn dashboard_org_member() {
        base::events::dashboard(Roles::OrgMember, true);
    }
    #[test]
    fn dashboard_admin() {
        base::events::dashboard(Roles::Admin, true);
    }
    #[test]
    fn dashboard_user() {
        base::events::dashboard(Roles::User, false);
    }
    #[test]
    fn dashboard_org_owner() {
        base::events::dashboard(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod create_tests {
    use super::*;
    #[test]
    fn create_org_member() {
        base::events::create(Roles::OrgMember, true);
    }
    #[test]
    fn create_admin() {
        base::events::create(Roles::Admin, true);
    }
    #[test]
    fn create_user() {
        base::events::create(Roles::User, false);
    }
    #[test]
    fn create_org_owner() {
        base::events::create(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod update_tests {
    use super::*;
    #[test]
    fn update_org_member() {
        base::events::update(Roles::OrgMember, true);
    }
    #[test]
    fn update_admin() {
        base::events::update(Roles::Admin, true);
    }
    #[test]
    fn update_user() {
        base::events::update(Roles::User, false);
    }
    #[test]
    fn update_org_owner() {
        base::events::update(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod cancel_tests {
    use super::*;
    #[test]
    fn cancel_org_member() {
        base::events::cancel(Roles::OrgMember, true);
    }
    #[test]
    fn cancel_admin() {
        base::events::cancel(Roles::Admin, true);
    }
    #[test]
    fn cancel_user() {
        base::events::cancel(Roles::User, false);
    }
    #[test]
    fn cancel_org_owner() {
        base::events::cancel(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod add_artist_tests {
    use super::*;
    #[test]
    fn add_artist_org_member() {
        base::events::add_artist(Roles::OrgMember, true);
    }
    #[test]
    fn add_artist_admin() {
        base::events::add_artist(Roles::Admin, true);
    }
    #[test]
    fn add_artist_user() {
        base::events::add_artist(Roles::User, false);
    }
    #[test]
    fn add_artist_org_owner() {
        base::events::add_artist(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod list_interested_users_tests {
    use super::*;

    #[test]
    fn list_interested_users_org_member() {
        base::events::list_interested_users(Roles::OrgMember, true);
    }
    #[test]
    fn list_interested_users_admin() {
        base::events::list_interested_users(Roles::Admin, true);
    }
    #[test]
    fn list_interested_users_user() {
        base::events::list_interested_users(Roles::User, true);
    }
    #[test]
    fn list_interested_users_org_owner() {
        base::events::list_interested_users(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod add_interest_tests {
    use super::*;

    #[test]
    fn add_interest_org_member() {
        base::events::add_interest(Roles::OrgMember, true);
    }
    #[test]
    fn add_interest_admin() {
        base::events::add_interest(Roles::Admin, true);
    }
    #[test]
    fn add_interest_user() {
        base::events::add_interest(Roles::User, true);
    }
    #[test]
    fn add_interest_org_owner() {
        base::events::add_interest(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod remove_interest_tests {
    use super::*;

    #[test]
    fn remove_interest_org_member() {
        base::events::remove_interest(Roles::OrgMember, true);
    }
    #[test]
    fn remove_interest_admin() {
        base::events::remove_interest(Roles::Admin, true);
    }
    #[test]
    fn remove_interest_user() {
        base::events::remove_interest(Roles::User, true);
    }
    #[test]
    fn remove_interest_org_owner() {
        base::events::remove_interest(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod update_artists_tests {
    use super::*;
    #[test]
    fn update_artists_org_member() {
        base::events::update_artists(Roles::OrgMember, true);
    }
    #[test]
    fn update_artists_admin() {
        base::events::update_artists(Roles::Admin, true);
    }
    #[test]
    fn update_artists_user() {
        base::events::update_artists(Roles::User, false);
    }
    #[test]
    fn update_artists_org_owner() {
        base::events::update_artists(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod guest_list_tests {
    use super::*;
    #[test]
    fn guest_list_org_member() {
        base::events::guest_list(Roles::OrgMember, true);
    }
    #[test]
    fn guest_list_admin() {
        base::events::guest_list(Roles::Admin, true);
    }
    #[test]
    fn guest_list_user() {
        base::events::guest_list(Roles::User, false);
    }
    #[test]
    fn guest_list_org_owner() {
        base::events::guest_list(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod codes_tests {
    use super::*;
    #[test]
    fn codes_org_member() {
        base::events::codes(Roles::OrgMember, true);
    }
    #[test]
    fn codes_admin() {
        base::events::codes(Roles::Admin, true);
    }
    #[test]
    fn codes_user() {
        base::events::codes(Roles::User, false);
    }
    #[test]
    fn codes_org_owner() {
        base::events::codes(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod holds_tests {
    use super::*;
    #[test]
    fn holds_org_member() {
        base::events::holds(Roles::OrgMember, true);
    }
    #[test]
    fn holds_admin() {
        base::events::holds(Roles::Admin, true);
    }
    #[test]
    fn holds_user() {
        base::events::holds(Roles::User, false);
    }
    #[test]
    fn holds_org_owner() {
        base::events::holds(Roles::OrgOwner, true);
    }
}

#[test]
fn dashboard_with_default_range() {
    let database = TestDatabase::new();
    let connection = &database.connection;
    let user = database.create_user().finish();
    let organization = database
        .create_organization()
        .with_fee_schedule(&database.create_fee_schedule().finish())
        .finish();
    let auth_user =
        support::create_auth_user_from_user(&user, Roles::OrgOwner, Some(&organization), &database);
    let event = database
        .create_event()
        .with_organization(&organization)
        .with_event_start(&NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11))
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
        connection,
    ).unwrap();
    assert_eq!(cart.calculate_total(connection).unwrap(), 1700);
    cart.add_external_payment("test".to_string(), user.id, 1700, connection)
        .unwrap();
    assert_eq!(cart.status().unwrap(), OrderStatus::Paid);

    let test_request = TestRequest::create_with_uri(&format!("/events/{}/dashboard?", event.id));
    let query_parameters =
        Query::<DashboardParameters>::from_request(&test_request.request, &()).unwrap();
    let mut path_parameters = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path_parameters.id = event.id;

    let response: HttpResponse = events::dashboard((
        database.connection.clone().into(),
        path_parameters,
        query_parameters,
        auth_user.clone(),
    )).into();

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let dashboard_result: DashboardResult = serde_json::from_str(&body).unwrap();
    assert_eq!(dashboard_result.day_stats.len(), 30);
    assert_eq!(
        dashboard_result.day_stats[29],
        DayStats {
            date: Utc::now().naive_utc().date(),
            revenue_in_cents: 1700,
            ticket_sales: 10,
        }
    );
}

#[test]
pub fn show_from_organizations_past() {
    let database = TestDatabase::new();

    let organization = database.create_organization().finish();
    let event = database
        .create_event()
        .with_name("NewEvent".to_string())
        .with_event_start(
            &NaiveDateTime::parse_from_str("2014-03-04 12:00:00.000", "%Y-%m-%d %H:%M:%S%.f")
                .unwrap(),
        ).with_organization(&organization)
        .finish();
    let _event2 = database
        .create_event()
        .with_name("NewEvent2".to_string())
        .with_event_start(
            &NaiveDateTime::parse_from_str("2059-03-02 12:00:00.000", "%Y-%m-%d %H:%M:%S%.f")
                .unwrap(),
        ).with_organization(&organization)
        .finish();

    let user = database.create_user().finish();
    let auth_user = support::create_auth_user_from_user(
        &user,
        Roles::OrgMember,
        Some(&organization),
        &database,
    );

    let expected_events = vec![event.id];

    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;
    let test_request = TestRequest::create_with_uri(&format!("/events?past_or_upcoming=Past"));
    let query_parameters =
        Query::<PagingParameters>::from_request(&test_request.request, &()).unwrap();
    let response = events::show_from_organizations((
        database.connection.into(),
        path,
        query_parameters,
        auth_user,
    )).unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        expected_events,
        response
            .payload()
            .data
            .iter()
            .map(|i| i.id)
            .collect::<Vec<Uuid>>()
    );
}

#[test]
pub fn show_from_organizations_upcoming() {
    let database = TestDatabase::new();

    let organization = database.create_organization().finish();
    let _event = database
        .create_event()
        .with_name("NewEvent".to_string())
        .with_event_start(
            &NaiveDateTime::parse_from_str("2014-03-04 12:00:00.000", "%Y-%m-%d %H:%M:%S%.f")
                .unwrap(),
        ).with_organization(&organization)
        .finish();
    let event2 = database
        .create_event()
        .with_event_start(
            &NaiveDateTime::parse_from_str("2059-03-02 12:00:00.000", "%Y-%m-%d %H:%M:%S%.f")
                .unwrap(),
        ).with_name("NewEvent2".to_string())
        .with_organization(&organization)
        .finish();

    let user = database.create_user().finish();
    let auth_user = support::create_auth_user_from_user(
        &user,
        Roles::OrgMember,
        Some(&organization),
        &database,
    );

    let expected_events = vec![event2.id];
    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;
    let test_request = TestRequest::create_with_uri(&format!("/events?past_or_upcoming=Upcoming"));
    let query_parameters =
        Query::<PagingParameters>::from_request(&test_request.request, &()).unwrap();
    let response = events::show_from_organizations((
        database.connection.into(),
        path,
        query_parameters,
        auth_user,
    )).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        expected_events,
        response
            .payload()
            .data
            .iter()
            .map(|i| i.id)
            .collect::<Vec<Uuid>>()
    );
}

#[test]
pub fn show_from_venues() {
    let database = TestDatabase::new();

    let organization = database.create_organization().finish();
    let venue = database.create_venue().finish();
    let event = database
        .create_event()
        .with_venue(&venue)
        .with_name("NewEvent".to_string())
        .with_organization(&organization)
        .finish();
    let event2 = database
        .create_event()
        .with_venue(&venue)
        .with_name("NewEvent2".to_string())
        .with_organization(&organization)
        .finish();
    // Private event is not returned
    let private_venue = database.create_venue().make_private().finish();
    let _event3 = database
        .create_event()
        .with_name("NewEvent3".to_string())
        .with_organization(&organization)
        .with_venue(&private_venue)
        .finish();

    let all_events = vec![event, event2];
    //find venue from organization
    let test_request = TestRequest::create();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = venue.id;
    let test_request = TestRequest::create_with_uri(&format!("/limits?"));
    let query_parameters =
        Query::<PagingParameters>::from_request(&test_request.request, &()).unwrap();
    let response: HttpResponse =
        events::show_from_venues((database.connection.into(), path, query_parameters)).into();
    let wrapped_expected_events = Payload {
        data: all_events,
        paging: Paging {
            page: 0,
            limit: 100,
            sort: "".to_string(),
            dir: SortingDir::Asc,
            total: 2,
            tags: HashMap::new(),
        },
    };
    let expected_json = serde_json::to_string(&wrapped_expected_events).unwrap();
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body, expected_json);
}

fn expected_show_json(
    event: Event,
    organization: Organization,
    venue: Venue,
    connection: &PgConnection,
) -> String {
    #[derive(Serialize)]
    struct ShortOrganization {
        id: Uuid,
        name: String,
    }
    #[derive(Serialize)]
    struct R {
        id: Uuid,
        name: String,
        organization_id: Uuid,
        venue_id: Option<Uuid>,
        created_at: NaiveDateTime,
        event_start: Option<NaiveDateTime>,
        door_time: Option<NaiveDateTime>,
        fee_in_cents: Option<i64>,
        status: String,
        publish_date: Option<NaiveDateTime>,
        promo_image_url: Option<String>,
        additional_info: Option<String>,
        top_line_info: Option<String>,
        age_limit: Option<i32>,
        video_url: Option<String>,
        organization: ShortOrganization,
        venue: Venue,
        artists: Vec<DisplayEventArtist>,
        ticket_types: Vec<UserDisplayTicketType>,
        total_interest: u32,
        user_is_interested: bool,
        min_ticket_price: Option<i64>,
        max_ticket_price: Option<i64>,
    }

    let fee_schedule = FeeSchedule::find(organization.fee_schedule_id, connection).unwrap();
    let event_artists = EventArtist::find_all_from_event(event.id, connection).unwrap();

    let display_ticket_types: Vec<UserDisplayTicketType> = event
        .ticket_types(connection)
        .unwrap()
        .iter()
        .map(|ticket_type| {
            UserDisplayTicketType::from_ticket_type(ticket_type, &fee_schedule, connection).unwrap()
        }).collect();

    serde_json::to_string(&R {
        id: event.id,
        name: event.name,
        organization_id: event.organization_id,
        venue_id: event.venue_id,
        created_at: event.created_at,
        event_start: event.event_start,
        door_time: event.door_time,
        fee_in_cents: event.fee_in_cents,
        status: event.status,
        publish_date: event.publish_date,
        promo_image_url: event.promo_image_url,
        additional_info: event.additional_info,
        top_line_info: event.top_line_info,
        age_limit: event.age_limit,
        video_url: event.video_url,
        organization: ShortOrganization {
            id: organization.id,
            name: organization.name,
        },
        venue,
        artists: event_artists,
        ticket_types: display_ticket_types,
        total_interest: 1,
        user_is_interested: true,
        min_ticket_price: None,
        max_ticket_price: None,
    }).unwrap()
}

#[derive(Serialize)]
struct EventVenueEntry {
    id: Uuid,
    name: String,
    organization_id: Uuid,
    venue_id: Option<Uuid>,
    created_at: NaiveDateTime,
    event_start: Option<NaiveDateTime>,
    door_time: Option<NaiveDateTime>,
    status: String,
    publish_date: Option<NaiveDateTime>,
    promo_image_url: Option<String>,
    additional_info: Option<String>,
    top_line_info: Option<String>,
    age_limit: Option<i32>,
    cancelled_at: Option<NaiveDateTime>,
    venue: Option<Venue>,
    min_ticket_price: Option<i64>,
    max_ticket_price: Option<i64>,
}

fn event_venue_entry(event: &Event, venue: &Venue) -> EventVenueEntry {
    EventVenueEntry {
        id: event.id,
        name: event.name.clone(),
        organization_id: event.organization_id,
        venue_id: event.venue_id,
        created_at: event.created_at,
        event_start: event.event_start,
        door_time: event.door_time,
        status: event.status.clone(),
        publish_date: event.publish_date,
        promo_image_url: event.promo_image_url.clone(),
        additional_info: event.additional_info.clone(),
        top_line_info: event.top_line_info.clone(),
        age_limit: event.age_limit,
        cancelled_at: event.cancelled_at,
        venue: Some(venue.clone()),
        min_ticket_price: event.min_ticket_price.clone(),
        max_ticket_price: event.max_ticket_price.clone(),
    }
}
