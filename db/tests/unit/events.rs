use bigneon_db::dev::TestProject;
use bigneon_db::models::*;
use chrono::prelude::*;
use uuid::Uuid;

#[test]
fn create() {
    let project = TestProject::new();
    let venue = project.create_venue().finish();
    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let event = project
        .create_event()
        .with_name("NewEvent".into())
        .with_organization(&organization)
        .with_venue(&venue)
        .finish();

    assert_eq!(event.venue_id, Some(venue.id));
    assert_eq!(event.organization_id, organization.id);
    assert_eq!(event.id.to_string().is_empty(), false);
}

#[test]
fn update() {
    //create event
    let project = TestProject::new();
    let venue = project.create_venue().finish();

    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let event = project
        .create_event()
        .with_name("NewEvent".into())
        .with_organization(&organization)
        .with_venue(&venue)
        .finish();
    //Edit event
    let parameters = EventEditableAttributes {
        door_time: Some(NaiveDate::from_ymd(2016, 7, 8).and_hms(4, 10, 11)),
        ..Default::default()
    };
    let event = event.update(parameters, project.get_connection()).unwrap();
    assert_eq!(
        event.door_time,
        Some(NaiveDate::from_ymd(2016, 7, 8).and_hms(4, 10, 11))
    );
}

#[test]
fn publish() {
    //create event
    let project = TestProject::new();
    let venue = project.create_venue().finish();

    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let event = project
        .create_event()
        .with_status(EventStatus::Draft)
        .with_name("NewEvent".into())
        .with_organization(&organization)
        .with_venue(&venue)
        .finish();

    assert_eq!(event.status(), EventStatus::Draft);

    let event_id = event.id;
    let venue = event.venue(project.get_connection()).unwrap().unwrap();
    let result = event.publish(project.get_connection());
    assert!(result.is_err());

    let venue_update = VenueEditableAttributes {
        address: Some("address".to_string()),
        city: Some("city".to_string()),
        state: Some("state".to_string()),
        country: Some("country".to_string()),
        postal_code: Some("333".to_string()),
        phone: Some("33333".to_string()),
        ..Default::default()
    };

    venue
        .update(venue_update, project.get_connection())
        .unwrap();

    let event = Event::find(event_id, project.get_connection())
        .unwrap()
        .publish(project.get_connection())
        .unwrap();

    assert_eq!(event.status(), EventStatus::Published);
    assert!(event.publish_date.is_some());
}

#[test]
fn cancel() {
    //create event
    let project = TestProject::new();
    let venue = project.create_venue().finish();

    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let event = project
        .create_event()
        .with_name("NewEvent".into())
        .with_organization(&organization)
        .with_venue(&venue)
        .finish();

    let event = event.cancel(&project.get_connection()).unwrap();
    assert!(!event.cancelled_at.is_none());
}

#[test]
fn find_individuals() {
    //create event
    let project = TestProject::new();
    let venue = project.create_venue().finish();

    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let event = project
        .create_event()
        .with_name("NewEvent".into())
        .with_organization(&organization)
        .with_venue(&venue)
        .finish();
    //Edit event
    let parameters = EventEditableAttributes {
        door_time: Some(NaiveDate::from_ymd(2016, 7, 8).and_hms(4, 10, 11)),
        ..Default::default()
    };
    let event = event.update(parameters, project.get_connection()).unwrap();

    //find event
    let found_event = Event::find(event.id, project.get_connection()).unwrap();
    assert_eq!(found_event, event);
    //find event via organisation
    let found_event_via_organization = Event::find_all_events_for_organization(
        found_event.organization_id,
        PastOrUpcoming::Past,
        0,
        100,
        project.get_connection(),
    ).unwrap();
    assert_eq!(found_event_via_organization.data[0].id, found_event.id);

    //find event via venue
    let found_event_via_venue =
        Event::find_all_events_for_venue(&event.venue_id.unwrap(), project.get_connection())
            .unwrap();
    assert_eq!(found_event_via_venue[0], event);
}

