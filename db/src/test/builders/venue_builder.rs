use diesel::prelude::*;
use models::*;
use uuid::Uuid;

pub struct VenueBuilder<'a> {
    name: String,
    region_id: Option<Uuid>,
    organization_id: Option<Uuid>,
    is_private: Option<bool>,
    connection: &'a PgConnection,
}

impl<'a> VenueBuilder<'a> {
    pub fn new(connection: &PgConnection) -> VenueBuilder {
        VenueBuilder {
            connection,
            name: "Name".into(),
            region_id: None,
            is_private: None,
            organization_id: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_region(mut self, region: &Region) -> Self {
        self.region_id = Some(region.id.clone());
        self
    }

    pub fn make_private(mut self) -> Self {
        self.is_private = Some(true);
        self
    }

    pub fn finish(self) -> Venue {
        Venue::create(
            &self.name,
            self.region_id,
            self.organization_id,
            self.is_private,
        ).commit(self.connection)
        .unwrap()
    }
}
