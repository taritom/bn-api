use bigneon_db::models::{OrganizationVenue, Venue};
use support::project::TestProject;
use uuid::Uuid;

#[test]
fn create() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let venue = project.create_venue().finish();
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
    let project = TestProject::new();
    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let venue = project.create_venue().finish();

    OrganizationVenue::create(organization.id, venue.id)
        .commit(&project)
        .unwrap();
    //find organization linked to venue
    let organization_venue = OrganizationVenue::find_via_venue_all(&venue.id, &project).unwrap();
    assert_eq!(organization_venue[0].organization_id, organization.id);
    let found_venue = Venue::find(&organization_venue[0].venue_id, &project).unwrap();
    assert_eq!(found_venue, venue);
}
#[test]
fn find_lists() {
    //create user and organization
    let project = TestProject::new();
    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let organization2 = project.create_organization().with_owner(&user).finish();
    let all_organizations = vec![organization.id, organization2.id];

    let venue = project.create_venue().finish();
    let venue2 = project.create_venue().finish();
    let all_venues = vec![venue, venue2];

    //create organization > venue links
    OrganizationVenue::create(all_organizations[0], all_venues[0].id)
        .commit(&project)
        .unwrap();
    OrganizationVenue::create(all_organizations[0], all_venues[1].id)
        .commit(&project)
        .unwrap();
    OrganizationVenue::create(all_organizations[1], all_venues[0].id)
        .commit(&project)
        .unwrap();
    OrganizationVenue::create(all_organizations[1], all_venues[1].id)
        .commit(&project)
        .unwrap();

    //find organization linked to venue
    let organization_ids =
        OrganizationVenue::find_via_venue_all(&all_venues[0].id, &project).unwrap();

    let organization_ids: Vec<Uuid> = organization_ids
        .iter()
        .map(|i| {
            let x: Uuid = i.organization_id;
            x
        })
        .collect();
    assert_eq!(organization_ids, all_organizations);
}
