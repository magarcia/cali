use crate::date::{local_naive_to_utc, start_of_day};
use crate::error::{CaliError, Result};
use crate::storage::Event;
use crate::sync::tz::resolve_tzid;
use chrono::{DateTime, LocalResult, NaiveDateTime, TimeZone, Utc};
use ical::parser::ical::component::{IcalCalendar, IcalEvent};
use ical::property::Property;

const ICS_DATE_ONLY_LENGTH: usize = 8;

pub fn parse_ics(name: &str, data: &str) -> Result<Vec<Event>> {
    let mut events = Vec::new();
    let mut reader = data.as_bytes();

    let parser = ical::IcalParser::new(&mut reader);

    for calendar in parser {
        let calendar = calendar.map_err(|e| CaliError::ParseFailure {
            name: name.to_string(),
            source: Box::new(e),
        })?;

        let default_tzid = extract_calendar_tzid(&calendar);

        for event in calendar.events {
            if let Ok(evt) = parse_event(name, &event, default_tzid.as_deref()) {
                events.push(evt);
            }
        }
    }

    Ok(events)
}

fn parse_event(source: &str, event: &IcalEvent, default_tzid: Option<&str>) -> Result<Event> {
    let mut id = String::new();
    let mut title = "(No title)".to_string();
    let mut start: Option<DateTime<Utc>> = None;
    let mut end: Option<DateTime<Utc>> = None;
    let mut location = None;
    let mut description = None;
    let mut all_day = false;
    let mut rrule = None;
    let mut tzid = None;
    let mut exdates = Vec::new();
    let mut recurrence_id = None;

    for prop in &event.properties {
        match prop.name.as_str() {
            "UID" => {
                id = prop.value.clone().unwrap_or_default();
            }
            "SUMMARY" => {
                title = prop
                    .value
                    .clone()
                    .unwrap_or_else(|| "(No title)".to_string());
            }
            "DTSTART" => {
                tzid = extract_tzid(prop).or_else(|| default_tzid.map(|v| v.to_string()));
                start = Some(parse_date_time(prop, tzid.as_deref().or(default_tzid))?);
                all_day = is_date_only(prop);
            }
            "DTEND" => {
                let end_tzid = extract_tzid(prop)
                    .or_else(|| tzid.clone())
                    .or_else(|| default_tzid.map(|v| v.to_string()));
                end = Some(parse_date_time(prop, end_tzid.as_deref().or(default_tzid))?);
            }
            "LOCATION" => {
                location = prop.value.clone();
            }
            "DESCRIPTION" => {
                description = prop.value.clone();
            }
            "RRULE" => {
                rrule = prop.value.clone();
            }
            "RDATE" => {
                // Store for future use
            }
            "EXDATE" => {
                if let Some(ref value) = prop.value {
                    let exdate_tzid = extract_tzid(prop).or_else(|| tzid.clone());
                    for date_str in value.split(',') {
                        if let Ok(dt) = parse_exdate_value(
                            date_str.trim(),
                            exdate_tzid.as_deref().or(default_tzid),
                        ) {
                            exdates.push(dt);
                        }
                    }
                }
            }
            "RECURRENCE-ID" => {
                recurrence_id = parse_date_time(
                    prop,
                    extract_tzid(prop)
                        .as_deref()
                        .or(tzid.as_deref())
                        .or(default_tzid),
                )
                .ok();
            }
            _ => {}
        }
    }

    let start = start.ok_or_else(|| CaliError::ParseFailure {
        name: source.to_string(),
        source: "Missing DTSTART".into(),
    })?;

    let end = end.unwrap_or(start);

    Ok(Event::new(id, title, start, end, source.to_string())
        .with_location(location)
        .with_description(description)
        .with_all_day(all_day)
        .with_rrule(rrule)
        .with_tzid(tzid)
        .with_exdates(exdates)
        .with_recurrence_id(recurrence_id))
}

fn extract_calendar_tzid(calendar: &IcalCalendar) -> Option<String> {
    if let Some(prop) = calendar
        .properties
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case("X-WR-TIMEZONE"))
    {
        if let Some(value) = prop.value.as_ref() {
            return Some(value.to_string());
        }
    }

    if let Some(prop) = calendar
        .properties
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case("TZID"))
    {
        if let Some(value) = prop.value.as_ref() {
            return Some(value.to_string());
        }
    }

    for tz in &calendar.timezones {
        if let Some(prop) = tz
            .properties
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case("TZID"))
        {
            if let Some(value) = prop.value.as_ref() {
                return Some(value.to_string());
            }
        }
    }

    None
}

