use bigneon_db::models::Venue;
use bigneon_db::utils::errors::*;
use support::project::TestProject;
use uuid::Uuid;

#[test]
fn create() {
    let project = TestProject::new();
    let venue = Venue::create("Name").commit(&project).unwrap();

    assert_eq!(venue.name, venue.name);
    assert_eq!(venue.id.to_string().is_empty(), false);
}

#[test]
fn update() {
    let project = TestProject::new();
    //Edit Venue
    let mut venue = Venue::create("NewVenue").commit(&project).unwrap();
    venue.address = Some(<String>::from("Test Address"));
    venue.city = Some(<String>::from("Test Address"));
    venue.state = Some(<String>::from("Test state"));
    venue.country = Some(<String>::from("Test country"));
    venue.zip = Some(<String>::from("0124"));
    venue.phone = Some(<String>::from("+27123456789"));
    let updated_venue = Venue::update(&venue, &project).unwrap();
    assert_eq!(venue, updated_venue);
}

#[test]
fn find() {
    let project = TestProject::new();
    //create Venue
    let mut edited_venue = Venue::create("VenueForFindTest").commit(&project).unwrap();
    edited_venue.address = Some(<String>::from("Test Address"));
    edited_venue.city = Some(<String>::from("Test Address"));
    edited_venue.state = Some(<String>::from("Test state"));
    edited_venue.country = Some(<String>::from("Test country"));
    edited_venue.zip = Some(<String>::from("0124"));
    edited_venue.phone = Some(<String>::from("+27123456789"));
    //find venue
    let _updated_organization = Venue::update(&edited_venue, &project).unwrap();
    let found_organization = Venue::find(&edited_venue.id, &project).unwrap();
    assert_eq!(edited_venue, found_organization);

    //find more than one venue
    let mut edited_venue2 = Venue::create("VenueForFindTest2").commit(&project).unwrap();
    edited_venue2.address = Some(<String>::from("Test Address2"));
    edited_venue2.city = Some(<String>::from("Test Address2"));
    edited_venue2.state = Some(<String>::from("Test state2"));
    edited_venue2.country = Some(<String>::from("Test country2"));
    edited_venue2.zip = Some(<String>::from("0125"));
    edited_venue2.phone = Some(<String>::from("+27123456780"));
    let _updated_venue = Venue::update(&edited_venue2, &project).unwrap();
    let all_found_venues = Venue::all(&project).unwrap();
    let all_venues = vec![edited_venue, edited_venue2];
    assert_eq!(all_venues, all_found_venues);
}

#[test]
fn has_organization() {
    let project = TestProject::new();
    let venue = Venue::create("Name").commit(&project).unwrap();
    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let organization2 = project.create_organization().with_owner(&user).finish();
    assert!(!venue.has_organization(organization.id, &project).unwrap());
    assert!(!venue.has_organization(organization2.id, &project).unwrap());

    venue
        .add_to_organization(&organization.id, &project)
        .unwrap();

    assert!(venue.has_organization(organization.id, &project).unwrap());
    assert!(!venue.has_organization(organization2.id, &project).unwrap());
}

#[test]
fn create_find_via_org() {
    let project = TestProject::new();
    //create Venues
    let mut edited_venue = Venue::create("VenueForOrgTest").commit(&project).unwrap();
    edited_venue.address = Some(<String>::from("Test Address"));
    edited_venue.city = Some(<String>::from("Test Address"));
    edited_venue.state = Some(<String>::from("Test state"));
    edited_venue.country = Some(<String>::from("Test country"));
    edited_venue.zip = Some(<String>::from("0124"));
    edited_venue.phone = Some(<String>::from("+27123456789"));
    let updated_venue = Venue::update(&edited_venue, &project).unwrap();
    let mut edited_venue2 = Venue::create("VenueForOrgTest").commit(&project).unwrap();
    edited_venue2.address = Some(<String>::from("Test Address"));
    edited_venue2.city = Some(<String>::from("Test Address"));
    edited_venue2.state = Some(<String>::from("Test state"));
    edited_venue2.country = Some(<String>::from("Test country"));
    edited_venue2.zip = Some(<String>::from("0124"));
    edited_venue2.phone = Some(<String>::from("+27123456789"));
    let updated_venue2 = Venue::update(&edited_venue2, &project).unwrap();
    //create user
    let user = project.create_user().finish();
    //create organization
    let organization = project.create_organization().with_owner(&user).finish();

    //Do linking
    let _org_venue_link = updated_venue
        .add_to_organization(&organization.id, &project)
        .unwrap();
    let _org_venue_link = updated_venue2
        .add_to_organization(&organization.id, &project)
        .unwrap();
    let all_venues = vec![updated_venue, updated_venue2];

    let found_venues = organization.venues(&project).unwrap();
    assert_eq!(found_venues, all_venues);
    let found_venues = Venue::find_for_organization(organization.id, &project).unwrap();
    assert_eq!(found_venues, all_venues);

    // Add another venue for another org to make sure it isn't included
    let other_venue = Venue::create("VenueNotInOrg").commit(&project).unwrap();
    let organization2 = project.create_organization().with_owner(&user).finish();
    other_venue
        .add_to_organization(&organization2.id, &project)
        .unwrap();

    let found_venues = Venue::find_for_organization(organization.id, &project).unwrap();
    assert_eq!(found_venues, all_venues);
    assert!(!found_venues.contains(&other_venue));
}
