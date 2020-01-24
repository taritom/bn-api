use diesel::prelude::*;
use models::*;
use rand::prelude::*;
use test::builders::*;
use uuid::Uuid;

pub struct SlugBuilder<'a> {
    slug: String,
    main_table: Tables,
    main_table_id: Option<Uuid>,
    slug_type: SlugTypes,
    title: Option<String>,
    description: Option<String>,
    connection: &'a PgConnection,
}

impl<'a> SlugBuilder<'a> {
    pub fn new(connection: &PgConnection) -> SlugBuilder {
        let x: u32 = random();
        SlugBuilder {
            connection,
            slug_type: SlugTypes::Venue,
            main_table_id: None,
            main_table: Tables::Venues,
            slug: format!("slug-example-{}", x).into(),
            title: None,
            description: None,
        }
    }

    pub fn with_slug(mut self, slug: &str) -> Self {
        self.slug = slug.to_string();
        self
    }

    pub fn with_type(mut self, slug_type: SlugTypes) -> Self {
        self.slug_type = slug_type;
        self
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn for_event(mut self, event: &Event) -> Self {
        self.main_table_id = Some(event.id);
        self.slug_type = SlugTypes::Event;
        self.main_table = Tables::Events;
        self
    }

    pub fn for_organization(mut self, organization: &Organization) -> Self {
        self.main_table_id = Some(organization.id);
        self.slug_type = SlugTypes::Organization;
        self.main_table = Tables::Organizations;
        self
    }

    pub fn for_venue(mut self, venue: &Venue, slug_type: SlugTypes) -> Self {
        self.main_table_id = Some(venue.id);
        self.slug_type = slug_type;
        self.main_table = Tables::Venues;
        self
    }

    pub fn finish(self) -> Slug {
        let main_table_id = self
            .main_table_id
            .unwrap_or_else(|| VenueBuilder::new(self.connection).finish().id);

        Slug::create(
            self.slug,
            self.main_table,
            main_table_id,
            self.slug_type,
            self.title,
            self.description,
        )
        .commit(self.connection)
        .unwrap()
    }
}
