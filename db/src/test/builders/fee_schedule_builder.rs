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
        FeeSchedule::create(
            self.name,
            vec![
                NewFeeScheduleRange {
                    min_price: 50,
                    company_fee_in_cents: 4,
                    client_fee_in_cents: 6,
                },
                NewFeeScheduleRange {
                    min_price: 100,
                    company_fee_in_cents: 8,
                    client_fee_in_cents: 12,
                },
            ],
        )
        .commit(self.connection)
        .unwrap()
    }
}
