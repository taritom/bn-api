use actix_web::Query;
use actix_web::{http::StatusCode, FromRequest, HttpResponse, Path};
use bigneon_api::controllers::events::SearchParameters;
use bigneon_api::controllers::events::{self, PathParameters};
use bigneon_db::models::*;
use chrono::NaiveDateTime;
use functional::base;
use serde_json;
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
        age_limit: Option<i32>,
        cancelled_at: Option<NaiveDateTime>,
        venue: Option<Venue>,
    }

    let expected_results = vec![
        EventVenueEntry {
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
            age_limit: event.age_limit,
            cancelled_at: event.cancelled_at,
            venue: Some(venue.clone()),
        },
        EventVenueEntry {
            id: event2.id,
            name: event2.name,
            organization_id: event2.organization_id,
            venue_id: event2.venue_id,
            created_at: event2.created_at,
            event_start: event2.event_start,
            door_time: event2.door_time,
            status: event2.status,
            publish_date: event2.publish_date,
            promo_image_url: event2.promo_image_url,
            additional_info: event2.additional_info,
            age_limit: event2.age_limit,
            cancelled_at: event2.cancelled_at,
            venue: Some(venue),
        },
    ];
    let events_expected_json = serde_json::to_string(&expected_results).unwrap();

    let test_request = TestRequest::create_with_uri("/events?query=New");
    let parameters = Query::<SearchParameters>::from_request(&test_request.request, &()).unwrap();
    let response: HttpResponse = events::index((database.connection.into(), parameters)).into();

    let body = support::unwrap_body_to_string(&response).unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body, events_expected_json);
}

#[test]
pub fn index_search_returns_only_one_event() {
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
        age_limit: Option<i32>,
        cancelled_at: Option<NaiveDateTime>,
        venue: Option<Venue>,
    }

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
        age_limit: event.age_limit,
        cancelled_at: event.cancelled_at,
        venue: None,
    }];
    let events_expected_json = serde_json::to_string(&expected_events).unwrap();

    let test_request = TestRequest::create_with_uri("/events?query=NewEvent1");
    let parameters = Query::<SearchParameters>::from_request(&test_request.request, &()).unwrap();
    let response: HttpResponse = events::index((database.connection.into(), parameters)).into();

    let body = support::unwrap_body_to_string(&response).unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body, events_expected_json);
}

#[cfg(test)]
mod show_tests {
    use super::*;
    #[test]
    fn show_org_member() {
        base::events::show(Roles::OrgMember);
    }
    #[test]
    fn show_admin() {
        base::events::show(Roles::Admin);
    }
    #[test]
    fn show_user() {
        base::events::show(Roles::User);
    }
    #[test]
    fn show_org_owner() {
        base::events::show(Roles::OrgOwner);
    }
}

#[cfg(test)]
mod create_tests {
    use super::*;
    #[test]
    fn create_org_member() {
        base::events::create(Roles::OrgMember, true, true);
    }
    #[test]
    fn create_admin() {
        base::events::create(Roles::Admin, true, true);
    }
    #[test]
    fn create_user() {
        base::events::create(Roles::User, false, true);
    }
    #[test]
    fn create_org_owner() {
        base::events::create(Roles::OrgOwner, true, true);
    }
    #[test]
    fn create_other_organization_org_member() {
        base::events::create(Roles::OrgMember, false, false);
    }
    #[test]
    fn create_other_organization_admin() {
        base::events::create(Roles::Admin, true, false);
    }
    #[test]
    fn create_other_organization_user() {
        base::events::create(Roles::User, false, false);
    }
    #[test]
    fn create_other_organization_org_owner() {
        base::events::create(Roles::OrgOwner, false, false);
    }
}

#[cfg(test)]
mod update_tests {
    use super::*;
    #[test]
    fn update_org_member() {
        base::events::update(Roles::OrgMember, true, true);
    }
    #[test]
    fn update_admin() {
        base::events::update(Roles::Admin, true, true);
    }
    #[test]
    fn update_user() {
        base::events::update(Roles::User, false, true);
    }
    #[test]
    fn update_org_owner() {
        base::events::update(Roles::OrgOwner, true, true);
    }
    #[test]
    fn update_other_organization_org_member() {
        base::events::update(Roles::OrgMember, false, false);
    }
    #[test]
    fn update_other_organization_admin() {
        base::events::update(Roles::Admin, true, false);
    }
    #[test]
    fn update_other_organization_user() {
        base::events::update(Roles::User, false, false);
    }
    #[test]
    fn update_other_organization_org_owner() {
        base::events::update(Roles::OrgOwner, false, false);
    }
}

