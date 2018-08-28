use db::Connectable;
use diesel;
use diesel::prelude::*;
use models::{Organization, User};
use schema::organization_users;
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
}

#[derive(Insertable)]
#[table_name = "organization_users"]
pub struct NewOrganizationUser {
    pub organization_id: Uuid,
    pub user_id: Uuid,
}

impl NewOrganizationUser {
    pub fn commit(&self, conn: &Connectable) -> Result<OrganizationUser, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new organization user",
            diesel::insert_into(organization_users::table)
                .values(self)
                .get_result(conn.get_connection()),
        )
    }
}

impl OrganizationUser {
    pub fn create(organization_id: Uuid, user_id: Uuid) -> NewOrganizationUser {
        NewOrganizationUser {
            organization_id,
            user_id,
        }
    }

    pub fn find_users_by_organization(
        organization_id: Uuid,
        conn: &Connectable,
    ) -> Result<Vec<OrganizationUser>, DatabaseError> {
        organization_users::table
            .filter(organization_users::organization_id.eq(organization_id))
            .load(conn.get_connection())
            .to_db_error(ErrorCode::QueryError, "Could not load organization users")
    }
}
