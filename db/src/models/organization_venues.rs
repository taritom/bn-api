use diesel;
use diesel::prelude::*;
use models::{Organization, Venue};
use schema::organization_venues;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Associations, Identifiable, Queryable, AsChangeset)]
#[belongs_to(Venue, foreign_key = "venue_id")]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[belongs_to(Organization, foreign_key = "organization_id")]
#[table_name = "organization_venues"]
pub struct OrganizationVenue {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub venue_id: Uuid,
}

#[derive(Insertable)]
#[table_name = "organization_venues"]
pub struct NewOrganizationVenue {
    pub organization_id: Uuid,
    pub venue_id: Uuid,
}

impl NewOrganizationVenue {
    pub fn commit(&self, conn: &PgConnection) -> Result<OrganizationVenue, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new organization venue",
            diesel::insert_into(organization_venues::table)
                .values(self)
                .get_result(conn),
        )
    }
}

impl OrganizationVenue {
    pub fn create(organization_id: Uuid, venue_id: Uuid) -> NewOrganizationVenue {
        NewOrganizationVenue {
            organization_id: organization_id,
            venue_id: venue_id,
        }
    }
    pub fn update(&self, conn: &PgConnection) -> Result<OrganizationVenue, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update organization venue link",
            diesel::update(self).set(self).get_result(conn),
        )
    }
    pub fn find(id: &Uuid, conn: &PgConnection) -> Result<OrganizationVenue, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading organization venue link",
            organization_venues::table
                .find(id)
                .first::<OrganizationVenue>(conn),
        )
    }
    pub fn find_via_venue_all(
        venue_id: &Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<OrganizationVenue>, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading event via venue",
            organization_venues::table
                .filter(organization_venues::venue_id.eq(venue_id))
                .load(conn),
        )
    }

    pub fn all(conn: &PgConnection) -> Result<Vec<OrganizationVenue>, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Unable to load all organizations venue links",
            organization_venues::table.load(conn),
        )
    }
}
