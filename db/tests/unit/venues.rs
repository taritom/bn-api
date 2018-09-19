use bigneon_db::models::{Venue, VenueEditableAttributes};
use support::project::TestProject;

#[test]
fn commit() {
    let project = TestProject::new();
    let name = "Name";
    let venue = Venue::create(name.clone(), None, None)
        .commit(project.get_connection())
        .unwrap();

    assert_eq!(venue.name, name);
    assert_eq!(venue.id.to_string().is_empty(), false);
}

#[test]
fn update() {
    let project = TestProject::new();
    let venue = project.create_venue().finish();

    let new_name = "New Venue Name";
    let new_address = "Test Address";

    let parameters = VenueEditableAttributes {
        name: Some(new_name.to_string()),
        address: Some(new_address.to_string()),
        ..Default::default()
    };

    let updated_venue = venue.update(parameters, project.get_connection()).unwrap();
    assert_eq!(updated_venue.name, new_name);
    assert_eq!(updated_venue.address.unwrap(), new_address);
}

#[test]
fn set_privacy() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let mut venue = project.create_venue().finish();
    assert!(!venue.is_private);

    venue = venue.set_privacy(true, connection).unwrap();
    assert!(venue.is_private);

    venue = venue.set_privacy(false, connection).unwrap();
    assert!(!venue.is_private);
}

#[test]
fn find() {
    let project = TestProject::new();
    let venue = project.create_venue().finish();

    let found_venue = Venue::find(venue.id, project.get_connection()).unwrap();
    assert_eq!(venue, found_venue);
}

#[test]
fn all() {
    let project = TestProject::new();
    let venue = project
        .create_venue()
        .with_name("Venue1".to_string())
        .finish();
    let venue2 = project
        .create_venue()
        .with_name("Venue2".to_string())
        .finish();
    let organization = project.create_organization().finish();

    let all_found_venues = Venue::all(None, &project.get_connection()).unwrap();
    let mut all_venues = vec![venue, venue2];
    assert_eq!(all_venues, all_found_venues);

    let venue3 = project
        .create_venue()
        .with_name("Venue3".to_string())
        .make_private()
        .finish();
    let venue3 = venue3.add_to_organization(&organization.id, &project.get_connection());
    let user = project.create_user().finish();
    let _ = organization
        .add_user(user.id, &project.get_connection())
        .unwrap();
    all_venues.push(venue3.unwrap());
    let all_found_venues = Venue::all(Some(user.id), &project.get_connection()).unwrap();
    assert_eq!(all_venues, all_found_venues);
    let all_found_venues =
        Venue::all(Some(organization.owner_user_id), &project.get_connection()).unwrap();
    assert_eq!(all_venues, all_found_venues);
}

#[test]
fn find_via_org() {
    let project = TestProject::new();

    let venue = project
        .create_venue()
        .with_name("Venue 1".to_string())
        .finish();
    let venue2 = project
        .create_venue()
        .with_name("Venue 2".to_string())
        .finish();
    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();

    let venue = venue
        .add_to_organization(&organization.id, project.get_connection())
        .unwrap();
    let venue2 = venue2
        .add_to_organization(&organization.id, project.get_connection())
        .unwrap();

    let all_venues = vec![venue, venue2];

    let found_venues = organization.venues(project.get_connection()).unwrap();
    assert_eq!(found_venues, all_venues);
    let found_venues =
        Venue::find_for_organization(None, organization.id, project.get_connection()).unwrap();
    assert_eq!(found_venues, all_venues);

    // Add another venue for another org to make sure it isn't included
    let other_venue = project.create_venue().finish();
    let organization2 = project.create_organization().with_owner(&user).finish();
    let other_venue = other_venue
        .add_to_organization(&organization2.id, project.get_connection())
        .unwrap();

    let found_venues =
        Venue::find_for_organization(None, organization.id, project.get_connection()).unwrap();
    assert_eq!(found_venues, all_venues);
    assert!(!found_venues.contains(&other_venue));
}

#[test]
fn organization() {
    let project = TestProject::new();
    let organization = project.create_organization().finish();
    let venue = project
        .create_venue()
        .with_organization(&organization)
        .finish();
    let venue2 = project.create_venue().finish();

    assert_eq!(
        Ok(Some(organization)),
        venue.organization(project.get_connection())
    );
    assert_eq!(Ok(None), venue2.organization(project.get_connection()));
}
