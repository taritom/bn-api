const LOWER_CONTROL_CHAR_RANGE: u8 = 0x00; // NUL
const UPPER_CONTROL_CHAR_RANGE: u8 = 0x1F; // US

/// returns string with escaped control characters
pub fn escape_control_chars(s: &str) -> String {
    let mut buf = String::new();
    for c in s.chars() {
        let b = c as u8;
        if b >= LOWER_CONTROL_CHAR_RANGE && b <= UPPER_CONTROL_CHAR_RANGE {
            let b = c.escape_unicode();
            buf.push_str(&b.to_string());
            continue;
        }
        buf.push(c);
    }

    buf
}

pub fn remove_control_chars(s: &str) -> String {
    let mut buf = String::new();
    for c in s.chars() {
        let b = c as u8;
        if b >= LOWER_CONTROL_CHAR_RANGE && b <= UPPER_CONTROL_CHAR_RANGE {
            continue;
        }
        buf.push(c);
    }

    buf
}

#[test]
fn test_escape_control_chars() {
    let subject = escape_control_chars;

    let example = "A n«rmal(ish) string".to_string();
    assert_eq!(subject(&example), example.as_str(), "no change to string");

    let example = "bad:   ".to_string();
    assert_eq!(
        subject(&example),
        "bad: \\u{0} \\u{1f}",
        "escapes control chars"
    );
}
