use db::Connectable;
use diesel;
use diesel::prelude::*;
use models::{OrganizationUser, OrganizationVenue, User, Venue};
use schema::{organization_users, organization_venues, organizations, users, venues};
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Identifiable, Associations, Queryable)]
#[belongs_to(User, foreign_key = "owner_user_id")]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "organizations"]
pub struct Organization {
    pub id: Uuid,
    pub owner_user_id: Uuid,
    pub name: String,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub zip: Option<String>,
    pub phone: Option<String>,
}

#[derive(Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "organizations"]
pub struct NewOrganization {
    pub owner_user_id: Uuid,
    pub name: String,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub zip: Option<String>,
    pub phone: Option<String>,
}

impl NewOrganization {
    pub fn commit(&self, conn: &Connectable) -> Result<Organization, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new organization",
            diesel::insert_into(organizations::table)
                .values(self)
                .get_result(conn.get_connection()),
        )
    }
}

#[derive(AsChangeset, Default, Deserialize)]
#[table_name = "organizations"]
pub struct OrganizationEditableAttributes {
    pub name: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub zip: Option<String>,
    pub phone: Option<String>,
}

impl Organization {
    pub fn create(owner_user_id: Uuid, name: &str) -> NewOrganization {
        NewOrganization {
            owner_user_id: owner_user_id,
            name: name.into(),
            address: None,
            city: None,
            state: None,
            country: None,
            zip: None,
            phone: None,
        }
    }

    pub fn update(
        &self,
        attributes: OrganizationEditableAttributes,
        conn: &Connectable,
    ) -> Result<Organization, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update organization",
            diesel::update(self)
                .set(attributes)
                .get_result(conn.get_connection()),
        )
    }

    pub fn set_owner(
        &self,
        owner_user_id: Uuid,
        conn: &Connectable,
    ) -> Result<Organization, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update organization owner",
            diesel::update(self)
                .set(organizations::owner_user_id.eq(owner_user_id))
                .get_result(conn.get_connection()),
        )
    }

    pub fn users(&self, conn: &Connectable) -> Vec<User> {
        let organization_users = OrganizationUser::belonging_to(self);
        let organization_owner = users::table
            .find(self.owner_user_id)
            .first::<User>(conn.get_connection())
            .expect("Error loading organization owner");
        let mut users = organization_users
            .inner_join(users::table)
            .filter(users::id.ne(self.owner_user_id))
            .select(users::all_columns)
            .load::<User>(conn.get_connection())
            .expect("Error loading organization users");

        users.insert(0, organization_owner);
        users
    }

    pub fn venues(&self, conn: &Connectable) -> Result<Vec<Venue>, DatabaseError> {
        let organization_venues = OrganizationVenue::belonging_to(self);

        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Could not retrieve venues",
            organization_venues
                .inner_join(venues::table)
                .select(venues::all_columns)
                .load::<Venue>(conn.get_connection()),
        )
    }

    pub fn find(id: &Uuid, conn: &Connectable) -> Result<Organization, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading organization",
            organizations::table
                .find(id)
                .first::<Organization>(conn.get_connection()),
        )
    }

    pub fn all(conn: &Connectable) -> Result<Vec<Organization>, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Unable to load all organizations",
            organizations::table.load(conn.get_connection()),
        )
    }

    pub fn all_linked_to_user(
        user_id: Uuid,
        conn: &Connectable,
    ) -> Result<Vec<Organization>, DatabaseError> {
        let orgs = DatabaseError::wrap(
            ErrorCode::QueryError,
            "Unable to load all organizations",
            organizations::table
                .filter(organizations::owner_user_id.eq(user_id))
                .load(conn.get_connection()),
        );

        let mut orgs = match orgs {
            Ok(o) => o,
            Err(e) => return Err(e),
        };

        let org_members = DatabaseError::wrap(
            ErrorCode::QueryError,
            "Unable to load all organizations",
            organization_users::table
                .filter(organization_users::user_id.eq(user_id))
                .inner_join(organizations::table)
                .select(organizations::all_columns)
                .load::<Organization>(conn.get_connection()),
        );

        let mut org_members = match org_members {
            Ok(o) => o,
            Err(e) => return Err(e),
        };

        orgs.append(&mut org_members);
        orgs.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(orgs)
    }
    pub fn remove_user(&self, user_id: &Uuid, conn: &Connectable) -> Result<usize, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading organization",
            diesel::delete(
                organization_users::table
                    .filter(organization_users::user_id.eq(user_id))
                    .filter(organization_users::organization_id.eq(self.id)),
            ).execute(conn.get_connection()),
        )
    }
}
