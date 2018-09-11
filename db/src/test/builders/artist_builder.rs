use diesel::prelude::*;
use models::Artist;
use rand::prelude::*;
use uuid::Uuid;

pub struct ArtistBuilder<'a> {
    name: String,
    organization_id: Option<Uuid>,
    is_private: Option<bool>,
    bio: String,
    website_url: String,
    connection: &'a PgConnection,
}

impl<'a> ArtistBuilder<'a> {
    pub fn new(connection: &'a PgConnection) -> Self {
        let x: u16 = random();
        ArtistBuilder {
            name: format!("Artist {}", x).into(),
            bio: "Bigraphy".into(),
            website_url: "http://www.example.com".into(),
            connection,
            is_private: None,
            organization_id: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_organization(mut self, organization_id: Uuid) -> Self {
        self.organization_id = Some(organization_id);
        self
    }

    pub fn make_private(mut self) -> Self {
        self.is_private = Some(true);
        self
    }

    pub fn finish(&self) -> Artist {
        Artist::create(
            &self.name,
            self.organization_id,
            self.is_private,
            &self.bio,
            &self.website_url,
        ).commit(self.connection)
            .unwrap()
    }
}
