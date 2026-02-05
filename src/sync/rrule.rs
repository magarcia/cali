use crate::error::{CaliError, Result};
use crate::storage::Event;
use crate::sync::tz::resolve_tzid;
use chrono::{DateTime, TimeZone, Utc};
use std::collections::HashMap;
use std::sync::Arc;

pub fn expand_recurrence(
    events: &[Event],
    window_start: DateTime<Utc>,
    window_end: DateTime<Utc>,
) -> Result<Vec<Event>> {
    let mut expanded = Vec::new();
    let mut recurrence_groups: HashMap<String, Vec<Event>> = HashMap::new();

    // Group by UID
    for event in events {
        let key = format!("{}:{}", event.source, event.id);
        recurrence_groups
            .entry(key)
            .or_default()
            .push(event.clone());
    }

    // Expand each group
    for (_key, group) in recurrence_groups {
        // Non-recurring events: add as-is
        if group.len() == 1 && group[0].rrule.is_none() {
            if event_overlaps(&group[0], window_start, window_end) {
                expanded.push(group[0].clone());
            }
            continue;
        }

        // Partition into base event and overrides
        let (base_events, overrides): (Vec<_>, Vec<_>) =
            group.iter().partition(|e| e.recurrence_id.is_none());

        // Recurring events: use first base event as template
        if let Some(base) = base_events.first() {
            if let Some(ref rrule_str) = base.rrule {
                // Build override lookup by recurrence_id timestamp
                let override_map: HashMap<i64, &Event> = overrides
                    .iter()
                    .filter_map(|e| e.recurrence_id.map(|rid| (rid.timestamp(), *e)))
                    .collect();

                match expand_rrule_event(
                    base,
                    rrule_str,
                    window_start,
                    window_end,
                    base.tzid.as_deref(),
                ) {
                    Ok(instances) => {
                        for instance in instances {
                            // Check if this occurrence has an override
                            if let Some(override_event) =
                                override_map.get(&instance.start.timestamp())
                            {
                                // Use the override instead if it's in the window
                                if event_overlaps(override_event, window_start, window_end) {
                                    expanded.push((*override_event).clone());
                                }
                            } else {
                                expanded.push(instance);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to expand RRULE for event '{}': {}",
                            base.title, e
                        );
                        // Add base event as fallback
                        if event_overlaps(base, window_start, window_end) {
                            expanded.push((*base).clone());
                        }
                    }
                }
            } else {
                // No RRULE but multiple instances (e.g., RECURRENCE-ID events only)
                // Add all instances within window
                for event in &group {
                    if event_overlaps(event, window_start, window_end) {
                        expanded.push(event.clone());
                    }
                }
            }
        }
    }

    expanded.sort();
    expanded.dedup_by(|a, b| a.id == b.id && a.start == b.start);
    Ok(expanded)
}

fn event_overlaps(event: &Event, window_start: DateTime<Utc>, window_end: DateTime<Utc>) -> bool {
    event.start <= window_end && event.end >= window_start
}

fn expand_rrule_event(
    base: &Event,
    rrule_str: &str,
    window_start: DateTime<Utc>,
    window_end: DateTime<Utc>,
    tzid: Option<&str>,
) -> Result<Vec<Event>> {
    use rrule::{RRuleSet, Tz};

    let (rrule_set, window_start_tz, window_end_tz) = if let Some(tzid) = tzid {
        if let Some(tz) = resolve_tzid(tzid) {
            let tz_wrapped: Tz = tz.into();
            let start_local = base.start.with_timezone(&tz_wrapped);
            let start_str = start_local.format("%Y%m%dT%H%M%S").to_string();
            let tz_name = tz.name();
            let ics_input = format!("DTSTART;TZID={tz_name}:{start_str}\nRRULE:{rrule_str}");

            let mut rrule_set: RRuleSet =
                ics_input.parse().map_err(|e| CaliError::RruleFailure {
                    name: base.title.to_string(),
                    source: Box::new(e),
                })?;

            for exdate in &base.exdates {
                let exdate_tz = tz_wrapped.from_utc_datetime(&exdate.naive_utc());
                rrule_set = rrule_set.exdate(exdate_tz);
            }

            let window_start_tz = tz_wrapped.from_utc_datetime(&window_start.naive_utc());
            let window_end_tz = tz_wrapped.from_utc_datetime(&window_end.naive_utc());
            (rrule_set, window_start_tz, window_end_tz)
        } else {
            build_utc_rrule_set(base, rrule_str, window_start, window_end)?
        }
    } else {
        build_utc_rrule_set(base, rrule_str, window_start, window_end)?
    };

    // Get all occurrences in window (limit to prevent infinite loops)
    let result = rrule_set
        .after(window_start_tz)
        .before(window_end_tz)
        .all(1000);

    let occurrences = result.dates;

    // Generate Event for each occurrence
    let duration = base.end.signed_duration_since(base.start);
    let mut instances = Vec::new();

    for dt in occurrences {
        // Convert from rrule's DateTime<Tz> back to chrono::DateTime<Utc>
        let event_start = Utc.from_utc_datetime(&dt.naive_utc());
        let event_end = event_start + duration;

        let mut event = base.clone();
        event.start = event_start;
        event.end = event_end;
        // Generate unique ID for this occurrence
        event.id = Arc::from(format!("{}@{}", base.id, event_start.timestamp()));
        instances.push(event);
    }

    Ok(instances)
}

fn build_utc_rrule_set(
    base: &Event,
    rrule_str: &str,
    window_start: DateTime<Utc>,
    window_end: DateTime<Utc>,
) -> Result<(
    rrule::RRuleSet,
    chrono::DateTime<rrule::Tz>,
    chrono::DateTime<rrule::Tz>,
)> {
    use rrule::{RRuleSet, Tz};

    let start_str = base.start.format("%Y%m%dT%H%M%SZ").to_string();
    let ics_input = format!("DTSTART:{start_str}\nRRULE:{rrule_str}");

    let mut rrule_set: RRuleSet = ics_input.parse().map_err(|e| CaliError::RruleFailure {
        name: base.title.to_string(),
        source: Box::new(e),
    })?;

    for exdate in &base.exdates {
        let exdate_tz = Tz::UTC.from_utc_datetime(&exdate.naive_utc());
        rrule_set = rrule_set.exdate(exdate_tz);
    }

    let window_start_tz = Tz::UTC.from_utc_datetime(&window_start.naive_utc());
    let window_end_tz = Tz::UTC.from_utc_datetime(&window_end.naive_utc());

    Ok((rrule_set, window_start_tz, window_end_tz))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_expand_empty() {
        let now = Utc::now();
        let result =
            expand_recurrence(&[], now - Duration::days(1), now + Duration::days(1)).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_expand_single_non_recurring() {
        let event = Event::new(
            "1".to_string(),
            "Test".to_string(),
            Utc::now(),
            Utc::now() + Duration::hours(1),
            "test".to_string(),
        );
        let now = Utc::now();
        let result =
            expand_recurrence(&[event], now - Duration::days(1), now + Duration::days(1)).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_expand_recurring_weekly() {
        let base = Event::new(
            "123".to_string(),
            "Weekly Meeting".to_string(),
            Utc::now(),
            Utc::now() + Duration::hours(1),
            "test".to_string(),
        )
        .with_rrule(Some("FREQ=WEEKLY;BYDAY=MO,WE,FR".to_string()));

        let now = Utc::now();
        let result =
            expand_recurrence(&[base], now - Duration::days(30), now + Duration::days(365))
                .unwrap();
        // Should have multiple occurrences within a year
        assert!(result.len() > 1);
    }
}
