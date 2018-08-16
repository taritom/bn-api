use bigneon_db::models::{OrganizationVenue, Venue};
use support::project::TestProject;
use uuid::Uuid;

#[test]
fn create() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let venue = Venue::create("Name").commit(&project).unwrap();
    let organization = project.create_organization().with_owner(&user).finish();
    let organization_venue = OrganizationVenue::create(organization.id, venue.id)
        .commit(&project)
        .unwrap();

    assert_eq!(organization_venue.venue_id, venue.id);
    assert_eq!(organization_venue.organization_id, organization.id);
    assert_eq!(organization_venue.id.to_string().is_empty(), false);
}

#[test]
fn find() {
    //create user and organization
    let project = TestProject::new();
    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();

    //create Venue
    let mut edited_venue = Venue::create("VenueForFindTest").commit(&project).unwrap();
    edited_venue.address = Some(<String>::from("Test Address"));
    edited_venue.city = Some(<String>::from("Test Address"));
    edited_venue.state = Some(<String>::from("Test state"));
    edited_venue.country = Some(<String>::from("Test country"));
    edited_venue.zip = Some(<String>::from("0124"));
    edited_venue.phone = Some(<String>::from("+27123456789"));
    let updated_venue = Venue::update(&edited_venue, &project).unwrap();
    //create organization>venue link
    let _organization_venue = OrganizationVenue::create(organization.id, updated_venue.id)
        .commit(&project)
        .unwrap();
    //find organization linked to venue
    let organization_venue =
        OrganizationVenue::find_via_venue_all(&updated_venue.id, &project).unwrap();
    assert_eq!(organization_venue[0].organization_id, organization.id);
    let found_venue = Venue::find(&organization_venue[0].venue_id, &project).unwrap();
    assert_eq!(found_venue, updated_venue);
}
#[test]
fn find_lists() {
    //create user and organization
    let project = TestProject::new();
    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let organization2 = project.create_organization().with_owner(&user).finish();
    let all_organizations = vec![organization.id, organization2.id];

    //create Venue
    let mut edited_venue = Venue::create("VenueForFindTest").commit(&project).unwrap();
    edited_venue.address = Some(<String>::from("Test Address"));
    edited_venue.city = Some(<String>::from("Test Address"));
    edited_venue.state = Some(<String>::from("Test state"));
    edited_venue.country = Some(<String>::from("Test country"));
    edited_venue.zip = Some(<String>::from("0124"));
    edited_venue.phone = Some(<String>::from("+27123456789"));
    let updated_venue = Venue::update(&edited_venue, &project).unwrap();
    let mut edited_venue2 = Venue::create("VenueForFindTest").commit(&project).unwrap();
    edited_venue2.address = Some(<String>::from("Test Address"));
    edited_venue2.city = Some(<String>::from("Test Address"));
    edited_venue2.state = Some(<String>::from("Test state"));
    edited_venue2.country = Some(<String>::from("Test country"));
    edited_venue2.zip = Some(<String>::from("0124"));
    edited_venue2.phone = Some(<String>::from("+27123456789"));
    let updated_venue2 = Venue::update(&edited_venue2, &project).unwrap();
    let all_venues = vec![updated_venue, updated_venue2];

    //create organization > venue links
    let _organization_venue = OrganizationVenue::create(all_organizations[0], all_venues[0].id)
        .commit(&project)
        .unwrap();
    let _organization_venue = OrganizationVenue::create(all_organizations[0], all_venues[1].id)
        .commit(&project)
        .unwrap();
    let _organization_venue = OrganizationVenue::create(all_organizations[1], all_venues[0].id)
        .commit(&project)
        .unwrap();
    let _organization_venue = OrganizationVenue::create(all_organizations[1], all_venues[1].id)
        .commit(&project)
        .unwrap();
    //find organization linked to venue
    let organization_ids =
        OrganizationVenue::find_via_venue_all(&all_venues[0].id, &project).unwrap();
    //let mut found_organizations: Vec<Organization> = Vec::new();

    let organization_ids: Vec<Uuid> = organization_ids
        .iter()
        .map(|i| {
            let x: Uuid = i.organization_id;
            x
        })
        .collect();
    assert_eq!(organization_ids, all_organizations);
}
