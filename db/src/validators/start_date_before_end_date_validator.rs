use chrono::NaiveDateTime;
use validator::ValidationError;

pub fn start_date_valid(
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
) -> Result<(), ValidationError> {
    if start_date >= end_date {
        return Err(ValidationError::new("start_date_must_be_before_end_date"));
    }
    Ok(())
}