#[test]
fn search() {
    //create event
    let project = TestProject::new();
    let region1 = project.create_region().finish();
    let region2 = project.create_region().finish();
    let venue1 = project
        .create_venue()
        .with_name("Venue1".into())
        .with_region(&region1)
        .finish();
    let venue2 = project
        .create_venue()
        .with_name("Venue2".into())
        .with_region(&region2)
        .finish();
    let artist1 = project.create_artist().with_name("Artist1".into()).finish();
    let artist2 = project.create_artist().with_name("Artist2".into()).finish();
    let organization_owner = project.create_user().finish();
    let organization_user = project.create_user().finish();
    let user = project.create_user().finish();
    let admin = project
        .create_user()
        .finish()
        .add_role(Roles::Admin, project.get_connection())
        .unwrap();
    let organization = project
        .create_organization()
        .with_owner(&organization_owner)
        .with_user(&organization_user)
        .finish();
    let event = project
        .create_event()
        .with_status(EventStatus::Published)
        .with_name("OldEvent".into())
        .with_organization(&organization)
        .with_venue(&venue1)
        .with_event_start(&NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11))
        .finish();

    event
        .add_artist(artist1.id, project.get_connection())
        .unwrap();
    event
        .add_artist(artist2.id, project.get_connection())
        .unwrap();

    //find more than one event
    let event2 = project
        .create_event()
        .with_status(EventStatus::Closed)
        .with_name("NewEvent".into())
        .with_organization(&organization)
        .with_venue(&venue2)
        .with_event_start(&NaiveDate::from_ymd(2017, 7, 8).and_hms(9, 10, 11))
        .finish();

    event2
        .add_artist(artist1.id, project.get_connection())
        .unwrap();

    let event3 = project
        .create_event()
        .with_name("NewEvent2".into())
        .with_status(EventStatus::Offline)
        .with_organization(&organization)
        .with_venue(&venue2)
        .with_event_start(&NaiveDate::from_ymd(2017, 7, 8).and_hms(9, 10, 11))
        .finish();

    // Event draft, not returned except for organization user or owner
    let event4 = project
        .create_event()
        .with_name("NewEventDraft".into())
        .with_status(EventStatus::Draft)
        .with_organization(&organization)
        .with_venue(&venue2)
        .with_event_start(&NaiveDate::from_ymd(2017, 7, 8).and_hms(9, 10, 11))
        .finish();

    // Event draft belonging to other organization
    let event5 = project
        .create_event()
        .with_name("NewEventDraft2".into())
        .with_status(EventStatus::Draft)
        .with_event_start(&NaiveDate::from_ymd(2017, 7, 8).and_hms(9, 10, 11))
        .finish();

    let all_events = vec![event, event2, event3];
    let mut all_events_for_organization = all_events.clone();
    all_events_for_organization.push(event4);
    let mut all_events_for_admin = all_events_for_organization.clone();
    all_events_for_admin.push(event5);

    // All events unauthorized user
    let all_found_events =
        Event::search(None, None, None, None, None, None, project.get_connection()).unwrap();
    assert_eq!(all_events, all_found_events);

    // All events organization owner
    let all_found_events = Event::search(
        None,
        None,
        None,
        None,
        None,
        Some(organization_owner),
        project.get_connection(),
    ).unwrap();
    assert_eq!(all_events_for_organization, all_found_events);

    // All events organization user
    let all_found_events = Event::search(
        None,
        None,
        None,
        None,
        None,
        Some(organization_user),
        project.get_connection(),
    ).unwrap();
    assert_eq!(all_events_for_organization, all_found_events);

    // All events normal user not part of event organization
    let all_found_events = Event::search(
        None,
        None,
        None,
        None,
        None,
        Some(user),
        project.get_connection(),
    ).unwrap();
    assert_eq!(all_events, all_found_events);

    // All events for admin
    let all_found_events = Event::search(
        None,
        None,
        None,
        None,
        None,
        Some(admin),
        project.get_connection(),
    ).unwrap();
    assert_eq!(all_events_for_admin, all_found_events);

    // No name specified
    let all_found_events = Event::search(
        Some("".to_string()),
        None,
        None,
        None,
        None,
        None,
        project.get_connection(),
    ).unwrap();
    assert_eq!(all_events, all_found_events);

    // Limited by just Published and Offline events
    let all_found_events = Event::search(
        Some("".to_string()),
        None,
        None,
        None,
        Some(vec![EventStatus::Published, EventStatus::Offline]),
        None,
        project.get_connection(),
    ).unwrap();
    assert_eq!(all_found_events.len(), 2);
    assert_eq!(all_events[0], all_found_events[0]);
    assert_eq!(all_events[2], all_found_events[1]);

    // Limited by just Closed events
    let all_found_events = Event::search(
        Some("".to_string()),
        None,
        None,
        None,
        Some(vec![EventStatus::Closed]),
        None,
        project.get_connection(),
    ).unwrap();
    assert_eq!(all_found_events.len(), 1);
    assert_eq!(all_events[1], all_found_events[0]);

    // Event name search
    let all_found_events = Event::search(
        Some("New".to_string()),
        None,
        None,
        None,
        None,
        None,
        project.get_connection(),
    ).unwrap();
    assert_eq!(all_found_events.len(), 2);
    assert_eq!(all_events[1], all_found_events[0]);
    assert_eq!(all_events[2], all_found_events[1]);

    // Venue name search
    let all_found_events = Event::search(
        Some("Venue1".to_string()),
        None,
        None,
        None,
        None,
        None,
        project.get_connection(),
    ).unwrap();
    assert_eq!(all_found_events.len(), 1);
    assert_eq!(all_events[0], all_found_events[0]);

    // Artist name search for artist in both events
    let all_found_events = Event::search(
        Some("Artist1".to_string()),
        None,
        None,
        None,
        None,
        None,
        project.get_connection(),
    ).unwrap();
    assert_eq!(all_found_events.len(), 2);
    assert_eq!(all_events[0], all_found_events[0]);
    assert_eq!(all_events[1], all_found_events[1]);

    // Artist name search for artist at only one event
    let all_found_events = Event::search(
        Some("Artist2".to_string()),
        None,
        None,
        None,
        None,
        None,
        project.get_connection(),
    ).unwrap();
    assert_eq!(all_found_events.len(), 1);
    assert_eq!(all_events[0], all_found_events[0]);

    // Match names Venue2 and Artist2 returning all events
    let all_found_events = Event::search(
        Some("2".to_string()),
        None,
        None,
        None,
        None,
        None,
        project.get_connection(),
    ).unwrap();
    assert_eq!(all_events, all_found_events);

    // Match events belonging to given region
    let all_found_events = Event::search(
        None,
        Some(region1.id.into()),
        None,
        None,
        None,
        None,
        project.get_connection(),
    ).unwrap();
    assert_eq!(all_found_events.len(), 1);
    assert_eq!(all_events[0], all_found_events[0]);

    // Match events belonging to other region
    let all_found_events = Event::search(
        None,
        Some(region2.id.into()),
        None,
        None,
        None,
        None,
        project.get_connection(),
    ).unwrap();
    assert_eq!(all_found_events.len(), 2);
    assert_eq!(all_events[1], all_found_events[0]);
    assert_eq!(all_events[2], all_found_events[1]);

    // Combination of query and region resulting in no records
    let all_found_events = Event::search(
        Some("Artist2".to_string()),
        Some(region2.id.into()),
        None,
        None,
        None,
        None,
        project.get_connection(),
    ).unwrap();
    assert_eq!(all_found_events.len(), 0);

    // Combination of query and region resulting in records
    let all_found_events = Event::search(
        Some("Artist2".to_string()),
        Some(region1.id.into()),
        None,
        None,
        None,
        None,
        project.get_connection(),
    ).unwrap();
    assert_eq!(all_found_events.len(), 1);
    assert_eq!(all_events[0], all_found_events[0]);

    let all_found_events = Event::search(
        None,
        None,
        Some(NaiveDate::from_ymd(2017, 7, 8).and_hms(9, 0, 11)),
        None,
        None,
        None,
        project.get_connection(),
    ).unwrap();
    assert_eq!(all_found_events.len(), 2);
    assert_eq!(all_events[1], all_found_events[0]);
    assert_eq!(all_events[2], all_found_events[1]);

    let all_found_events = Event::search(
        None,
        None,
        None,
        Some(NaiveDate::from_ymd(2017, 7, 8).and_hms(9, 0, 11)),
        None,
        None,
        project.get_connection(),
    ).unwrap();
    assert_eq!(all_found_events.len(), 1);
    assert_eq!(all_events[0], all_found_events[0]);
}

