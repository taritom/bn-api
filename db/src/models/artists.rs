use db::connections::Connectable;
use diesel;
use diesel::prelude::*;
use schema::artists;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Associations, Deserialize, Identifiable, Queryable, Serialize)]
pub struct Artist {
    pub id: Uuid,
    pub name: String,
}

#[derive(Insertable, Deserialize)]
#[table_name = "artists"]
pub struct NewArtist {
    pub name: String,
}

#[derive(AsChangeset, Deserialize)]
#[table_name = "artists"]
pub struct UserEditableAttributes {
    pub name: String,
}

impl NewArtist {
    pub fn commit(&self, conn: &Connectable) -> Result<Artist, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new artist",
            diesel::insert_into(artists::table)
                .values(self)
                .get_result(conn.get_connection()),
        )
    }
}

impl Artist {
    pub fn create(name: &str) -> NewArtist {
        NewArtist {
            name: String::from(name),
        }
    }

    pub fn all(conn: &Connectable) -> Result<Vec<Artist>, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Unable to load artists",
            artists::table.load(conn.get_connection()),
        )
    }

    pub fn find(id: &Uuid, conn: &Connectable) -> Result<Artist, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading artist",
            artists::table
                .find(id)
                .first::<Artist>(conn.get_connection()),
        )
    }

    pub fn update(
        &self,
        attributes: &UserEditableAttributes,
        conn: &Connectable,
    ) -> Result<Artist, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Error updating artist",
            diesel::update(self)
                .set(attributes)
                .get_result(conn.get_connection()),
        )
    }

    pub fn destroy(&self, conn: &Connectable) -> Result<usize, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::DeleteError,
            "Failed to destroy artist record",
            diesel::delete(self).execute(conn.get_connection()),
        )
    }
}
