use cali::storage::{CACHE_VERSION, Event, EventCache, EventCacheLoader, Paths};
use chrono::{Duration, Utc};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_event_cache_new_sorts_and_dedupes() {
    let now = Utc::now();
    let event1 = Event::new("id1", "Event 1", now, now + Duration::hours(1), "test");
    let event2 = Event::new("id1", "Event 1", now, now + Duration::hours(1), "test"); // Duplicate
    let event3 = Event::new("id2", "Event 2", now - Duration::hours(1), now, "test");

    let cache = EventCache::new(
        now - Duration::days(1),
        now + Duration::days(1),
        vec![event1.clone(), event2, event3.clone()],
    );

    // Should have 2 events (deduped)
    assert_eq!(cache.events.len(), 2);
    // Should be sorted by start time
    assert_eq!(&*cache.events[0].id, "id2"); // Earlier event
    assert_eq!(&*cache.events[1].id, "id1");
}

#[test]
fn test_event_cache_is_valid_for() {
    let now = Utc::now();
    let window_start = now - Duration::days(30);
    let window_end = now + Duration::days(365);

    let cache = EventCache::new(window_start, window_end, vec![]);

    // Exact window - valid
    assert!(cache.is_valid_for(window_start, window_end));

    // Smaller window inside cache - valid
    assert!(cache.is_valid_for(
        window_start + Duration::days(10),
        window_end - Duration::days(10)
    ));

    // Wider window - invalid
    assert!(!cache.is_valid_for(window_start - Duration::days(1), window_end));

    assert!(!cache.is_valid_for(window_start, window_end + Duration::days(1)));

    // Wrong version - invalid (simulate by modifying version)
    let mut wrong_version_cache = cache.clone();
    wrong_version_cache.version = CACHE_VERSION - 1;
    assert!(!wrong_version_cache.is_valid_for(window_start, window_end));
}

#[test]
fn test_event_cache_loader_write_and_load() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let paths = Paths::with_base(temp_dir.path());
    let loader = EventCacheLoader::new(paths.clone());

    // Create cache directory
    fs::create_dir_all(paths.cache_file().parent().unwrap())?;

    let now = Utc::now();
    let event = Event::new(
        "test-id",
        "Test Event",
        now,
        now + Duration::hours(1),
        "test",
    );

    let cache = EventCache::new(
        now - Duration::days(1),
        now + Duration::days(1),
        vec![event],
    );

    // Write cache
    loader.write(&cache)?;

    // Load cache
    let loaded = loader.load()?.expect("Cache should exist");

    assert_eq!(loaded.version, cache.version);
    assert_eq!(loaded.events.len(), cache.events.len());
    assert_eq!(&*loaded.events[0].id, "test-id");

    Ok(())
}

#[test]
fn test_event_cache_loader_returns_none_when_missing() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let paths = Paths::with_base(temp_dir.path());
    let loader = EventCacheLoader::new(paths);

    let loaded = loader.load()?;
    assert!(loaded.is_none());

    Ok(())
}

#[test]
fn test_event_cache_loader_corrupt_cache() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let paths = Paths::with_base(temp_dir.path());
    let cache_path = paths.cache_file();

    // Create parent dir if needed
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write garbage
    fs::write(&cache_path, b"definitely not valid bincode")?;

    let loader = EventCacheLoader::new(paths);
    let result = loader.load();

    // Should return error for corrupt cache
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_event_cache_loader_mtime() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let paths = Paths::with_base(temp_dir.path());
    let loader = EventCacheLoader::new(paths.clone());

    // No cache yet
    assert!(loader.mtime()?.is_none());

    // Create cache directory
    fs::create_dir_all(paths.cache_file().parent().unwrap())?;

    // Write cache
    let now = Utc::now();
    let cache = EventCache::new(now - Duration::days(1), now + Duration::days(1), vec![]);
    loader.write(&cache)?;

    // Should have mtime now
    assert!(loader.mtime()?.is_some());

    Ok(())
}

#[test]
fn test_event_cache_loader_is_stale() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let paths = Paths::with_base(temp_dir.path());
    let loader = EventCacheLoader::new(paths.clone());

    // No cache = stale
    assert!(loader.is_stale(1000)?);

    // Create cache directory
    fs::create_dir_all(paths.cache_file().parent().unwrap())?;

    // Write cache
    let now = Utc::now();
    let cache = EventCache::new(now - Duration::days(1), now + Duration::days(1), vec![]);
    loader.write(&cache)?;

    // Fresh cache = not stale
    assert!(!loader.is_stale(1000)?);

    Ok(())
}

#[test]
fn test_event_cache_uses_correct_version() {
    let now = Utc::now();
    let cache = EventCache::new(now - Duration::days(1), now + Duration::days(1), vec![]);

    assert_eq!(cache.version, 5); // Should match CACHE_VERSION
}
