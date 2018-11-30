use chrono::NaiveDateTime;
use diesel;
use diesel::prelude::*;
use models::enums::Roles;
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
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "organization_users"]
pub struct NewOrganizationUser {
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
}

impl NewOrganizationUser {
    pub fn commit(&self, conn: &PgConnection) -> Result<OrganizationUser, DatabaseError> {
        match self.validate_record() {
            Ok(_) => (),
            Err(_e) => {
                return Err(DatabaseError::new(
                    ErrorCode::InvalidInput,
                    Some("Invalid user role supplied".to_string()),
                ));
            }
        }
        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new organization user",
            diesel::insert_into(organization_users::table)
                .values(self)
                .get_result(conn),
        )
    }

    pub fn validate_record(&self) -> Result<(), EnumParseError> {
        let _: Roles = self.role.parse()?;
        Ok(())
    }
}

impl OrganizationUser {
    pub fn create(
        organization_id: Uuid,
        user_id: Uuid,
        role: Option<Roles>,
    ) -> NewOrganizationUser {
        let role = role.unwrap_or(Roles::OrgMember).to_string();
        NewOrganizationUser {
            organization_id,
            user_id,
            role,
        }
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
}
