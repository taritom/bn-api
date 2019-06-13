use diesel::prelude::*;
use models::*;
use test::builders::*;
use uuid::Uuid;

pub struct NoteBuilder<'a> {
    created_by: Option<User>,
    main_table: Option<Tables>,
    main_id: Option<Uuid>,
    note: String,
    connection: &'a PgConnection,
}

impl<'a> NoteBuilder<'a> {
    pub fn new(connection: &'a PgConnection) -> NoteBuilder<'a> {
        NoteBuilder {
            connection,
            created_by: None,
            main_table: None,
            main_id: None,
            note: "Will pick up tickets at 16h00".to_string(),
        }
    }

    pub fn created_by(mut self, created_by: &User) -> NoteBuilder<'a> {
        self.created_by = Some(created_by.clone());
        self
    }

    pub fn for_order(mut self, order: &Order) -> NoteBuilder<'a> {
        self.main_table = Some(Tables::Orders);
        self.main_id = Some(order.id);
        self
    }

    pub fn with_note(mut self, note: &String) -> NoteBuilder<'a> {
        self.note = note.clone();
        self
    }

    pub fn finish(mut self) -> Note {
        if self.created_by.is_none() {
            let created_by = UserBuilder::new(self.connection).finish();
            self.created_by = Some(created_by);
        }
        if self.main_table.is_none() || self.main_id.is_none() {
            let order = OrderBuilder::new(self.connection).finish();
            self.main_table = Some(Tables::Orders);
            self.main_id = Some(order.id);
        }
        Note::create(
            self.main_table.unwrap(),
            self.main_id.unwrap(),
            self.created_by.unwrap().id,
            self.note.clone(),
        )
        .commit(self.connection)
        .unwrap()
    }
}
