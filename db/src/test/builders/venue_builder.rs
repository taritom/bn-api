use db::Connectable;
use models::*;

pub struct VenueBuilder<'a> {
    connection: &'a Connectable,
}

impl<'a> VenueBuilder<'a> {
    pub fn new(connection: &Connectable) -> VenueBuilder {
        VenueBuilder { connection }
    }

    pub fn finish(self) -> Venue {
        Venue::create("Name").commit(self.connection).unwrap()
    }
}
