use diesel::result::ConnectionError;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error as DieselError;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::error::Error;
use std::fmt;
use std::fmt::Display;

pub enum ErrorCode {
    InvalidInput,
    MissingInput,
    NoResults,
    QueryError,
    InsertError,
    UpdateError,
    DeleteError,
    DuplicateKeyError,
    ConnectionError,
    InternalError,
    AccessError,
    Unknown,
}

pub fn get_error_message(code: ErrorCode) -> (i32, String) {
    use self::ErrorCode::*;
    let (code, msg) = match code {
        InvalidInput => (1000, "Invalid input"),
        MissingInput => (1100, "Missing input"),
        NoResults => (2000, "No results"),
        QueryError => (3000, "Query Error"),
        InsertError => (3100, "Could not insert record"),
        UpdateError => (3200, "Could not update record"),
        DeleteError => (3300, "Could not delete record"),
        DuplicateKeyError => (3400, "Duplicate key error"),
        ConnectionError => (4000, "Connection Error"),
        InternalError => (5000, "Internal error"),
        AccessError => (6000, "Access error"),
        Unknown => (10, "Unknown database error"),
    };
    (code, msg.to_string())
}

#[derive(Debug, PartialEq)]
pub struct DatabaseError {
    pub code: i32,
    pub message: String,
    pub cause: Option<String>,
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(ref cause) = self.cause {
            write!(f, "\nCaused by: {}", cause)?;
        }
        Ok(())
    }
}

impl Error for DatabaseError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl Serialize for DatabaseError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("DatabaseError", 3)?;
        state.serialize_field("code", &self.code)?;
        state.serialize_field("message", &self.message)?;
        state.serialize_field("cause", &self.cause)?;
        state.end()
    }
}

impl DatabaseError {
    pub fn new(error_code: ErrorCode, cause: Option<&str>) -> DatabaseError {
        let (code, message) = get_error_message(error_code);
        let description = match cause {
            Some(err) => Some(String::from(err)),
            None => None,
        };
        DatabaseError {
            code,
            message,
            cause: description,
        }
    }

    /// Wraps the error from a Result into a DatabaseError
    pub fn wrap<T>(
        error_code: ErrorCode,
        message: &str,
        res: Result<T, DieselError>,
    ) -> Result<T, DatabaseError> {
        match res {
            Ok(val) => Ok(val),
            Err(e) => {
                println!("PG Database error:{}", e.to_string());
                match e {
                    DieselError::DatabaseError(kind, _) => match kind {
                        DatabaseErrorKind::UniqueViolation => Err(DatabaseError::new(
                            ErrorCode::DuplicateKeyError,
                            Some(&format!("{}, {}", message, e.to_string())),
                        )),
                        _ => Err(DatabaseError::new(
                            error_code,
                            Some(&format!("{}, {}", message, e.to_string())),
                        )),
                    },
                    _ => Err(DatabaseError::new(
                        error_code,
                        Some(&format!("{}, {}", message, e.to_string())),
                    )),
                }
            }
        }
    }
}

impl From<ConnectionError> for DatabaseError {
    fn from(e: ConnectionError) -> Self {
        DatabaseError::new(ErrorCode::ConnectionError, Some(&e.to_string()))
    }
}

#[test]
fn error_with_unknown_code() {
    let err = DatabaseError::new(ErrorCode::Unknown, None);
    assert_eq!(err.description(), err.message);
    assert_eq!(err.code, 10);
    assert!(err.cause.is_none());
    assert_eq!(format!("{}", err), "[10] Unknown database error");
}

#[test]
fn error_with_known_code() {
    let err = DatabaseError::new(ErrorCode::InvalidInput, None);
    assert_eq!(err.description(), "Invalid input");
    assert_eq!(err.code, 1000);
    assert!(err.cause.is_none());
    assert_eq!(format!("{}", err), "[1000] Invalid input");
}

#[test]
fn unknown_error_with_cause() {
    let cause = DatabaseError::new(ErrorCode::Unknown, None);
    let err = DatabaseError::new(ErrorCode::InvalidInput, Some(cause.description()));
    assert_eq!(err.description(), "Invalid input");
    assert_eq!(err.code, 1000);
    assert!(err.cause.is_some());
    assert_eq!(
        format!("{}", err),
        "\
[1000] Invalid input
Caused by: Unknown database error"
    );
}

#[test]
fn nested_causes() {
    let cause1 = DatabaseError::new(ErrorCode::Unknown, None);
    let cause2 = DatabaseError::new(ErrorCode::NoResults, Some(&format!("{}", cause1)));
    let err = DatabaseError::new(ErrorCode::InvalidInput, Some(&format!("{}", cause2)));
    assert_eq!(err.code, 1000);
    assert!(err.cause.is_some());
    assert_eq!(
        format!("{}", err),
        "\
[1000] Invalid input
Caused by: [2000] No results
Caused by: [10] Unknown database error"
    );
}
