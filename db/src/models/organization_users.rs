use chrono::NaiveDateTime;
use diesel;
use diesel::prelude::*;
use models::enums::Roles;
use models::{Organization, User};
use schema::{event_users, events, organization_users};
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use utils::errors::*;
use uuid::Uuid;

#[derive(Associations, Identifiable, Queryable, Serialize)]
#[belongs_to(User)]
#[belongs_to(Organization)]
#[table_name = "organization_users"]
pub struct OrganizationUser {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub role: Vec<Roles>,
}

#[derive(Insertable)]
#[table_name = "organization_users"]
pub struct NewOrganizationUser {
    pub organization_id: Uuid,
    pub user_id: Uuid,
    role: Vec<Roles>,
}

impl NewOrganizationUser {
    pub fn commit(self, conn: &PgConnection) -> Result<OrganizationUser, DatabaseError> {
        let existing_user = OrganizationUser::find_by_user_id(self.user_id, self.organization_id, conn).optional()?;
        match existing_user {
            Some(mut user) => {
                // Merge roles
                user.role.extend(self.role);
                user.role.sort();
                user.role.dedup();

                diesel::update(organization_users::table.filter(organization_users::id.eq(user.id)))
                    .set((organization_users::role.eq(user.role),))
                    .get_result(conn)
                    .to_db_error(
                        ErrorCode::UpdateError,
                        "Could not add role to existing organization user",
                    )
            }
            None => DatabaseError::wrap(
                ErrorCode::InsertError,
                "Could not create new organization user",
                diesel::insert_into(organization_users::table)
                    .values(self)
                    .get_result(conn),
            ),
        }
    }
}

impl OrganizationUser {
    pub fn create(organization_id: Uuid, user_id: Uuid, role: Vec<Roles>) -> NewOrganizationUser {
        NewOrganizationUser {
            organization_id,
            user_id,
            role,
        }
    }

    pub fn event_ids(&self, conn: &PgConnection) -> Result<Vec<Uuid>, DatabaseError> {
        organization_users::table
            .inner_join(events::table.on(events::organization_id.eq(organization_users::organization_id)))
            .inner_join(
                event_users::table.on(event_users::user_id
                    .eq(self.user_id)
                    .and(event_users::event_id.eq(events::id))),
            )
            .filter(organization_users::organization_id.eq(self.organization_id))
            .select(event_users::event_id)
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Could not load organization user event ids")
    }

    pub fn find_users_by_organization(
        organization_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<OrganizationUser>, DatabaseError> {
        organization_users::table
            .filter(organization_users::organization_id.eq(organization_id))
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Could not load organization users")
    }

    pub fn find_by_user_id(
        user_id: Uuid,
        organization_id: Uuid,
        conn: &PgConnection,
    ) -> Result<OrganizationUser, DatabaseError> {
        organization_users::table
            .filter(
                organization_users::user_id
                    .eq(user_id)
                    .and(organization_users::organization_id.eq(organization_id)),
            )
            .first(conn)
            .to_db_error(ErrorCode::QueryError, "Could not find user for organization")
    }
}
