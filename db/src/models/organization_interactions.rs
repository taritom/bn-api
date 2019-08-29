use chrono::NaiveDateTime;
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use schema::organization_interactions;
use utils::errors::ConvertToDatabaseError;
use utils::errors::{DatabaseError, ErrorCode};
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable, PartialEq, Serialize)]
#[table_name = "organization_interactions"]
pub struct OrganizationInteraction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub first_interaction: NaiveDateTime,
    pub last_interaction: NaiveDateTime,
    pub interaction_count: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, PartialEq, Debug, Deserialize)]
#[table_name = "organization_interactions"]
pub struct NewOrganizationInteraction {
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub first_interaction: NaiveDateTime,
    pub last_interaction: NaiveDateTime,
    pub interaction_count: i64,
}

#[derive(AsChangeset, Default, Deserialize)]
#[table_name = "organization_interactions"]
pub struct OrganizationInteractionEditableAttributes {
    pub first_interaction: Option<NaiveDateTime>,
    pub last_interaction: Option<NaiveDateTime>,
    pub interaction_count: Option<i64>,
}

impl NewOrganizationInteraction {
    pub fn commit(
        &mut self,
        conn: &PgConnection,
    ) -> Result<OrganizationInteraction, DatabaseError> {
        diesel::insert_into(organization_interactions::table)
            .values(&*self)
            .get_result(conn)
            .to_db_error(
                ErrorCode::InsertError,
                "Could not create new interaction table row",
            )
    }
}

impl OrganizationInteraction {
    pub fn create(
        organization_id: Uuid,
        user_id: Uuid,
        first_interaction: NaiveDateTime,
        last_interaction: NaiveDateTime,
        interaction_count: i64,
    ) -> NewOrganizationInteraction {
        NewOrganizationInteraction {
            organization_id,
            user_id,
            first_interaction,
            last_interaction,
            interaction_count,
        }
    }

    pub fn find_by_organization_user(
        organization_id: Uuid,
        user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<OrganizationInteraction, DatabaseError> {
        organization_interactions::table
            .filter(organization_interactions::organization_id.eq(organization_id))
            .filter(organization_interactions::user_id.eq(user_id))
            .first(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not retrieve organization user interactions",
            )
    }

    pub fn update(
        &self,
        attributes: &OrganizationInteractionEditableAttributes,
        conn: &PgConnection,
    ) -> Result<OrganizationInteraction, DatabaseError> {
        diesel::update(self)
            .set((
                attributes,
                organization_interactions::updated_at.eq(dsl::now),
            ))
            .get_result(conn)
            .to_db_error(
                ErrorCode::UpdateError,
                "Error updating organization interaction",
            )
    }
}
