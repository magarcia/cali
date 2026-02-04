use cali::storage::{CalendarSource, Config, Paths};
use cali::sync::{expand_recurrence, fetch_calendars, parse_ics, perform_sync};
use chrono::{Duration, Utc};
use tempfile::TempDir;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_fetch_calendars_empty_sources() {
    let config = Config {
        sources: vec![],
        ..Default::default()
    };

    let result = fetch_calendars(&config).await.unwrap();
    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_fetch_calendars_single_source() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"BEGIN:VCALENDAR
BEGIN:VEVENT
UID:test@example.com
DTSTART:20250115T100000Z
DTEND:20250115T110000Z
SUMMARY:Test Event
END:VEVENT
END:VCALENDAR"#,
        ))
        .mount(&mock_server)
        .await;

    let config = Config {
        sources: vec![CalendarSource {
            name: "test".to_string(),
            url: mock_server.uri(),
            color: "#ffffff".to_string(),
            last_sync: None,
        }],
        ..Default::default()
    };

    let result = fetch_calendars(&config).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, "test");
    assert!(result[0].1.contains("Test Event"));
}

#[tokio::test]
async fn test_fetch_calendars_multiple_sources() {
    let mock_server1 = MockServer::start().await;
    let mock_server2 = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("BEGIN:VCALENDAR\nEND:VCALENDAR"))
        .mount(&mock_server1)
        .await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("BEGIN:VCALENDAR\nEND:VCALENDAR"))
        .mount(&mock_server2)
        .await;

    let config = Config {
        sources: vec![
            CalendarSource {
                name: "cal1".to_string(),
                url: mock_server1.uri(),
                color: "#ffffff".to_string(),
                last_sync: None,
            },
            CalendarSource {
                name: "cal2".to_string(),
                url: mock_server2.uri(),
                color: "#ffffff".to_string(),
                last_sync: None,
            },
        ],
        ..Default::default()
    };

    let result = fetch_calendars(&config).await.unwrap();
    assert_eq!(result.len(), 2);
}

#[tokio::test]
async fn test_fetch_calendars_handles_404() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let config = Config {
        sources: vec![CalendarSource {
            name: "test".to_string(),
            url: mock_server.uri(),
            color: "#ffffff".to_string(),
            last_sync: None,
        }],
        ..Default::default()
    };

    let result = fetch_calendars(&config).await.unwrap();
    // Should return empty list on failure (with warning printed to stderr)
    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_fetch_calendars_handles_timeout() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(15)))
        .mount(&mock_server)
        .await;

    let config = Config {
        sources: vec![CalendarSource {
            name: "test".to_string(),
            url: mock_server.uri(),
            color: "#ffffff".to_string(),
            last_sync: None,
        }],
        ..Default::default()
    };

    let result = fetch_calendars(&config).await.unwrap();
    assert_eq!(result.len(), 0);
}

#[test]
fn test_parse_ics_basic_event() {
    let ics_data = r#"BEGIN:VCALENDAR
BEGIN:VEVENT
UID:test@example.com
DTSTART:20250115T100000Z
DTEND:20250115T110000Z
SUMMARY:Test Event
END:VEVENT
END:VCALENDAR"#;

    let events = parse_ics("test", ics_data).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(&*events[0].title, "Test Event");
    assert_eq!(&*events[0].id, "test@example.com");
}

#[test]
fn test_parse_ics_all_day_event() {
    let ics_data = r#"BEGIN:VCALENDAR
BEGIN:VEVENT
UID:test@example.com
DTSTART;VALUE=DATE:20250115
DTEND;VALUE=DATE:20250116
SUMMARY:All Day Event
END:VEVENT
END:VCALENDAR"#;

    let events = parse_ics("test", ics_data).unwrap();
    assert_eq!(events.len(), 1);
    assert!(events[0].all_day);
}

#[test]
fn test_parse_ics_with_location_and_description() {
    let ics_data = r#"BEGIN:VCALENDAR
BEGIN:VEVENT
UID:test@example.com
DTSTART:20250115T100000Z
DTEND:20250115T110000Z
SUMMARY:Meeting
LOCATION:Conference Room A
DESCRIPTION:Quarterly review meeting
END:VEVENT
END:VCALENDAR"#;

    let events = parse_ics("test", ics_data).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].location.as_deref(), Some("Conference Room A"));
    assert_eq!(
        events[0].description.as_deref(),
        Some("Quarterly review meeting")
    );
}

