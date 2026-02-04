mod fetcher;
mod parser;
mod rrule;
mod tz;

pub use fetcher::fetch_calendars;
pub use parser::parse_ics;
pub use rrule::expand_recurrence;

use crate::date::{end_of_day, start_of_day, today};
use crate::error::{CaliError, Result};
use crate::storage::{Config, Event, EventCache, Paths};
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use std::fs::OpenOptions;

#[derive(Clone, Copy, Debug)]
struct SyncWindow {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

fn build_sync_windows(max_days: i64) -> Vec<SyncWindow> {
    let max_days = if max_days <= 0 { 1 } else { max_days };
    let today = today();

    let today_window = SyncWindow {
        start: start_of_day(today),
        end: end_of_day(today),
    };

    let days_since_monday = today.weekday().num_days_from_monday() as i64;
    let week_start = today - Duration::days(days_since_monday);
    let week_end = week_start + Duration::days(6);
    let week_window = SyncWindow {
        start: start_of_day(week_start),
        end: end_of_day(week_end),
    };

    let month_start =
        NaiveDate::from_ymd_opt(today.year(), today.month(), 1).expect("valid month start");
    let (next_year, next_month) = if today.month() == 12 {
        (today.year() + 1, 1)
    } else {
        (today.year(), today.month() + 1)
    };
    let next_month_start =
        NaiveDate::from_ymd_opt(next_year, next_month, 1).expect("valid next month");
    let month_end = next_month_start - Duration::days(1);
    let month_window = SyncWindow {
        start: start_of_day(month_start),
        end: end_of_day(month_end),
    };

    let full_window = SyncWindow {
        start: start_of_day(today - Duration::days(max_days)),
        end: end_of_day(today + Duration::days(max_days)),
    };

    let candidates = [today_window, week_window, month_window, full_window];
    let mut windows: Vec<SyncWindow> = Vec::new();

    for candidate in candidates {
        if candidate.start < full_window.start || candidate.end > full_window.end {
            continue;
        }
        if let Some(last) = windows.last() {
            if candidate.start >= last.start && candidate.end <= last.end {
                continue;
            }
        }
        windows.push(candidate);
    }

    windows
}

pub async fn perform_sync(config: &Config, paths: &Paths) -> Result<Vec<Event>> {
    let windows = build_sync_windows(config.sync.cache_window_days);
    perform_sync_with_windows(config, paths, &windows).await
}

pub async fn perform_sync_quick(config: &Config, paths: &Paths) -> Result<Vec<Event>> {
    let windows = build_sync_windows(config.sync.cache_window_days);
    if windows.is_empty() {
        return Ok(Vec::new());
    }
    perform_sync_with_windows(config, paths, &windows[..1]).await
}

async fn perform_sync_with_windows(
    config: &Config,
    paths: &Paths,
    windows: &[SyncWindow],
) -> Result<Vec<Event>> {
    let lock_file = paths.lock_file();

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&lock_file)
        .map_err(|e| CaliError::Io {
            message: format!("Failed to open lock file: {}", lock_file.display()),
            source: e.into(),
        })?;

    use fs2::FileExt;
    file.try_lock_exclusive()
        .map_err(|_| CaliError::SyncLocked)?;

    let ics_data = fetch_calendars(config).await?;

    let mut all_events = Vec::new();

    for (name, data) in &ics_data {
        let events = parse_ics(name, data)?;
        all_events.extend(events);
    }

    let cache_loader = crate::storage::EventCacheLoader::new(paths.clone());
    let mut last_expanded = Vec::new();

    for window in windows {
        let expanded = expand_recurrence(&all_events, window.start, window.end)?;
        let cache = EventCache::new(window.start, window.end, expanded.clone());
        cache_loader.write(&cache)?;
        last_expanded = expanded;
    }

    Ok(last_expanded)
}

pub fn setup_ctrlc() {
    let _ = ctrlc::set_handler(|| {
        std::process::exit(130);
    });
}
