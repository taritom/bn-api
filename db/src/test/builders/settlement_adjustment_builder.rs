use diesel::prelude::*;
use prelude::*;
use test::builders::*;
use uuid::Uuid;

pub struct SettlementAdjustmentBuilder<'a> {
    settlement_id: Option<Uuid>,
    amount_in_cents: i64,
    note: Option<String>,
    connection: &'a PgConnection,
}

impl<'a> SettlementAdjustmentBuilder<'a> {
    pub fn new(connection: &PgConnection) -> SettlementAdjustmentBuilder {
        SettlementAdjustmentBuilder {
            settlement_id: None,
            note: None,
            amount_in_cents: 100,
            connection,
        }
    }

    pub fn with_amount_in_cents(mut self, amount_in_cents: i64) -> Self {
        self.amount_in_cents = amount_in_cents;
        self
    }

    pub fn with_note(mut self, note: Option<String>) -> Self {
        self.note = note;
        self
    }

    pub fn with_settlement(mut self, settlement: &Settlement) -> Self {
        self.settlement_id = Some(settlement.id);
        self
    }

    pub fn finish(&mut self) -> SettlementAdjustment {
        let settlement_id = self
            .settlement_id
            .or_else(|| Some(SettlementBuilder::new(self.connection).finish().id))
            .unwrap();

        SettlementAdjustment::create(
            settlement_id,
            SettlementAdjustmentTypes::ManualDeduction,
            self.note.clone(),
            self.amount_in_cents,
        )
        .commit(self.connection)
        .unwrap()
    }
}
