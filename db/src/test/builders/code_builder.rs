use chrono::prelude::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use models::*;
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use rand::{thread_rng, Rng};
use test::builders::*;
use time::Duration;
use uuid::Uuid;

pub struct CodeBuilder<'a> {
    name: String,
    redemption_code: String,
    event_id: Option<Uuid>,
    connection: &'a PgConnection,
    ticket_type_ids: Vec<Uuid>,
    code_type: CodeTypes,
}

impl<'a> CodeBuilder<'a> {
    pub fn new(connection: &'a PgConnection) -> Self {
        let x: u16 = random();
        let redemption_code = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<String>()
            .to_uppercase();

        CodeBuilder {
            name: format!("Code {}", x).into(),
            redemption_code,
            connection,
            ticket_type_ids: Vec::new(),
            event_id: None,
            code_type: CodeTypes::Discount,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_redemption_code(mut self, redemption_code: String) -> Self {
        self.redemption_code = redemption_code;
        self
    }

    pub fn with_code_type(mut self, code_type: CodeTypes) -> Self {
        self.code_type = code_type;
        self
    }

    pub fn for_ticket_type(mut self, ticket_type: &TicketType) -> Self {
        self.ticket_type_ids.push(ticket_type.id);
        self
    }

    pub fn with_event(mut self, event: &Event) -> Self {
        self.event_id = Some(event.id);
        self
    }

    pub fn finish(mut self) -> Code {
        if self.event_id.is_none() {
            self.event_id = Some(
                EventBuilder::new(self.connection)
                    .with_ticket_pricing()
                    .with_tickets()
                    .finish()
                    .id,
            );
        }

        let start_date = NaiveDateTime::from(Utc::now().naive_utc() - Duration::days(1));
        let end_date = NaiveDateTime::from(Utc::now().naive_utc() + Duration::days(2));

        let code = Code::create(
            self.name,
            self.event_id.unwrap(),
            self.code_type,
            self.redemption_code,
            10,
            100,
            start_date,
            end_date,
            None,
        ).commit(self.connection)
        .unwrap();

        for ticket_type_id in self.ticket_type_ids {
            TicketTypeCode::create(ticket_type_id, code.id)
                .commit(self.connection)
                .unwrap();
        }

        code
    }
}
