use bigneon_db::models::Artist;
use support::project::TestProject;

use rand::prelude::*;

pub struct ArtistBuilder<'a> {
    name: String,
    bio: String,
    website_url: String,
    test_project: &'a TestProject,
}

impl<'a> ArtistBuilder<'a> {
    pub fn new(test_project: &'a TestProject) -> Self {
        let x: u8 = random();

        ArtistBuilder {
            name: format!("Artist {}", x).into(),
            bio: "Bigraphy".into(),
            website_url: "http://www.example.com".into(),
            test_project,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn finish(&self) -> Artist {
        Artist::create(&self.name, &self.bio, &self.website_url)
            .commit(self.test_project)
            .unwrap()
    }
}
