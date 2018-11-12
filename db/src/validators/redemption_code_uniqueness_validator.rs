use diesel::dsl::select;
use diesel::prelude::*;
use diesel::sql_types::{Text, Uuid as dUuid};
use std::borrow::Cow;
use utils::errors::*;
use uuid::Uuid;
use validator::*;
use validators::*;

sql_function!(fn redemption_code_unique_per_event(id: dUuid, table: Text, redemption_code: Text) -> Bool);

pub fn redemption_code_unique_per_event_validation(
    id: Option<Uuid>,
    table: String,
    redemption_code: String,
    conn: &PgConnection,
) -> Result<Result<(), ValidationError>, DatabaseError> {
    let result = select(redemption_code_unique_per_event(
        id.unwrap_or(Uuid::default()),
        table,
        redemption_code.clone(),
    )).get_result::<bool>(conn)
    .to_db_error(
        if id.is_none() {
            ErrorCode::InsertError
        } else {
            ErrorCode::UpdateError
        },
        "Could not confirm if redemption code unique",
    )?;
    if !result {
        let mut validation_error =
            create_validation_error("uniqueness", "Redemption code must be unique");
        validation_error.add_param(Cow::from("id"), &id);
        validation_error.add_param(Cow::from("redemption_code"), &redemption_code);

        return Ok(Err(validation_error));
    }
    Ok(Ok(()))
}
