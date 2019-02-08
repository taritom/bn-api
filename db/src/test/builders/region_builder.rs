use diesel::prelude::*;
use models::*;
use rand::prelude::*;

pub struct RegionBuilder<'a> {
    name: String,
    connection: &'a PgConnection,
}

impl<'a> RegionBuilder<'a> {
    pub fn new(connection: &PgConnection) -> RegionBuilder {
        let x: u32 = random();
        RegionBuilder {
            connection,
            name: format!("Region {}", x).into(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn finish(self) -> Region {
        Region::create(self.name).commit(self.connection).unwrap()
    }
}
