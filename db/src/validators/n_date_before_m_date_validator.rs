use chrono::NaiveDateTime;
use std::borrow::Cow;
use validator::ValidationError;
use validators::*;

pub fn n_date_valid(
    before_date: Option<NaiveDateTime>,
    after_date: Option<NaiveDateTime>,
    validation_code: &'static str,
    validation_message: &'static str,
    before_field_key: &'static str,
    after_field_key: &'static str,
) -> Result<(), ValidationError> {
    if before_date.is_none() || after_date.is_none() {
        return Ok(());
    }
    if before_date >= after_date {
        let mut validation_error = create_validation_error(validation_code, validation_message);
        validation_error.add_param(Cow::from(before_field_key), &before_date);
        validation_error.add_param(Cow::from(after_field_key), &after_date);
        return Err(validation_error);
    }
    Ok(())
}
