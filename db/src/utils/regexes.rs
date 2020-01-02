use regex::Regex;

pub fn whitespace() -> Regex {
    Regex::new(r#"\s+"#).unwrap()
}

pub fn non_ascii() -> Regex {
    Regex::new(r#"[^a-zA-Z0-9]+"#).unwrap()
}
