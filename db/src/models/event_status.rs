use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EventStatus {
    Draft,
    Closed,
    Published,
    Offline,
}

impl EventStatus {
    pub fn parse(s: &str) -> Result<EventStatus, &'static str> {
        match s {
            "Draft" => Ok(EventStatus::Draft),
            "Closed" => Ok(EventStatus::Closed),
            "Published" => Ok(EventStatus::Published),
            "Offline" => Ok(EventStatus::Offline),
            _ => Err("Could not parse event status. Unexpected value occurred"),
        }
    }
}

impl Display for EventStatus {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            EventStatus::Draft => write!(f, "Draft"),
            EventStatus::Closed => write!(f, "Closed"),
            EventStatus::Published => write!(f, "Published"),
            EventStatus::Offline => write!(f, "Offline"),
        }
    }
}

#[test]
fn display() {
    assert_eq!(EventStatus::Draft.to_string(), "Draft");
    assert_eq!(EventStatus::Closed.to_string(), "Closed");
    assert_eq!(EventStatus::Published.to_string(), "Published");
    assert_eq!(EventStatus::Offline.to_string(), "Offline");
}

#[test]
fn parse() {
    assert_eq!(EventStatus::Draft, EventStatus::parse("Draft").unwrap());
    assert_eq!(EventStatus::Closed, EventStatus::parse("Closed").unwrap());
    assert_eq!(
        EventStatus::Published,
        EventStatus::parse("Published").unwrap()
    );
    assert_eq!(EventStatus::Offline, EventStatus::parse("Offline").unwrap());
    assert!(EventStatus::parse("Not status").is_err());
}
