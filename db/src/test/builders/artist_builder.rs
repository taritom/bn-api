use diesel::prelude::*;
use models::Artist;
use rand::prelude::*;

pub struct ArtistBuilder<'a> {
    name: String,
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
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn finish(&self) -> Artist {
        Artist::create(&self.name, &self.bio, &self.website_url)
            .commit(self.connection)
            .unwrap()
    }
}
