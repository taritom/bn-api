mod redemption_code_uniqueness_validator;
mod start_date_before_end_date_validator;
mod url_array_validator;

pub use self::redemption_code_uniqueness_validator::redemption_code_unique_per_event_validation;
pub use self::start_date_before_end_date_validator::start_date_valid;
pub use self::url_array_validator::validate_urls;
use validator::*;

pub fn append_validation_error(
    validation_errors: Result<(), ValidationErrors>,
    field: &'static str,
    validation_error: Result<(), ValidationError>,
) -> Result<(), ValidationErrors> {
    if let Err(validation_error) = validation_error {
        let mut validation_errors = match validation_errors {
            Ok(_) => ValidationErrors::new(),
            Err(mut validation_errors) => validation_errors,
        };
        validation_errors.add(field, validation_error);
        Err(validation_errors)
    } else {
        validation_errors
    }
}