#[cfg(test)]
mod cancel_tests {
    use super::*;
    #[test]
    fn cancel_org_member() {
        base::events::cancel(Roles::OrgMember, true, true);
    }
    #[test]
    fn cancel_admin() {
        base::events::cancel(Roles::Admin, true, true);
    }
    #[test]
    fn cancel_user() {
        base::events::cancel(Roles::User, false, true);
    }
    #[test]
    fn cancel_org_owner() {
        base::events::cancel(Roles::OrgOwner, true, true);
    }
    #[test]
    fn cancel_other_organization_org_member() {
        base::events::cancel(Roles::OrgMember, false, false);
    }
    #[test]
    fn cancel_other_organization_admin() {
        base::events::cancel(Roles::Admin, true, false);
    }
    #[test]
    fn cancel_other_organization_user() {
        base::events::cancel(Roles::User, false, false);
    }
    #[test]
    fn cancel_other_organization_org_owner() {
        base::events::cancel(Roles::OrgOwner, false, false);
    }
}

#[cfg(test)]
mod add_artist_tests {
    use super::*;
    #[test]
    fn add_artist_org_member() {
        base::events::add_artist(Roles::OrgMember, true, true);
    }
    #[test]
    fn add_artist_admin() {
        base::events::add_artist(Roles::Admin, true, true);
    }
    #[test]
    fn add_artist_user() {
        base::events::add_artist(Roles::User, false, true);
    }
    #[test]
    fn add_artist_org_owner() {
        base::events::add_artist(Roles::OrgOwner, true, true);
    }
    #[test]
    fn add_artist_other_organization_org_member() {
        base::events::add_artist(Roles::OrgMember, false, false);
    }
    #[test]
    fn add_artist_other_organization_admin() {
        base::events::add_artist(Roles::Admin, true, false);
    }
    #[test]
    fn add_artist_other_organization_user() {
        base::events::add_artist(Roles::User, false, false);
    }
    #[test]
    fn add_artist_other_organization_org_owner() {
        base::events::add_artist(Roles::OrgOwner, false, false);
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
        base::events::update_artists(Roles::OrgMember, true, true);
    }
    #[test]
    fn update_artists_admin() {
        base::events::update_artists(Roles::Admin, true, true);
    }
    #[test]
    fn update_artists_user() {
        base::events::update_artists(Roles::User, false, true);
    }
    #[test]
    fn update_artists_org_owner() {
        base::events::update_artists(Roles::OrgOwner, true, true);
    }
    #[test]
    fn update_artists_other_organization_org_member() {
        base::events::update_artists(Roles::OrgMember, false, false);
    }
    #[test]
    fn update_artists_other_organization_admin() {
        base::events::update_artists(Roles::Admin, true, false);
    }
    #[test]
    fn update_artists_other_organization_user() {
        base::events::update_artists(Roles::User, false, false);
    }
    #[test]
    fn update_artists_other_organization_org_owner() {
        base::events::update_artists(Roles::OrgOwner, false, false);
    }
}

#[test]
pub fn show_from_organizations() {
    let database = TestDatabase::new();

    let organization = database.create_organization().finish();
    let event = database
        .create_event()
        .with_name("NewEvent".to_string())
        .with_organization(&organization)
        .finish();
    let event2 = database
        .create_event()
        .with_name("NewEvent2".to_string())
        .with_organization(&organization)
        .finish();

    let all_events = vec![event, event2];
    let event_expected_json = serde_json::to_string(&all_events).unwrap();

    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;
    let response: HttpResponse =
        events::show_from_organizations((database.connection.into(), path)).into();

    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body, event_expected_json);
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
    let event_expected_json = serde_json::to_string(&all_events).unwrap();
    //find venue from organization
    let test_request = TestRequest::create();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = venue.id;
    let response: HttpResponse =
        events::show_from_venues((database.connection.into(), path)).into();

    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body, event_expected_json);
}

#[cfg(test)]
mod create_tickets_tests {
    use super::*;
    #[test]
    fn create_tickets_org_member() {
        base::events::create_tickets(Roles::OrgMember, true);
    }
    #[test]
    fn create_tickets_admin() {
        base::events::create_tickets(Roles::Admin, true);
    }
    #[test]
    fn create_tickets_user() {
        base::events::create_tickets(Roles::User, false);
    }
    #[test]
    fn create_tickets_org_owner() {
        base::events::create_tickets(Roles::OrgOwner, true);
    }
}

#[test]
fn list_ticket_types() {
    let database = TestDatabase::new();
    let event = database.create_event().with_ticket_pricing().finish();
    let request = TestRequest::create();

    let mut path = Path::<PathParameters>::extract(&request.request).unwrap();
    path.id = event.id;

    let response = events::list_ticket_types((database.connection.into(), path)).unwrap();

    let _body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
