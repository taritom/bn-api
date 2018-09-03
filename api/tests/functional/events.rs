use actix_web::Query;
use actix_web::{http::StatusCode, FromRequest, HttpResponse, Path};
use bigneon_api::controllers::events::SearchParameters;
use bigneon_api::controllers::events::{self, PathParameters};
use bigneon_api::database::ConnectionGranting;
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
            venue: Some(venue),
        },
    ];
    let events_expected_json = serde_json::to_string(&expected_results).unwrap();

    let test_request = TestRequest::create_with_uri(database, "/events?query=New");
    let state = test_request.extract_state();
    let parameters = Query::<SearchParameters>::from_request(&test_request.request, &()).unwrap();
    let response: HttpResponse = events::index((state, parameters)).into();

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
        venue: None,
    }];
    let events_expected_json = serde_json::to_string(&expected_events).unwrap();

    let test_request = TestRequest::create_with_uri(database, "/events?query=NewEvent1");
    let state = test_request.extract_state();
    let parameters = Query::<SearchParameters>::from_request(&test_request.request, &()).unwrap();
    let response: HttpResponse = events::index((state, parameters)).into();

    let body = support::unwrap_body_to_string(&response).unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body, events_expected_json);
}

#[test]
pub fn show() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let auth_user = support::create_auth_user_from_user(&user, Roles::User, &database);
    let organization = database.create_organization().finish();
    let venue = database.create_venue().finish();
    let event = database
        .create_event()
        .with_name("NewEvent".to_string())
        .with_organization(&organization)
        .with_venue(&venue)
        .finish();
    let event_id = event.id;

    let artist1 = database.create_artist().finish();
    let artist2 = database.create_artist().finish();

    event
        .add_artist(artist1.id, &*database.get_connection())
        .unwrap();
    event
        .add_artist(artist2.id, &*database.get_connection())
        .unwrap();

    let _event_interest =
        EventInterest::create(event.id, user.id).commit(&*database.get_connection());

    #[derive(Serialize)]
    struct ShortOrganization {
        id: Uuid,
        name: String,
    }
    #[derive(Serialize)]
    struct DisplayEventArtist {
        event_id: Uuid,
        artist_id: Uuid,
        rank: i32,
        set_time: Option<NaiveDateTime>,
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
        status: String,
        publish_date: Option<NaiveDateTime>,
        promo_image_url: Option<String>,
        additional_info: Option<String>,
        age_limit: Option<i32>,
        organization: ShortOrganization,
        venue: Venue,
        artists: Vec<DisplayEventArtist>,
        total_interest: u32,
        user_is_interested: bool,
    }

    let event_artists =
        EventArtist::find_all_from_event(event.id, &*database.get_connection()).unwrap();

    let display_event_artists: Vec<DisplayEventArtist> = event_artists
        .iter()
        .map(|e| DisplayEventArtist {
            event_id: e.event_id,
            artist_id: e.artist_id,
            rank: e.rank,
            set_time: e.set_time,
        })
        .collect();

    let event_expected_json = serde_json::to_string(&R {
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
        organization: ShortOrganization {
            id: organization.id,
            name: organization.name,
        },
        venue: venue,
        artists: display_event_artists,
        total_interest: 1,
        user_is_interested: true,
    }).unwrap();

    let test_request = TestRequest::create(database);
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event_id;
    let state = test_request.extract_state();

    let response: HttpResponse = events::show((state, path, Some(auth_user))).into();
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body, event_expected_json);
}

#[cfg(test)]
mod create_tests {
    use super::*;
    #[test]
    fn create_org_member() {
        base::events::create(Roles::OrgMember, true);
    }
    #[test]
    fn create_guest() {
        base::events::create(Roles::Guest, false);
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
    fn update_guest() {
        base::events::update(Roles::Guest, false);
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
mod add_artist_tests {
    use super::*;
    #[test]
    fn add_artist_org_member() {
        base::events::add_artist(Roles::OrgMember, true);
    }
    #[test]
    fn add_artist_guest() {
        base::events::add_artist(Roles::Guest, false);
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

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;
    let response: HttpResponse = events::show_from_organizations((state, path)).into();

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

    let all_events = vec![event, event2];
    let event_expected_json = serde_json::to_string(&all_events).unwrap();
    //find venue from organization
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = venue.id;
    let response: HttpResponse = events::show_from_venues((state, path)).into();

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
    fn create_tickets_guest() {
        base::events::create_tickets(Roles::Guest, false);
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
    let db = TestDatabase::new();
    let event = db.create_event().with_price_points().finish();
    let request = TestRequest::create(db);
    let state = request.extract_state();

    let mut path = Path::<PathParameters>::extract(&request.request).unwrap();
    path.id = event.id;

    let response = events::list_ticket_types((state, path)).unwrap();

    let _body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
