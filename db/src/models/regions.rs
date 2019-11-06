use chrono::NaiveDateTime;
use diesel;
use diesel::expression::dsl;
use diesel::expression::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types;
use schema::regions;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Deserialize, Identifiable, Queryable, PartialEq, Debug, Serialize)]
pub struct Region {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(AsChangeset, Default, Deserialize)]
#[table_name = "regions"]
pub struct RegionEditableAttributes {
    pub name: Option<String>,
}

#[derive(Insertable, Deserialize)]
#[table_name = "regions"]
pub struct NewRegion {
    pub name: String,
}

impl Region {
    pub fn create(name: String) -> NewRegion {
        NewRegion { name }
    }

    pub fn update(&self, attributes: RegionEditableAttributes, conn: &PgConnection) -> Result<Region, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update region",
            diesel::update(self)
                .set((attributes, regions::updated_at.eq(dsl::now)))
                .get_result(conn),
        )
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<Region, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading region",
            regions::table.find(id).first::<Region>(conn),
        )
    }

    pub fn all(conn: &PgConnection) -> Result<Vec<Region>, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Unable to load all regions",
            regions::table.then_order_by(regions::name.asc()).load(conn),
        )
    }

    pub fn find_by_name(name: &str, conn: &PgConnection) -> Result<Option<Region>, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading region",
            regions::table
                .filter(sql("LOWER(name)=").bind::<sql_types::Text, _>(name.to_lowercase()))
                .first::<Region>(conn)
                .optional(),
        )
    }
}

impl NewRegion {
    pub fn commit(self, conn: &PgConnection) -> Result<Region, DatabaseError> {
        diesel::insert_into(regions::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create region")
    }
}