#[test]
fn test_parse_ics_recurring_event() {
    let ics_data = r#"BEGIN:VCALENDAR
BEGIN:VEVENT
UID:weekly@example.com
DTSTART:20250115T100000Z
DTEND:20250115T110000Z
SUMMARY:Weekly Meeting
RRULE:FREQ=WEEKLY;BYDAY=MO,WE,FR
END:VEVENT
END:VCALENDAR"#;

    let events = parse_ics("test", ics_data).unwrap();
    assert_eq!(events.len(), 1);
    assert!(events[0].rrule.is_some());
    assert_eq!(
        events[0].rrule.as_deref(),
        Some("FREQ=WEEKLY;BYDAY=MO,WE,FR")
    );
}

#[test]
fn test_parse_ics_multiple_events() {
    let ics_data = r#"BEGIN:VCALENDAR
BEGIN:VEVENT
UID:event1@example.com
DTSTART:20250115T100000Z
DTEND:20250115T110000Z
SUMMARY:Event 1
END:VEVENT
BEGIN:VEVENT
UID:event2@example.com
DTSTART:20250115T120000Z
DTEND:20250115T130000Z
SUMMARY:Event 2
END:VEVENT
END:VCALENDAR"#;

    let events = parse_ics("test", ics_data).unwrap();
    assert_eq!(events.len(), 2);
}

#[test]
fn test_parse_ics_handles_missing_title() {
    let ics_data = r#"BEGIN:VCALENDAR
BEGIN:VEVENT
UID:test@example.com
DTSTART:20250115T100000Z
DTEND:20250115T110000Z
END:VEVENT
END:VCALENDAR"#;

    let events = parse_ics("test", ics_data).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(&*events[0].title, "(No title)");
}

#[test]
fn test_expand_recurrence_with_rrule() {
    let base = cali::storage::Event::new(
        "weekly@test.com",
        "Weekly Meeting",
        Utc::now(),
        Utc::now() + Duration::hours(1),
        "test",
    )
    .with_rrule(Some("FREQ=WEEKLY;BYDAY=MO".to_string()));

    let now = Utc::now();
    let result =
        expand_recurrence(&[base], now - Duration::days(30), now + Duration::days(365)).unwrap();
    // Should generate multiple occurrences
    assert!(result.len() > 1);
}

#[test]
fn test_expand_recurrence_without_rrule() {
    let event = cali::storage::Event::new(
        "once@test.com",
        "One-time Event",
        Utc::now(),
        Utc::now() + Duration::hours(1),
        "test",
    );

    let now = Utc::now();
    let result =
        expand_recurrence(&[event], now - Duration::days(1), now + Duration::days(1)).unwrap();
    assert_eq!(result.len(), 1);
}

#[test]
fn test_expand_recurrence_empty() {
    let now = Utc::now();
    let result = expand_recurrence(&[], now - Duration::days(1), now + Duration::days(1)).unwrap();
    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_perform_sync_creates_cache() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let paths = Paths::with_base(temp_dir.path());

    // Create cache directory
    std::fs::create_dir_all(paths.cache_file().parent().unwrap())?;

    let mock_server = MockServer::start().await;

    // Use today's date to ensure the event is within the sync window
    let today = Utc::now().format("%Y%m%dT100000Z").to_string();
    let today_end = Utc::now().format("%Y%m%dT110000Z").to_string();
    let ics_data = format!(
        r#"BEGIN:VCALENDAR
BEGIN:VEVENT
UID:test@example.com
DTSTART:{today}
DTEND:{today_end}
SUMMARY:Test Event
END:VEVENT
END:VCALENDAR"#
    );

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(ics_data))
        .mount(&mock_server)
        .await;

    let config = Config {
        sources: vec![CalendarSource {
            name: "test".to_string(),
            url: mock_server.uri(),
            color: "#ffffff".to_string(),
            last_sync: None,
        }],
        ..Default::default()
    };

    let events = perform_sync(&config, &paths).await?;
    assert!(!events.is_empty());

    // Check cache was created
    let cache_loader = cali::storage::EventCacheLoader::new(paths);
    assert!(cache_loader.load()?.is_some());

    Ok(())
}