fn parse_date_time(prop: &Property, default_tzid: Option<&str>) -> Result<DateTime<Utc>> {
    let value = prop.value.as_ref().ok_or_else(|| CaliError::ParseFailure {
        name: "event".to_string(),
        source: "Missing date value".into(),
    })?;

    if value.len() == ICS_DATE_ONLY_LENGTH {
        parse_date_only(value)
    } else if value.ends_with('Z') {
        parse_utc(value)
    } else if let Some(tzid) = extract_tzid(prop).or_else(|| default_tzid.map(|v| v.to_string())) {
        parse_with_tz(value, &tzid).or_else(|_| parse_local(value))
    } else {
        parse_local(value)
    }
}

fn parse_utc(value: &str) -> Result<DateTime<Utc>> {
    // Strip the trailing 'Z' and parse
    let without_z = value.strip_suffix('Z').unwrap_or(value);
    parse_naive_datetime(without_z)
        .map(|dt| Utc.from_utc_datetime(&dt))
        .map_err(|e| CaliError::ParseFailure {
            name: "date".to_string(),
            source: Box::new(e),
        })
}

fn parse_date_only(value: &str) -> Result<DateTime<Utc>> {
    let date = chrono::NaiveDate::parse_from_str(value, "%Y%m%d").map_err(|e| {
        CaliError::ParseFailure {
            name: "date".to_string(),
            source: Box::new(e),
        }
    })?;

    Ok(start_of_day(date))
}

fn parse_local(value: &str) -> Result<DateTime<Utc>> {
    parse_naive_datetime(value)
        .map(|dt| local_naive_to_utc(dt, true))
        .map_err(|e| CaliError::ParseFailure {
            name: "date".to_string(),
            source: Box::new(e),
        })
}

fn parse_with_tz(value: &str, tzid: &str) -> Result<DateTime<Utc>> {
    let tz = resolve_tzid(tzid).ok_or_else(|| CaliError::ParseFailure {
        name: tzid.to_string(),
        source: std::io::Error::new(std::io::ErrorKind::InvalidInput, "Unknown TZID").into(),
    })?;

    let naive = parse_naive_datetime(value).map_err(|e| CaliError::ParseFailure {
        name: "date".to_string(),
        source: Box::new(e),
    })?;

    let local = match tz.from_local_datetime(&naive) {
        LocalResult::Single(dt) => dt,
        LocalResult::Ambiguous(earliest, _) => earliest,
        LocalResult::None => tz.from_utc_datetime(&naive),
    };

    Ok(local.with_timezone(&Utc))
}

fn parse_naive_datetime(value: &str) -> std::result::Result<NaiveDateTime, chrono::ParseError> {
    NaiveDateTime::parse_from_str(value, "%Y%m%dT%H%M%S")
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y%m%d%H%M%S"))
}

fn is_date_only(prop: &Property) -> bool {
    prop.value
        .as_ref()
        .is_some_and(|v| v.len() == ICS_DATE_ONLY_LENGTH)
}

fn extract_tzid(prop: &Property) -> Option<String> {
    prop.params
        .as_ref()
        .and_then(|params| params.iter().find(|(k, _)| k == "TZID"))
        .and_then(|(_, v)| v.first())
        .cloned()
}

