use std::borrow::Cow;
use validator::ValidationError;
use validators::create_validation_error;

pub fn validate_greater_than<T: std::cmp::Ord + serde::Serialize>(
    a: T,
    b: T,
    code: &'static str,
    msg: &'static str,
) -> Result<(), ValidationError> {
    use std::cmp::Ordering::*;

    match a.cmp(&b) {
        Less | Equal => {
            let mut validation_error = create_validation_error(code, msg);
            validation_error.add_param(Cow::from(code), &a);
            Err(validation_error)
        }
        _ => Ok(()),
    }
}

pub fn validate_greater_than_or_equal<T: std::cmp::Ord + serde::Serialize>(
    a: T,
    b: T,
    code: &'static str,
    msg: &'static str,
) -> Result<(), ValidationError> {
    use std::cmp::Ordering::*;

    match a.cmp(&b) {
        Less => {
            let mut validation_error = create_validation_error(code, msg);
            validation_error.add_param(Cow::from(code), &a);
            Err(validation_error)
        }
        _ => Ok(()),
    }
}

pub fn validate_less_than<T: std::cmp::Ord + serde::Serialize>(
    a: T,
    b: T,
    code: &'static str,
    msg: &'static str,
) -> Result<(), ValidationError> {
    use std::cmp::Ordering::*;

    match a.cmp(&b) {
        Greater | Equal => {
            let mut validation_error = create_validation_error(code, msg);
            validation_error.add_param(Cow::from(code), &a);
            Err(validation_error)
        }
        _ => Ok(()),
    }
}

pub fn validate_less_than_or_equal<T: std::cmp::Ord + serde::Serialize>(
    a: T,
    b: T,
    code: &'static str,
    msg: &'static str,
) -> Result<(), ValidationError> {
    use std::cmp::Ordering::*;

    match a.cmp(&b) {
        Greater => {
            let mut validation_error = create_validation_error(code, msg);
            validation_error.add_param(Cow::from(code), &a);
            Err(validation_error)
        }
        _ => Ok(()),
    }
}

#[test]
fn validate_greater_than_returns_ok() {
    assert_eq!(validate_greater_than(3, 2, "test_example", "test example"), Ok(()),);
}

#[test]
fn validate_greater_than_returns_err() {
    let result = validate_greater_than(-1 as i32, 0 as i32, "test_example", "test example");
    match result {
        Ok(_) => panic!("Unexpected Ok result"),
        Err(e) => {
            assert_eq!(e.code, "test_example");
        }
    }
}
