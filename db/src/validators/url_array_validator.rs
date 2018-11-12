use std::borrow::Cow;
use validator::{validate_url, ValidationError};
use validators::*;

pub fn validate_urls(urls: &Vec<String>) -> Result<(), ValidationError> {
    for url in urls {
        if !validate_url(url) {
            let mut validation_error = create_validation_error("url", "URL is invalid");
            validation_error.add_param(Cow::from("url"), &url);
            return Err(validation_error);
        }
    }
    Ok(())
}
