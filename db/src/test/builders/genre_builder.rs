use diesel::prelude::*;
use models::*;

pub struct GenreBuilder<'a> {
    name: String,
    connection: &'a PgConnection,
}

impl<'a> GenreBuilder<'a> {
    pub fn new(connection: &'a PgConnection) -> GenreBuilder<'a> {
        GenreBuilder {
            connection,
            name: "nu-metal".to_string(),
        }
    }

    pub fn with_name(mut self, name: &String) -> GenreBuilder<'a> {
        self.name = name.clone();
        self
    }

    pub fn finish(self) -> Genre {
        let ids = Genre::find_or_create(&vec![self.name], self.connection).unwrap();
        Genre::find(ids[0], self.connection).unwrap()
    }
}
