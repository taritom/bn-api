use actix_web::error;
use actix_web::Error as web_error;
use bigneon_db::utils::errors::DatabaseError;

pub trait ConvertToWebError {
    fn create_http_error(&self) -> web_error;
}

impl ConvertToWebError for DatabaseError {
    fn create_http_error(&self) -> web_error {
        let new_web_error: web_error = match self.code {
            1000 => error::ErrorBadRequest("Invalid input"),
            1100 => error::ErrorBadRequest("Missing input"),
            2000 => error::ErrorNotFound("No results"),
            3000 => error::ErrorInternalServerError("Query error"),
            3100 => error::ErrorInternalServerError("Could not insert record"),
            3200 => error::ErrorInternalServerError("Could not update record"),
            3300 => error::ErrorInternalServerError("Could not delete record"),
            3400 => error::ErrorConflict("Duplicate record exists"),
            4000 => error::ErrorInternalServerError("Connection error"),
            5000 => error::ErrorInternalServerError("Internal error"),
            _ => error::ErrorInternalServerError("Unknown error"),
        };
        new_web_error
    }
}
