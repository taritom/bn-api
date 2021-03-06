const LOWER_CONTROL_CHAR_RANGE: u8 = 0x00; // NUL
const UPPER_CONTROL_CHAR_RANGE: u8 = 0x1F; // US
const ADDITIONAL_ESCAPE_SET: &'static [u8] = &[127u8];

/// returns string with control characters replaced with unicode escape sequences
pub fn escape_control_chars(s: &str) -> String {
    with_control_chars(s, &|c: char| c.escape_unicode().to_string())
}

/// returns string with control characters removed
pub fn replace_control_chars(s: &str, replace: String) -> String {
    with_control_chars(s, &|_c: char| replace.clone())
}

pub fn with_control_chars(s: &str, transform: &dyn Fn(char) -> String) -> String {
    let mut buf = String::new();
    for c in s.chars() {
        let b = c as u8;
        if b >= LOWER_CONTROL_CHAR_RANGE && b <= UPPER_CONTROL_CHAR_RANGE
            || find_byte(ADDITIONAL_ESCAPE_SET, b).is_some()
        {
            buf.push_str(&transform(c));
            continue;
        }
        buf.push(c);
    }

    buf
}

fn find_byte(s: &[u8], b: u8) -> Option<usize> {
    for (i, c) in s.iter().enumerate() {
        if *c == b {
            return Some(i);
        }
    }
    None
}

#[test]
fn test_escape_control_chars() {
    let subject = escape_control_chars;

    let example = "A n«rmal(ish) string".to_string();
    assert_eq!(subject(&example), example.as_str(), "no change to string");

    let example = "bad:   ".to_string();
    assert_eq!(subject(&example), "bad: \\u{0} \\u{1f}", "escapes control chars");
}

#[test]
fn test_replace_control_chars() {
    let subject = replace_control_chars;

    let example = "A n«rmal(ish) string".to_string();
    assert_eq!(
        subject(&example, "!!".to_string()),
        example.as_str(),
        "no change to string"
    );

    let example = "bad:   ".to_string();
    assert_eq!(subject(&example, "?".to_string()), "bad: ? ?", "escapes control chars");
}
