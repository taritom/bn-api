use chrono::NaiveDateTime;
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use models::*;
use schema::stages;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Clone, Associations, Identifiable, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[belongs_to(Venue)]
#[table_name = "stages"]
pub struct Stage {
    pub id: Uuid,
    pub venue_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub capacity: Option<i64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(AsChangeset, Default, Deserialize)]
#[table_name = "stages"]
pub struct StageEditableAttributes {
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "double_option_deserialize_unless_blank")]
    pub description: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option_deserialize_unless_blank")]
    pub capacity: Option<Option<i64>>,
}

#[derive(Default, Insertable, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[table_name = "stages"]
pub struct NewStage {
    pub venue_id: Uuid,
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub description: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub capacity: Option<i64>,
}

impl NewStage {
    pub fn commit(&self, connection: &PgConnection) -> Result<Stage, DatabaseError> {
        diesel::insert_into(stages::table)
            .values(self)
            .get_result(connection)
            .to_db_error(ErrorCode::InsertError, "Could not create stage")
    }
}

impl Stage {
    pub fn create(venue_id: Uuid, name: String, description: Option<String>, capacity: Option<i64>) -> NewStage {
        NewStage {
            venue_id,
            name,
            description,
            capacity,
        }
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<Stage, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading stage",
            stages::table.find(id).first::<Stage>(conn),
        )
    }

    pub fn find_by_venue_id(venue_id: Uuid, conn: &PgConnection) -> Result<Vec<Stage>, DatabaseError> {
        stages::table
            .filter(stages::venue_id.eq(venue_id))
            .order_by(stages::name)
            .select(stages::all_columns)
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load all stages")
    }

    pub fn update(&self, attributes: StageEditableAttributes, conn: &PgConnection) -> Result<Stage, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update stage",
            diesel::update(self)
                .set((attributes, stages::updated_at.eq(dsl::now)))
                .get_result(conn),
        )
    }

    pub fn destroy(&self, conn: &PgConnection) -> Result<usize, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::DeleteError,
            "Failed to delete stage",
            diesel::delete(self).execute(conn),
        )
    }
}
