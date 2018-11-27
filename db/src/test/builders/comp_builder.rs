use diesel::prelude::*;
use models::*;
use rand::prelude::*;
use test::builders::*;
use uuid::Uuid;

pub struct CompBuilder<'a> {
    name: String,
    hold_id: Option<Uuid>,
    quantity: u32,
    connection: &'a PgConnection,
}

impl<'a> CompBuilder<'a> {
    pub fn new(connection: &'a PgConnection) -> Self {
        let x: u16 = random();
        CompBuilder {
            name: format!("Comp {}", x).into(),
            connection,
            quantity: 3,
            hold_id: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_hold(mut self, hold: &Hold) -> Self {
        self.hold_id = Some(hold.id);
        self
    }

    pub fn with_quantity(mut self, quantity: u32) -> Self {
        self.quantity = quantity;
        self
    }

    pub fn finish(mut self) -> Hold {
        if self.hold_id.is_none() {
            self.hold_id = Some(
                HoldBuilder::new(self.connection)
                    .with_hold_type(HoldTypes::Comp)
                    .finish()
                    .id,
            );
        }
        let x: u16 = random();

        Hold::create_comp_for_person(
            self.name,
            self.hold_id.unwrap(),
            Some("email@test.com".to_string()),
            None,
            format!("REDEEEEEMTPION{}", x),
            None,
            None,
            self.quantity,
            self.connection,
        ).unwrap()
    }
}
