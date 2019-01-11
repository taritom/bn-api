use diesel::prelude::*;
use models::*;
use uuid::Uuid;

pub struct StageBuilder<'a> {
    name: String,
    venue_id: Uuid,
    description: Option<String>,
    capacity: Option<i64>,
    connection: &'a PgConnection,
}

impl<'a> StageBuilder<'a> {
    pub fn new(connection: &PgConnection) -> StageBuilder {
        let x: i32 = rand::random::<i32>();

        StageBuilder {
            connection,
            name: format!("Stage {}", x).into(),
            venue_id: Uuid::nil(),
            description: None,
            capacity: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_capacity(mut self, capacity: i64) -> Self {
        self.capacity = Some(capacity);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_venue_id(mut self, venue_id: Uuid) -> Self {
        self.venue_id = venue_id;
        self
    }

    pub fn finish(self) -> Stage {
        Stage::create(self.venue_id, self.name, self.description, self.capacity)
            .commit(self.connection)
            .unwrap()
    }
}
