use chrono::NaiveDateTime;
use diesel;
use diesel::dsl::{exists, select};
use diesel::expression::dsl;
use diesel::prelude::*;
use models::scopes;
use models::*;
use schema::{organization_users, organizations, users, venues};
use utils::errors::*;
use uuid::Uuid;

#[derive(Identifiable, Associations, Queryable, AsChangeset)]
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
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub fee_schedule_id: Uuid,
}

#[derive(Serialize)]
pub struct DisplayOrganizationLink {
    pub id: Uuid,
    pub name: String,
    pub role: String,
}

#[derive(Default, Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "organizations"]
pub struct NewOrganization {
    pub owner_user_id: Uuid,
    pub name: String,
    pub fee_schedule_id: Uuid,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub phone: Option<String>,
}

impl NewOrganization {
    pub fn commit(&self, conn: &PgConnection) -> Result<Organization, DatabaseError> {
        let db_err = diesel::insert_into(organizations::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create new organization")?;

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
    pub fee_schedule_id: Option<Uuid>,
}

impl Organization {
    pub fn create(owner_user_id: Uuid, name: &str, fee_schedule_id: Uuid) -> NewOrganization {
        NewOrganization {
            owner_user_id: owner_user_id,
            name: name.into(),
            fee_schedule_id,
            ..Default::default()
        }
    }

    pub fn update(
        &self,
        attributes: OrganizationEditableAttributes,
        conn: &PgConnection,
    ) -> Result<Organization, DatabaseError> {
        diesel::update(self)
            .set((attributes, organizations::updated_at.eq(dsl::now)))
            .get_result(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not update organization")
    }

    pub fn set_owner(
        &self,
        owner_user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Organization, DatabaseError> {
        diesel::update(self)
            .set((
                organizations::owner_user_id.eq(owner_user_id),
                organizations::updated_at.eq(dsl::now),
            )).get_result(conn)
            .to_db_error(
                ErrorCode::UpdateError,
                "Could not update organization owner",
            )
    }

    pub fn users(&self, conn: &PgConnection) -> Result<Vec<User>, DatabaseError> {
        let organization_users = OrganizationUser::belonging_to(self);
        let organization_owner = users::table
            .find(self.owner_user_id)
            .first::<User>(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not retrieve organization users",
            )?;
        let mut users = organization_users
            .inner_join(users::table)
            .filter(users::id.ne(self.owner_user_id))
            .select(users::all_columns)
            .order_by(users::last_name.asc())
            .then_order_by(users::first_name.asc())
            .load::<User>(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not retrieve organization users",
            )?;

        users.insert(0, organization_owner);
        Ok(users)
    }

    pub fn venues(&self, conn: &PgConnection) -> Result<Vec<Venue>, DatabaseError> {
        venues::table
            .filter(venues::organization_id.eq(self.id))
            .order_by(venues::name)
            .get_results(conn)
            .to_db_error(ErrorCode::QueryError, "Could not retrieve venues")
    }

    pub fn get_scopes_for_user(
        &self,
        user: &User,
        conn: &PgConnection,
    ) -> Result<Vec<String>, DatabaseError> {
        Ok(scopes::get_scopes(self.get_roles_for_user(user, conn)?))
    }

    pub fn get_roles_for_user(
        &self,
        user: &User,
        conn: &PgConnection,
    ) -> Result<Vec<String>, DatabaseError> {
        let mut roles = Vec::new();
        if user.id == self.owner_user_id {
            roles.push(Roles::OrgOwner.to_string());
            roles.push(Roles::OrgMember.to_string());
        } else {
            if self.is_member(user, conn)? {
                roles.push(Roles::OrgMember.to_string());
            }
        }

        Ok(roles)
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<Organization, DatabaseError> {
        organizations::table
            .find(id)
            .first::<Organization>(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading organization")
    }

    pub fn all(conn: &PgConnection) -> Result<Vec<Organization>, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Unable to load all organizations",
            organizations::table
                .order_by(organizations::name)
                .load(conn),
        )
    }

    pub fn owner(&self, conn: &PgConnection) -> Result<User, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Unable to load owner",
            users::table.find(self.owner_user_id).first::<User>(conn),
        )
    }

    pub fn all_linked_to_user(
        user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<Organization>, DatabaseError> {
        let orgs = organizations::table
            .filter(organizations::owner_user_id.eq(user_id))
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load all organizations");

        let mut orgs = match orgs {
            Ok(o) => o,
            Err(e) => return Err(e),
        };

        let mut org_members = organization_users::table
            .filter(organization_users::user_id.eq(user_id))
            .inner_join(organizations::table)
            .select(organizations::all_columns)
            .load::<Organization>(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load all organizations")?;

        orgs.append(&mut org_members);
        orgs.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(orgs)
    }

    pub fn all_org_names_linked_to_user(
        user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<DisplayOrganizationLink>, DatabaseError> {
        //Compile list of organisations where the user is the owner
        let org_owner_list: Vec<Organization> = organizations::table
            .filter(organizations::owner_user_id.eq(user_id))
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load all organizations")?;
        //Compile list of organisations where the user is a member of that organisation
        let org_member_list: Vec<Organization> = organization_users::table
            .filter(organization_users::user_id.eq(user_id))
            .inner_join(organizations::table)
            .select(organizations::all_columns)
            .load::<Organization>(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load all organizations")?;
        //Compile complete list with only display information
        let role_owner_string = String::from("owner");
        let role_member_string = String::from("member");
        let mut result_list: Vec<DisplayOrganizationLink> = Vec::new();
        for curr_org_owner in &org_owner_list {
            let curr_entry = DisplayOrganizationLink {
                id: curr_org_owner.id,
                name: curr_org_owner.name.clone(),
                role: role_owner_string.clone(),
            };
            result_list.push(curr_entry);
        }
        for curr_org_member in &org_member_list {
            let curr_entry = DisplayOrganizationLink {
                id: curr_org_member.id,
                name: curr_org_member.name.clone(),
                role: role_member_string.clone(),
            };
            result_list.push(curr_entry);
        }
        result_list.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(result_list)
    }

    pub fn remove_user(&self, user_id: Uuid, conn: &PgConnection) -> Result<usize, DatabaseError> {
        diesel::delete(
            organization_users::table
                .filter(organization_users::user_id.eq(user_id))
                .filter(organization_users::organization_id.eq(self.id)),
        ).execute(conn)
        .to_db_error(ErrorCode::DeleteError, "Error removing user")
    }

    pub fn add_user(
        &self,
        user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<OrganizationUser, DatabaseError> {
        let org_user = OrganizationUser::create(self.id, user_id).commit(conn)?;
        Ok(org_user)
    }

    pub fn is_member(&self, user: &User, conn: &PgConnection) -> Result<bool, DatabaseError> {
        if self.owner_user_id == user.id {
            return Ok(true);
        }

        let query = select(exists(
            organization_users::table
                .filter(
                    organization_users::user_id
                        .eq(user.id)
                        .and(organization_users::organization_id.eq(self.id)),
                ).select(organization_users::organization_id),
        ));
        query
            .get_result(conn)
            .to_db_error(ErrorCode::QueryError, "Could not check user member status")
    }

    pub fn add_fee_schedule(
        &self,
        fee_schedule: &FeeSchedule,
        conn: &PgConnection,
    ) -> Result<Organization, DatabaseError> {
        let attributes = OrganizationEditableAttributes {
            fee_schedule_id: Some(fee_schedule.id),
            ..Default::default()
        };
        diesel::update(self)
            .set((attributes, organizations::updated_at.eq(dsl::now)))
            .get_result(conn)
            .to_db_error(
                ErrorCode::UpdateError,
                "Could not set the fee schedule for this organization",
            )
    }
}
