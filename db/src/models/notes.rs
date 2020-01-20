use chrono::NaiveDateTime;
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use models::*;
use schema::notes;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use utils::pagination::*;
use uuid::Uuid;
use validator::Validate;

#[derive(Associations, Clone, Debug, Deserialize, Identifiable, PartialEq, Queryable, Serialize)]
pub struct Note {
    pub id: Uuid,
    pub note: String,
    pub main_table: Tables,
    pub main_id: Uuid,
    pub deleted_by: Option<Uuid>,
    pub deleted_at: Option<NaiveDateTime>,
    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize, Validate)]
#[table_name = "notes"]
pub struct NewNote {
    pub main_table: Tables,
    pub main_id: Uuid,
    #[validate(length(min = "1", message = "Note cannot be blank"))]
    pub note: String,
    pub created_by: Uuid,
}

impl NewNote {
    pub fn commit(&self, conn: &PgConnection) -> Result<Note, DatabaseError> {
        self.validate()?;

        let note: Note = diesel::insert_into(notes::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could insert new note")?;

        DomainEvent::create(
            DomainEventTypes::NoteCreated,
            "Note created".to_string(),
            note.main_table,
            Some(note.main_id),
            Some(note.created_by),
            Some(json!({ "note": note.note, "note_id": note.id })),
        )
        .commit(conn)?;

        Ok(note)
    }
}

impl Note {
    pub fn create(main_table: Tables, main_id: Uuid, created_by: Uuid, note: String) -> NewNote {
        NewNote {
            main_table,
            main_id,
            created_by,
            note,
        }
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<Note, DatabaseError> {
        notes::table
            .find(id)
            .first(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading note")
    }

    pub fn find_for_table(
        main_table: Tables,
        main_id: Uuid,
        filter_deleted: bool,
        page: u32,
        limit: u32,
        conn: &PgConnection,
    ) -> Result<Payload<Note>, DatabaseError> {
        let mut query = notes::table
            .filter(notes::main_table.eq(main_table))
            .filter(notes::main_id.eq(main_id))
            .into_boxed();

        if filter_deleted {
            query = query.filter(notes::deleted_at.is_null());
        }

        let (notes, record_count): (Vec<Note>, i64) = query
            .order_by(notes::created_at.desc())
            .paginate(page as i64)
            .per_page(limit as i64)
            .load_and_count_pages(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading notes")?;

        let payload = Payload::from_data(notes, page, limit, Some(record_count as u64));
        Ok(payload)
    }

    pub fn destroy(&self, user_id: Uuid, conn: &PgConnection) -> Result<usize, DatabaseError> {
        let result = diesel::update(&*self)
            .set((
                notes::deleted_by.eq(user_id),
                notes::updated_at.eq(dsl::now),
                notes::deleted_at.eq(dsl::now),
            ))
            .execute(conn)
            .to_db_error(
                ErrorCode::UpdateError,
                "Could not change the external payment type for this order",
            )?;

        DomainEvent::create(
            DomainEventTypes::NoteDeleted,
            "Note soft deleted".to_string(),
            self.main_table,
            Some(self.main_id),
            Some(self.created_by),
            Some(json!({ "note": self.note, "note_id": self.id })),
        )
        .commit(conn)?;

        Ok(result)
    }
}
