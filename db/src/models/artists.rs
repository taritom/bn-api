use db::Connectable;
use diesel;
use diesel::prelude::*;
use schema::artists;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;
use validator::Validate;
use validators;

#[derive(Associations, Deserialize, Identifiable, Queryable, Serialize)]
pub struct Artist {
    pub id: Uuid,
    pub name: String,
    pub bio: String,
    pub website_url: Option<String>,
    pub youtube_video_urls: Vec<String>,
    pub facebook_username: Option<String>,
    pub instagram_username: Option<String>,
    pub snapshat_username: Option<String>,
    pub soundcloud_username: Option<String>,
    pub bandcamp_username: Option<String>,
}

#[derive(Insertable, Deserialize, Validate)]
#[table_name = "artists"]
pub struct NewArtist {
    pub name: String,
    pub bio: String,
    #[validate(url)]
    pub website_url: Option<String>,
    #[validate(custom = "validators::validate_urls")]
    pub youtube_video_urls: Vec<String>,
    pub facebook_username: Option<String>,
    pub instagram_username: Option<String>,
    pub snapshat_username: Option<String>,
    pub soundcloud_username: Option<String>,
    pub bandcamp_username: Option<String>,
}

#[derive(AsChangeset, Default, Deserialize, Validate)]
#[table_name = "artists"]
pub struct ArtistEditableAttributes {
    pub name: Option<String>,
    pub bio: Option<String>,
    #[validate(url)]
    pub website_url: Option<String>,
    #[validate(custom = "validators::validate_urls")]
    pub youtube_video_urls: Option<Vec<String>>,
    pub facebook_username: Option<String>,
    pub instagram_username: Option<String>,
    pub snapshat_username: Option<String>,
    pub soundcloud_username: Option<String>,
    pub bandcamp_username: Option<String>,
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
    pub fn create(name: &str, bio: &str, website_url: &str) -> NewArtist {
        NewArtist {
            name: String::from(name),
            bio: String::from(bio),
            website_url: Some(String::from(website_url)),
            youtube_video_urls: Vec::new(),
            facebook_username: None,
            instagram_username: None,
            snapshat_username: None,
            soundcloud_username: None,
            bandcamp_username: None,
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
        attributes: &ArtistEditableAttributes,
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
