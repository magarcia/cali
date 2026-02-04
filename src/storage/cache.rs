use super::{CACHE_VERSION, Event};
use crate::error::{CaliError, Result};
use crate::storage::Paths;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventCache {
    pub version: u32,
    pub generated_at: DateTime<Utc>,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub events: Vec<Event>,
}

impl EventCache {
    pub fn new(window_start: DateTime<Utc>, window_end: DateTime<Utc>, events: Vec<Event>) -> Self {
        let mut events = events;
        events.sort();
        events.dedup_by(|a, b| a.id == b.id && a.start == b.start);

        Self {
            version: CACHE_VERSION,
            generated_at: Utc::now(),
            window_start,
            window_end,
            events,
        }
    }

    pub fn is_valid_for(&self, window_start: DateTime<Utc>, window_end: DateTime<Utc>) -> bool {
        self.version == CACHE_VERSION
            && self.window_start <= window_start
            && self.window_end >= window_end
    }
}

pub struct EventCacheLoader {
    paths: Paths,
}

impl EventCacheLoader {
    pub fn new(paths: Paths) -> Self {
        Self { paths }
    }

    pub fn load(&self) -> Result<Option<EventCache>> {
        let cache_path = self.paths.cache_file();

        if !cache_path.exists() {
            return Ok(None);
        }

        let bytes = fs::read(&cache_path)
            .map_err(|_e| CaliError::cache_read(cache_path.display().to_string()))?;

        bincode::deserialize::<EventCache>(&bytes)
            .map(Some)
            .map_err(|_| CaliError::CacheCorrupt)
    }

    pub fn write(&self, cache: &EventCache) -> Result<()> {
        let cache_path = self.paths.cache_file();
        let temp_path = cache_path.with_extension("tmp");

        let bytes = bincode::serialize(cache)
            .map_err(|e| CaliError::cache_write(temp_path.display().to_string(), e))?;

        let mut file = fs::File::create(&temp_path)
            .map_err(|e| CaliError::cache_write(temp_path.display().to_string(), e))?;

        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| CaliError::cache_write(temp_path.display().to_string(), e))?;

        fs::rename(&temp_path, &cache_path)
            .map_err(|e| CaliError::cache_write(cache_path.display().to_string(), e))?;

        Ok(())
    }

    pub fn mtime(&self) -> Result<Option<SystemTime>> {
        let cache_path = self.paths.cache_file();

        if !cache_path.exists() {
            return Ok(None);
        }

        let meta = fs::metadata(&cache_path).map_err(|e| CaliError::Io {
            message: format!("Failed to get metadata for: {}", cache_path.display()),
            source: e.into(),
        })?;

        let modified = meta.modified().map_err(|e| CaliError::Io {
            message: format!("Failed to get mtime for: {}", cache_path.display()),
            source: e.into(),
        })?;

        Ok(Some(modified))
    }

    pub fn is_stale(&self, max_age_seconds: u64) -> Result<bool> {
        let mtime = match self.mtime()? {
            Some(t) => t,
            None => return Ok(true),
        };

        let elapsed = SystemTime::now()
            .duration_since(mtime)
            .map_err(|e| CaliError::Io {
                message: "Clock went backwards".to_string(),
                source: e.into(),
            })?;

        Ok(elapsed.as_secs() > max_age_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Paths;
    use chrono::{Duration, Utc};

    #[test]
    fn test_event_cache_new_dedupes_events() {
        let now = Utc::now();
        let event1 = Event::new("id1", "Event 1", now, now + Duration::hours(1), "test");
        let event2 = Event::new("id1", "Event 1", now, now + Duration::hours(1), "test");

        let cache = EventCache::new(
            now - Duration::days(1),
            now + Duration::days(1),
            vec![event1, event2],
        );
        assert_eq!(cache.events.len(), 1);
    }

    #[test]
    fn test_event_cache_new_sorts_events() {
        let now = Utc::now();
        let event1 = Event::new(
            "id2",
            "Later Event",
            now + Duration::hours(2),
            now + Duration::hours(3),
            "test",
        );
        let event2 = Event::new(
            "id1",
            "Earlier Event",
            now,
            now + Duration::hours(1),
            "test",
        );

        let cache = EventCache::new(
            now - Duration::days(1),
            now + Duration::days(1),
            vec![event1, event2],
        );
        assert_eq!(&*cache.events[0].id, "id1");
        assert_eq!(&*cache.events[1].id, "id2");
    }

    #[test]
    fn test_event_cache_uses_current_version() {
        let now = Utc::now();
        let cache = EventCache::new(now - Duration::days(1), now + Duration::days(1), vec![]);
        assert_eq!(cache.version, CACHE_VERSION);
    }

    #[test]
    fn test_event_cache_is_valid_for_same_window() {
        let now = Utc::now();
        let cache = EventCache::new(now - Duration::days(30), now + Duration::days(365), vec![]);
        assert!(cache.is_valid_for(now - Duration::days(30), now + Duration::days(365)));
    }

    #[test]
    fn test_event_cache_is_valid_for_smaller_window() {
        let now = Utc::now();
        let cache = EventCache::new(now - Duration::days(30), now + Duration::days(365), vec![]);
        assert!(cache.is_valid_for(now - Duration::days(20), now + Duration::days(300)));
    }

    #[test]
    fn test_event_cache_not_valid_for_wider_window() {
        let now = Utc::now();
        let cache = EventCache::new(now - Duration::days(30), now + Duration::days(365), vec![]);
        assert!(!cache.is_valid_for(now - Duration::days(40), now + Duration::days(365)));
    }

    #[test]
    fn test_event_cache_not_valid_for_wrong_version() {
        let now = Utc::now();
        let mut cache =
            EventCache::new(now - Duration::days(30), now + Duration::days(365), vec![]);
        cache.version = CACHE_VERSION - 1;
        assert!(!cache.is_valid_for(now - Duration::days(30), now + Duration::days(365)));
    }

    #[test]
    fn test_event_cache_loader_new() {
        let temp = std::path::PathBuf::from("/tmp/test");
        let paths = Paths::with_base(&temp);
        let loader = EventCacheLoader::new(paths);
        // Just verify it doesn't panic
        let _ = loader;
    }
}