fn parse_exdate_value(value: &str, tzid: Option<&str>) -> Result<DateTime<Utc>> {
    if value.len() == ICS_DATE_ONLY_LENGTH {
        parse_date_only(value)
    } else if value.ends_with('Z') {
        parse_utc(value)
    } else if let Some(tzid) = tzid {
        parse_with_tz(value, tzid).or_else(|_| parse_local(value))
    } else {
        parse_local(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{LocalResult, NaiveDateTime, TimeZone};

    #[test]
    fn test_parse_utc() {
        let result = parse_utc("20250115T140000Z");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_utc_without_seconds() {
        let result = parse_utc("20250115T140000Z");
        assert!(result.is_ok());
        // Also test without time separator
        let result2 = parse_utc("20250115140000Z");
        assert!(result2.is_ok());
    }

    #[test]
    fn test_parse_utc_invalid() {
        let result = parse_utc("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_date_only() {
        let result = parse_date_only("20250115");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_date_only_invalid() {
        let result = parse_date_only("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_local() {
        let result = parse_local("20250115T140000");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_local_invalid() {
        let result = parse_local("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_date_only_with_exact_length() {
        let prop = Property {
            name: "DTSTART".to_string(),
            value: Some("20250115".to_string()),
            params: None,
        };
        assert!(is_date_only(&prop));
    }

    #[test]
    fn test_parse_ics_default_timezone() {
        let ics_data = r#"BEGIN:VCALENDAR
X-WR-TIMEZONE:America/New_York
BEGIN:VEVENT
UID:test@example.com
DTSTART:20250204T103000
DTEND:20250204T111500
SUMMARY:Test Event
END:VEVENT
END:VCALENDAR"#;

        let events = parse_ics("test", ics_data).unwrap();
        assert_eq!(events.len(), 1);

        let naive = NaiveDateTime::parse_from_str("20250204T103000", "%Y%m%dT%H%M%S").unwrap();
        let tz = chrono_tz::America::New_York;
        let local = match tz.from_local_datetime(&naive) {
            LocalResult::Single(dt) => dt,
            LocalResult::Ambiguous(earliest, _) => earliest,
            LocalResult::None => tz.from_utc_datetime(&naive),
        };
        let expected = local.with_timezone(&Utc);

        assert_eq!(events[0].start, expected);
    }

    #[test]
    fn test_is_date_only_with_longer_value() {
        let prop = Property {
            name: "DTSTART".to_string(),
            value: Some("20250115T140000Z".to_string()),
            params: None,
        };
        assert!(!is_date_only(&prop));
    }

    #[test]
    fn test_is_date_only_with_none() {
        let prop = Property {
            name: "DTSTART".to_string(),
            value: None,
            params: None,
        };
        assert!(!is_date_only(&prop));
    }

    #[test]
    fn test_extract_tzid_with_tzid() {
        let params = vec![("TZID".to_string(), vec!["America/New_York".to_string()])];

        let prop = Property {
            name: "DTSTART".to_string(),
            value: Some("20250115T140000".to_string()),
            params: Some(params),
        };
        assert_eq!(extract_tzid(&prop), Some("America/New_York".to_string()));
    }

    #[test]
    fn test_extract_tzid_without_tzid() {
        let prop = Property {
            name: "DTSTART".to_string(),
            value: Some("20250115T140000Z".to_string()),
            params: None,
        };
        assert_eq!(extract_tzid(&prop), None);
    }

    #[test]
    fn test_parse_ics_empty() {
        let result = parse_ics("test", "");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_parse_ics_invalid() {
        let result = parse_ics("test", "not valid ics data");
        // Invalid ICS data returns an error
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_ics_basic_event() {
        let ics = r#"BEGIN:VCALENDAR
BEGIN:VEVENT
UID:test@example.com
DTSTART:20250115T100000Z
DTEND:20250115T110000Z
SUMMARY:Test Event
END:VEVENT
END:VCALENDAR"#;
        let result = parse_ics("test", ics);
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(&*events[0].title, "Test Event");
    }

    #[test]
    fn test_parse_ics_all_day_event() {
        let ics = r#"BEGIN:VCALENDAR
BEGIN:VEVENT
UID:test@example.com
DTSTART;VALUE=DATE:20250115
DTEND;VALUE=DATE:20250116
SUMMARY:All Day Event
END:VEVENT
END:VCALENDAR"#;
        let result = parse_ics("test", ics);
        assert!(result.is_ok());
        let events = result.unwrap();
        assert!(events[0].all_day);
    }

    #[test]
    fn test_parse_ics_with_location() {
        let ics = r#"BEGIN:VCALENDAR
BEGIN:VEVENT
UID:test@example.com
DTSTART:20250115T100000Z
DTEND:20250115T110000Z
SUMMARY:Meeting
LOCATION:Room A
END:VEVENT
END:VCALENDAR"#;
        let result = parse_ics("test", ics);
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events[0].location.as_deref(), Some("Room A"));
    }

    #[test]
    fn test_parse_ics_with_description() {
        let ics = r#"BEGIN:VCALENDAR
BEGIN:VEVENT
UID:test@example.com
DTSTART:20250115T100000Z
DTEND:20250115T110000Z
SUMMARY:Meeting
DESCRIPTION:Important meeting
END:VEVENT
END:VCALENDAR"#;
        let result = parse_ics("test", ics);
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events[0].description.as_deref(), Some("Important meeting"));
    }

    #[test]
    fn test_parse_ics_with_rrule() {
        let ics = r#"BEGIN:VCALENDAR
BEGIN:VEVENT
UID:weekly@example.com
DTSTART:20250115T100000Z
DTEND:20250115T110000Z
SUMMARY:Weekly Meeting
RRULE:FREQ=WEEKLY
END:VEVENT
END:VCALENDAR"#;
        let result = parse_ics("test", ics);
        assert!(result.is_ok());
        let events = result.unwrap();
        assert!(events[0].rrule.is_some());
    }

    #[test]
    fn test_parse_ics_missing_dtstart_returns_error() {
        let ics = r#"BEGIN:VCALENDAR
BEGIN:VEVENT
UID:test@example.com
SUMMARY:No DTSTART
END:VEVENT
END:VCALENDAR"#;
        let result = parse_ics("test", ics);
        assert!(result.is_ok());
        let events = result.unwrap();
        // Events without DTSTART are skipped
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_parse_ics_no_title_defaults() {
        let ics = r#"BEGIN:VCALENDAR
BEGIN:VEVENT
UID:test@example.com
DTSTART:20250115T100000Z
DTEND:20250115T110000Z
END:VEVENT
END:VCALENDAR"#;
        let result = parse_ics("test", ics);
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(&*events[0].title, "(No title)");
    }
}
