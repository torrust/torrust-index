use std::fmt;
use chrono::{Datelike, Timelike};

pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8
}

impl DateTime {
    pub fn now() -> Self {
        let dt = chrono::offset::Utc::now();

        Self {
            year: dt.year() as u16,
            month: dt.month() as u8,
            day: dt.day() as u8,
            hours: dt.hour() as u8,
            minutes: dt.minute() as u8,
            seconds: dt.second() as u8
        }
    }

    // min 0000 max 9999
    pub fn year(&self) -> String {
        let mut year_string = match self.year {
            10000 ..= u16::MAX => "9999".to_string(),
            year => year.to_string()
        };

        while year_string.len() < 4 {
            year_string = format!("0{}", year_string);
        }

        year_string
    }

    // min 01 max 12
    pub fn month(&self) -> String {
        let mut month_string = match self.month {
            13 ..= u8::MAX => "12".to_string(),
            0 => "01".to_string(),
            month => month.to_string()
        };

        while month_string.len() < 2 {
            month_string = format!("0{}", month_string);
        }

        month_string
    }

    // min 01 max 31
    pub fn day(&self) -> String {
        let mut day_string = match self.day {
            32 ..= u8::MAX => "31".to_string(),
            0 => "01".to_string(),
            day => day.to_string()
        };

        while day_string.len() < 2 {
            day_string = format!("0{}", day_string);
        }

        day_string
    }

    // min 00 max 23
    pub fn hours(&self) -> String {
        let mut hours_string = match self.hours {
            24 ..= u8::MAX => "23".to_string(),
            hours => hours.to_string()
        };

        while hours_string.len() < 2 {
            hours_string = format!("0{}", hours_string);
        }

        hours_string
    }

    // min 00 max 59
    pub fn minutes(&self) -> String {
        let mut minutes_string = match self.minutes {
            60 ..= u8::MAX => "59".to_string(),
            minutes => minutes.to_string()
        };

        while minutes_string.len() < 2 {
            minutes_string = format!("0{}", minutes_string);
        }

        minutes_string
    }

    // min 00 max 59
    pub fn seconds(&self) -> String {
        let mut seconds_string = match self.seconds {
            60 ..= u8::MAX => "59".to_string(),
            seconds => seconds.to_string()
        };

        while seconds_string.len() < 2 {
            seconds_string = format!("0{}", seconds_string);
        }

        seconds_string
    }
}

// display in 0000-00-00 00:00:00 format (ISO 8601)
impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}-{} {}:{}:{}", self.year(), self.month(), self.day(), self.hours(), self.minutes(), self.seconds())
    }
}