#[test]
fn find_for_organization_and_venue() {
    //create event
    let project = TestProject::new();
    let venue1 = project.create_venue().finish();
    let venue2 = project.create_venue().finish();
    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();

    let artist1 = project.create_artist().finish();
    let artist2 = project.create_artist().finish();

    let event = project
        .create_event()
        .with_name("Event1".into())
        .with_event_start(
            &NaiveDateTime::parse_from_str("2014-03-04 12:00:00.000", "%Y-%m-%d %H:%M:%S%.f")
                .unwrap(),
        ).with_organization(&organization)
        .with_venue(&venue1)
        .finish();
    event
        .add_artist(artist1.id, project.get_connection())
        .unwrap();
    event
        .add_artist(artist2.id, project.get_connection())
        .unwrap();

    //find more than one event
    let event2 = project
        .create_event()
        .with_name("Event2".into())
        .with_event_start(
            &NaiveDateTime::parse_from_str("2014-03-05 12:00:00.000", "%Y-%m-%d %H:%M:%S%.f")
                .unwrap(),
        ).with_organization(&organization)
        .with_venue(&venue2)
        .finish();
    event2
        .add_artist(artist1.id, project.get_connection())
        .unwrap();

    let all_events = vec![event2.id, event.id];

    //find all events via organisation
    let found_event_via_organizations = Event::find_all_events_for_organization(
        organization.id,
        PastOrUpcoming::Past,
        0,
        100,
        project.get_connection(),
    ).unwrap();
    assert_eq!(
        found_event_via_organizations
            .data
            .iter()
            .map(|i| i.id)
            .collect::<Vec<Uuid>>(),
        all_events
    );

    //find all events via venue
    let found_event_via_venues =
        Event::find_all_events_for_venue(&venue1.id, project.get_connection()).unwrap();
    assert_eq!(found_event_via_venues.len(), 1);
    assert_eq!(found_event_via_venues[0].id, all_events[1]);
}

