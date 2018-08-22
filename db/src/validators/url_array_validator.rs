use validator::{validate_url, ValidationError};

pub fn validate_urls(urls: &Vec<String>) -> Result<(), ValidationError> {
    for url in urls {
        if !validate_url(url) {
            return Err(ValidationError::new(&"url"));
        }
    }
    Ok(())
}
