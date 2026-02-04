use crate::date::{end_of_day, start_of_day};
use chrono::{DateTime, NaiveDate, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl DateRange {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        Self { start, end }
    }

    pub fn single(date: NaiveDate) -> Self {
        Self {
            start: date,
            end: date,
        }
    }

    pub fn start_utc(&self) -> DateTime<Utc> {
        start_of_day(self.start)
    }

    pub fn end_utc(&self) -> DateTime<Utc> {
        end_of_day(self.end)
    }

    pub fn contains(&self, date: NaiveDate) -> bool {
        date >= self.start && date <= self.end
    }

    pub fn contains_datetime(&self, dt: DateTime<Utc>) -> bool {
        let date = dt.date_naive();
        self.contains(date)
    }

    pub fn days(&self) -> i64 {
        self.end.signed_duration_since(self.start).num_days() + 1
    }
}

pub fn expand_to_range(
    from: Option<String>,
    to: Option<String>,
) -> crate::error::Result<DateRange> {
    use crate::error::CaliError;

    match (from, to) {
        (Some(from_str), Some(to_str)) => {
            let start = parse_iso_date(&from_str)?;
            let end = parse_iso_date(&to_str)?;
            if end < start {
                return Err(CaliError::DateParse {
                    input: "end date is before start date".to_string(),
                });
            }
            Ok(DateRange::new(start, end))
        }
        (Some(from_str), None) => {
            let start = parse_iso_date(&from_str)?;
            Ok(DateRange::single(start))
        }
        (None, Some(to_str)) => {
            let end = parse_iso_date(&to_str)?;
            Ok(DateRange::single(end))
        }
        (None, None) => Ok(DateRange::single(crate::date::today())),
    }
}

fn parse_iso_date(s: &str) -> crate::error::Result<NaiveDate> {
    use crate::error::CaliError;
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
        .map_err(|_| CaliError::DateParse {
            input: s.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_day_range() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let range = DateRange::single(date);
        assert_eq!(range.start, date);
        assert_eq!(range.end, date);
        assert_eq!(range.days(), 1);
    }

    #[test]
    fn test_multi_day_range() {
        let start = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 20).unwrap();
        let range = DateRange::new(start, end);
        assert_eq!(range.days(), 6);
    }

    #[test]
    fn test_contains() {
        let start = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 20).unwrap();
        let range = DateRange::new(start, end);

        assert!(range.contains(start));
        assert!(range.contains(end));
        assert!(range.contains(NaiveDate::from_ymd_opt(2025, 1, 17).unwrap()));
        assert!(!range.contains(NaiveDate::from_ymd_opt(2025, 1, 14).unwrap()));
        assert!(!range.contains(NaiveDate::from_ymd_opt(2025, 1, 21).unwrap()));
    }
}
