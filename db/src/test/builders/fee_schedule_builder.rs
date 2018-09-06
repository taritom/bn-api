use diesel::prelude::*;
use models::*;

pub struct FeeScheduleBuilder<'a> {
    name: String,
    connection: &'a PgConnection,
}

impl<'a> FeeScheduleBuilder<'a> {
    pub fn new(connection: &PgConnection) -> FeeScheduleBuilder {
        FeeScheduleBuilder {
            connection,
            name: "Name".into(),
        }
    }

    pub fn finish(self) -> FeeSchedule {
        FeeSchedule::create(self.name, vec![(0, 200), (10_000, 100)])
            .commit(self.connection)
            .unwrap()
    }
}