#[test]
fn organization() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let event = project
        .create_event()
        .with_name("NewEvent".into())
        .with_organization(&organization)
        .finish();

    assert_eq!(
        event.organization(project.get_connection()).unwrap(),
        organization
    );
}

#[test]
fn venue() {
    let project = TestProject::new();
    let venue = project.create_venue().finish();
    let event = project
        .create_event()
        .with_name("NewEvent".into())
        .with_venue(&venue)
        .finish();
    assert_eq!(event.venue(project.get_connection()).unwrap(), Some(venue));

    let event = project.create_event().with_name("NewEvent".into()).finish();
    assert_eq!(event.venue(project.get_connection()).unwrap(), None);
}

#[test]
fn add_ticket_type() {
    let project = TestProject::new();
    let conn = project.get_connection();
    let event = project.create_event().finish();
    let wallet_id = event.issuer_wallet(conn).unwrap().id;
    let sd = NaiveDate::from_ymd(2016, 7, 8).and_hms(4, 10, 11);
    let ed = NaiveDate::from_ymd(2016, 7, 9).and_hms(4, 10, 11);
    let ticket_type = event
        .add_ticket_type(
            "General Admission".to_string(),
            None,
            100,
            sd,
            ed,
            wallet_id,
            None,
            0,
            conn,
        ).unwrap();

    assert_eq!(ticket_type.event_id, event.id);
    assert_eq!(ticket_type.name, "General Admission".to_string());
}

#[test]
fn ticket_types() {
    let project = TestProject::new();
    let conn = project.get_connection();
    let event = project.create_event().finish();
    let wallet_id = event.issuer_wallet(conn).unwrap().id;
    let sd = NaiveDate::from_ymd(2016, 7, 8).and_hms(4, 10, 11);
    let ed = NaiveDate::from_ymd(2016, 7, 9).and_hms(4, 10, 11);
    let ticket_type_ga = event
        .add_ticket_type(
            "General Admission".to_string(),
            None,
            100,
            sd,
            ed,
            wallet_id,
            None,
            0,
            conn,
        ).unwrap();
    let ticket_type_vip = event
        .add_ticket_type(
            "VIP".to_string(),
            None,
            100,
            sd,
            ed,
            wallet_id,
            None,
            0,
            conn,
        ).unwrap();

    let ticket_types = event.ticket_types(conn).unwrap();

    assert_eq!(ticket_types, vec![ticket_type_ga, ticket_type_vip]);
}
