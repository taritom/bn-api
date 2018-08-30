use db::Connectable;
use models::*;
use rand::prelude::*;

pub struct RegionBuilder<'a> {
    name: String,
    connection: &'a Connectable,
}

impl<'a> RegionBuilder<'a> {
    pub fn new(connection: &Connectable) -> RegionBuilder {
        let x: u16 = random();
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
