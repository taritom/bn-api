use chrono::prelude::*;
use chrono::Duration;

pub struct DateBuilder {
    date: NaiveDateTime,
}

pub fn now() -> DateBuilder {
    DateBuilder {
        date: Utc::now().naive_utc(),
    }
}

impl DateBuilder {
    pub fn add_days(self, days: i64) -> DateBuilder {
        DateBuilder {
            date: self.date + Duration::days(days),
        }
    }

    pub fn add_seconds(self, seconds: i64) -> DateBuilder {
        DateBuilder {
            date: self.date + Duration::seconds(seconds),
        }
    }

    pub fn add_hours(self, hours: i64) -> DateBuilder {
        DateBuilder {
            date: self.date + Duration::hours(hours),
        }
    }

    pub fn add_minutes(self, minutes: i64) -> DateBuilder {
        DateBuilder {
            date: self.date + Duration::minutes(minutes),
        }
    }

    pub fn finish(self) -> NaiveDateTime {
        self.date
    }
}

pub trait IntoDateBuilder {
    fn into_builder(self) -> DateBuilder;
}

impl IntoDateBuilder for NaiveDateTime {
    fn into_builder(self) -> DateBuilder {
        DateBuilder { date: self }
    }
}
