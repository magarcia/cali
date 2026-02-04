mod natural;
mod range;

pub use natural::parse_date;
pub use range::{DateRange, expand_to_range};

use chrono::{DateTime, Local, LocalResult, NaiveDate, NaiveDateTime, TimeZone, Utc};

pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

pub fn local_naive_to_utc(naive: NaiveDateTime, prefer_earliest: bool) -> DateTime<Utc> {
    let local_dt = match Local.from_local_datetime(&naive) {
        LocalResult::Single(dt) => dt,
        LocalResult::Ambiguous(earliest, latest) => {
            if prefer_earliest {
                earliest
            } else {
                latest
            }
        }
        LocalResult::None => Local.from_utc_datetime(&naive),
    };

    local_dt.with_timezone(&Utc)
}

pub fn start_of_day(date: NaiveDate) -> DateTime<Utc> {
    let naive = date.and_hms_opt(0, 0, 0).expect("midnight is always valid");
    local_naive_to_utc(naive, true)
}

pub fn end_of_day(date: NaiveDate) -> DateTime<Utc> {
    let naive = date
        .and_hms_opt(23, 59, 59)
        .expect("23:59:59 is always valid");
    local_naive_to_utc(naive, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_today_returns_date() {
        let d = today();
        assert!(d.year() >= 2024);
    }

    #[test]
    fn test_start_end_of_day() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let start = start_of_day(date);
        let end = end_of_day(date);

        assert_eq!(start.with_timezone(&Local).date_naive(), date);
        assert_eq!(end.with_timezone(&Local).date_naive(), date);
        assert!(start < end);
    }
}
