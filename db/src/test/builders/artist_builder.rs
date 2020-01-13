use diesel::prelude::*;
use models::*;
use rand::prelude::*;
use uuid::Uuid;

pub struct ArtistBuilder<'a> {
    name: String,
    organization_id: Option<Uuid>,
    is_private: bool,
    bio: String,
    website_url: String,
    spotify_id: Option<String>,
    genres: Option<Vec<String>>,
    connection: &'a PgConnection,
}

impl<'a> ArtistBuilder<'a> {
    pub fn new(connection: &'a PgConnection) -> Self {
        let x: u32 = random();
        ArtistBuilder {
            name: format!("Artist {}", x).into(),
            bio: "Bigraphy".into(),
            website_url: "http://www.example.com".into(),
            connection,
            spotify_id: None,
            is_private: false,
            organization_id: None,
            genres: None,
        }
    }

    pub fn with_spotify_id(mut self, spotify_id: String) -> Self {
        self.spotify_id = Some(spotify_id);
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_organization(mut self, organization: &Organization) -> Self {
        self.organization_id = Some(organization.id.clone());
        self
    }

    pub fn with_genres(mut self, genres: Vec<String>) -> Self {
        self.genres = Some(genres);
        self
    }

    pub fn make_private(mut self) -> Self {
        self.is_private = true;
        self
    }

    pub fn finish(&self) -> Artist {
        let mut artist = Artist::create(&self.name, self.organization_id, &self.bio, &self.website_url);
        artist.spotify_id = self.spotify_id.clone();

        let artist = artist.commit(self.connection).unwrap();
        let artist = artist.set_privacy(self.is_private, self.connection).unwrap();
        match &self.genres {
            Some(genres) => {
                artist.set_genres(genres, None, self.connection).unwrap();
                Artist::find(&artist.id, self.connection).unwrap()
            }
            None => artist,
        }
    }
}
