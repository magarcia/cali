use crate::date::DateRange;
use crate::date::today;
use crate::error::{CaliError, Result};
use chrono::{Datelike, Duration, Local, NaiveDate};
use chrono_english::Dialect;
use chrono_english::parse_date_string;
use std::ops::{Add, Sub};

pub fn parse_date(input: &str) -> Result<DateRange> {
    let input_lower = input.to_lowercase();

    match input_lower.as_str() {
        "today" => Ok(DateRange::single(today())),
        "tomorrow" => {
            let tomorrow = today().add(Duration::days(1));
            Ok(DateRange::single(tomorrow))
        }
        "yesterday" => {
            let yesterday = today().sub(Duration::days(1));
            Ok(DateRange::single(yesterday))
        }
        "week" | "this week" => week_range(0),
        "next week" => week_range(1),
        "last week" => week_range(-1),
        "weekend" | "this weekend" => weekend_range(0),
        "next weekend" => weekend_range(1),
        "month" | "this month" => month_range(0),
        "next month" => month_range(1),
        _ => parse_english(input),
    }
}

fn parse_english(input: &str) -> Result<DateRange> {
    let now = Local::now();

    let parsed = parse_date_string(input, now, Dialect::Us).map_err(|_| {
        let suggestion = suggest_date(input);
        CaliError::DateParse {
            input: match suggestion {
                Some(s) => format!("{input}' (did you mean '{s}'?)"),
                None => input.to_string(),
            },
        }
    })?;

    Ok(DateRange::single(parsed.date_naive()))
}

fn suggest_date(input: &str) -> Option<&'static str> {
    const KNOWN: &[&str] = &[
        "today",
        "tomorrow",
        "yesterday",
        "week",
        "weekend",
        "month",
        "this week",
        "next week",
        "last week",
        "this weekend",
        "next weekend",
        "this month",
        "next month",
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "friday",
        "saturday",
        "sunday",
    ];

    let input_lower = input.to_lowercase();
    KNOWN
        .iter()
        .filter(|&&k| {
            let dist = edit_distance(&input_lower, k);
            dist > 0 && dist <= 2
        })
        .min_by_key(|&&k| edit_distance(&input_lower, k))
        .copied()
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut dp = vec![vec![0usize; b.len() + 1]; a.len() + 1];

    for (i, row) in dp.iter_mut().enumerate() {
        row[0] = i;
    }
    for j in 0..=b.len() {
        dp[0][j] = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[a.len()][b.len()]
}

fn week_range(weeks_offset: i64) -> Result<DateRange> {
    let today = today();
    let days_since_monday = today.weekday().num_days_from_monday();

    let week_start = today
        .sub(Duration::days(days_since_monday as i64))
        .add(Duration::weeks(weeks_offset));

    let week_end = week_start.add(Duration::days(6));

    Ok(DateRange::new(week_start, week_end))
}

fn weekend_range(weeks_offset: i64) -> Result<DateRange> {
    use chrono::Weekday;

    let today = today();
    let days_until_saturday = (Weekday::Sat.num_days_from_monday() as i64)
        .sub(today.weekday().num_days_from_monday() as i64);

    let saturday = if days_until_saturday < 0 {
        today.add(Duration::days(7 + days_until_saturday))
    } else {
        today.add(Duration::days(days_until_saturday))
    }
    .add(Duration::weeks(weeks_offset));

    let sunday = saturday.add(Duration::days(1));

    Ok(DateRange::new(saturday, sunday))
}

fn month_range(months_offset: i64) -> Result<DateRange> {
    let today = today();
    let year = today.year();
    let month = (today.month() as i32 + months_offset as i32 - 1) % 12 + 1;
    let year_adjust = (today.month() as i32 + months_offset as i32 - 1) / 12;

    let year = year + year_adjust;

    let first_day =
        NaiveDate::from_ymd_opt(year, month as u32, 1).ok_or_else(|| CaliError::DateParse {
            input: format!("month {months_offset}"),
        })?;

    let last_day = NaiveDate::from_ymd_opt(year, month as u32 + 1, 1)
        .map(|d| d.sub(Duration::days(1)))
        .or_else(|| NaiveDate::from_ymd_opt(year, 12, 31))
        .ok_or_else(|| CaliError::DateParse {
            input: format!("month {months_offset}"),
        })?;

    Ok(DateRange::new(first_day, last_day))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_today() {
        let range = parse_date("today").unwrap();
        assert_eq!(range.start, range.end);
    }

    #[test]
    fn test_parse_tomorrow() {
        let range = parse_date("tomorrow").unwrap();
        assert_eq!(range.start, range.end);
        assert_eq!(range.start, today().add(Duration::days(1)));
    }

    #[test]
    fn test_parse_week() {
        let range = parse_date("week").unwrap();
        let duration = range.end.signed_duration_since(range.start);
        assert_eq!(duration.num_days(), 6);
    }
}
