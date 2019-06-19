use regex::Regex;

pub fn whitespace() -> Regex {
    Regex::new(r#"\s+"#).unwrap()
}
