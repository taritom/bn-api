use db::Connectable;
use diesel;
use diesel::prelude::*;
use models::*;
use schema::{organization_users, organizations, users, venues};
use utils::errors::*;
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
    pub postal_code: Option<String>,
    pub phone: Option<String>,
    pub fee_schedule_id: Option<Uuid>,
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
    pub postal_code: Option<String>,
    pub phone: Option<String>,
}

impl NewOrganization {
    pub fn commit(&self, conn: &Connectable) -> Result<Organization, DatabaseError> {
        let db_err = diesel::insert_into(organizations::table)
            .values(self)
            .get_result(conn.get_connection())
            .to_db_error(ErrorCode::InsertError, "Could not create new organization")?;

        //Would not have gotten here if the user_id did not exist
        let _ = User::find(self.owner_user_id, conn)?.add_role(Roles::OrgOwner, conn)?;

        Ok(db_err)
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
    pub postal_code: Option<String>,
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
            postal_code: None,
            phone: None,
        }
    }

    pub fn update(
        &self,
        attributes: OrganizationEditableAttributes,
        conn: &Connectable,
    ) -> Result<Organization, DatabaseError> {
        diesel::update(self)
            .set(attributes)
            .get_result(conn.get_connection())
            .to_db_error(ErrorCode::UpdateError, "Could not update organization")
    }

    pub fn set_owner(
        &self,
        owner_user_id: Uuid,
        conn: &Connectable,
    ) -> Result<Organization, DatabaseError> {
        let old_owner_id = self.owner_user_id;

        let db_result = diesel::update(self)
            .set(organizations::owner_user_id.eq(owner_user_id))
            .get_result(conn.get_connection())
            .to_db_error(
                ErrorCode::UpdateError,
                "Could not update organization owner",
            )?;

        //Not checking this result as this user has to exist to get to this point.
        User::find(owner_user_id, conn)
            .unwrap()
            .add_role(Roles::OrgOwner, conn)?;

        //Check if the old owner is the owner of any OTHER orgs
        let result = organizations::table
            .filter(organizations::owner_user_id.eq(old_owner_id))
            .filter(organizations::id.ne(&self.id))
            .load::<Organization>(conn.get_connection())
            .to_db_error(ErrorCode::NoResults, "Could not search for organisations")?;
        //If not then remove the OrgOwner Role.
        if result.len() == 0 {
            //Not handling the returned Result as the old user had to exist
            User::find(old_owner_id, conn)
                .unwrap()
                .remove_role(Roles::OrgOwner, conn)?;
        }

        Ok(db_result)
    }

    pub fn users(&self, conn: &Connectable) -> Result<Vec<User>, DatabaseError> {
        let organization_users = OrganizationUser::belonging_to(self);
        let organization_owner = users::table
            .find(self.owner_user_id)
            .first::<User>(conn.get_connection())
            .to_db_error(
                ErrorCode::QueryError,
                "Could not retrieve organization users",
            )?;
        let mut users = organization_users
            .inner_join(users::table)
            .filter(users::id.ne(self.owner_user_id))
            .select(users::all_columns)
            .load::<User>(conn.get_connection())
            .to_db_error(
                ErrorCode::QueryError,
                "Could not retrieve organization users",
            )?;

        users.insert(0, organization_owner);
        Ok(users)
    }

    pub fn venues(&self, conn: &Connectable) -> Result<Vec<Venue>, DatabaseError> {
        let organization_venues = OrganizationVenue::belonging_to(self);

        organization_venues
            .inner_join(venues::table)
            .select(venues::all_columns)
            .load::<Venue>(conn.get_connection())
            .to_db_error(ErrorCode::QueryError, "Could not retrieve venues")
    }

    pub fn find(id: Uuid, conn: &Connectable) -> Result<Organization, DatabaseError> {
        organizations::table
            .find(id)
            .first::<Organization>(conn.get_connection())
            .to_db_error(ErrorCode::QueryError, "Error loading organization")
    }

    pub fn all(conn: &Connectable) -> Result<Vec<Organization>, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Unable to load all organizations",
            organizations::table
                .order_by(organizations::name)
                .load(conn.get_connection()),
        )
    }

    pub fn all_linked_to_user(
        user_id: Uuid,
        conn: &Connectable,
    ) -> Result<Vec<Organization>, DatabaseError> {
        let orgs = organizations::table
            .filter(organizations::owner_user_id.eq(user_id))
            .load(conn.get_connection())
            .to_db_error(ErrorCode::QueryError, "Unable to load all organizations");

        let mut orgs = match orgs {
            Ok(o) => o,
            Err(e) => return Err(e),
        };

        let org_members = organization_users::table
            .filter(organization_users::user_id.eq(user_id))
            .inner_join(organizations::table)
            .select(organizations::all_columns)
            .load::<Organization>(conn.get_connection())
            .to_db_error(ErrorCode::QueryError, "Unable to load all organizations");

        let mut org_members = match org_members {
            Ok(o) => o,
            Err(e) => return Err(e),
        };

        orgs.append(&mut org_members);
        orgs.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(orgs)
    }

    pub fn remove_user(&self, user_id: Uuid, conn: &Connectable) -> Result<usize, DatabaseError> {
        let rows_affected = diesel::delete(
            organization_users::table
                .filter(organization_users::user_id.eq(user_id))
                .filter(organization_users::organization_id.eq(self.id)),
        ).execute(conn.get_connection())
            .to_db_error(ErrorCode::DeleteError, "Error removing user")?;

        if Organization::all_linked_to_user(user_id, conn)?.len() == 0 {
            let user = User::find(user_id, conn)?;
            user.remove_role(Roles::OrgMember, conn)?;
        }

        Ok(rows_affected)
    }

    pub fn add_user(
        &self,
        user_id: Uuid,
        conn: &Connectable,
    ) -> Result<OrganizationUser, DatabaseError> {
        let org_user = OrganizationUser::create(self.id, user_id).commit(conn)?;
        let user = User::find(user_id, conn)?;
        user.add_role(Roles::OrgMember, conn)?;
        Ok(org_user)
    }

    pub fn is_member(&self, user: &User, conn: &Connectable) -> Result<bool, DatabaseError> {
        Ok(self.users(conn)?.contains(&user))
    }

    pub fn add_fee_schedule(
        &self,
        fee_schedule: &FeeSchedule,
        conn: &Connectable,
    ) -> Result<(), DatabaseError> {
        diesel::update(self)
            .set(organizations::fee_schedule_id.eq(fee_schedule.id))
            .execute(conn.get_connection())
            .to_db_error(
                ErrorCode::UpdateError,
                "Could not set the fee schedule for this organization",
            )?;
        Ok(())
    }
}
