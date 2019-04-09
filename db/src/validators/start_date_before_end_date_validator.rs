use chrono::NaiveDateTime;
use std::borrow::Cow;
use validator::ValidationError;
use validators::*;

pub fn start_date_valid(
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
) -> Result<(), ValidationError> {
    if start_date > end_date {
        let mut validation_error = create_validation_error(
            "start_date_must_be_before_end_date",
            "Start date must be before end date",
        );
        validation_error.add_param(Cow::from("start_date"), &start_date);
        validation_error.add_param(Cow::from("end_date"), &end_date);
        return Err(validation_error);
    }
    Ok(())
}
