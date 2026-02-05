mod cache;
mod config;
mod credentials;
mod paths;

pub use cache::{EventCache, EventCacheLoader};
pub use config::{CalendarSource, Config, ConfigLoader, DisplayConfig, SyncConfig};
pub use credentials::{CredentialBackend, SecureStorage};
pub use paths::Paths;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    pub id: Arc<str>,
    pub title: Arc<str>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub source: Arc<str>,
    pub location: Option<Arc<str>>,
    pub description: Option<Arc<str>>,
    pub all_day: bool,
    pub rrule: Option<Arc<str>>,
    pub tzid: Option<Arc<str>>,
    pub exdates: Vec<DateTime<Utc>>,
    pub recurrence_id: Option<DateTime<Utc>>,
}

impl Event {
    pub fn new(
        id: impl Into<Arc<str>>,
        title: impl Into<Arc<str>>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        source: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            start,
            end,
            source: source.into(),
            location: None,
            description: None,
            all_day: false,
            rrule: None,
            tzid: None,
            exdates: Vec::new(),
            recurrence_id: None,
        }
    }

    #[must_use = "use the builder chain to construct the Event"]
    pub fn with_location(mut self, location: Option<impl Into<Arc<str>>>) -> Self {
        self.location = location.map(|s| s.into());
        self
    }

    #[must_use = "use the builder chain to construct the Event"]
    pub fn with_description(mut self, description: Option<impl Into<Arc<str>>>) -> Self {
        self.description = description.map(|s| s.into());
        self
    }

    #[must_use = "use the builder chain to construct the Event"]
    pub fn with_all_day(mut self, all_day: bool) -> Self {
        self.all_day = all_day;
        self
    }

    #[must_use = "use the builder chain to construct the Event"]
    pub fn with_rrule(mut self, rrule: Option<impl Into<Arc<str>>>) -> Self {
        self.rrule = rrule.map(|s| s.into());
        self
    }

    #[must_use = "use the builder chain to construct the Event"]
    pub fn with_tzid(mut self, tzid: Option<impl Into<Arc<str>>>) -> Self {
        self.tzid = tzid.map(|s| s.into());
        self
    }

    #[must_use = "use the builder chain to construct the Event"]
    pub fn with_exdates(mut self, exdates: Vec<DateTime<Utc>>) -> Self {
        self.exdates = exdates;
        self
    }

    #[must_use = "use the builder chain to construct the Event"]
    pub fn with_recurrence_id(mut self, recurrence_id: Option<DateTime<Utc>>) -> Self {
        self.recurrence_id = recurrence_id;
        self
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start.cmp(&other.start)
    }
}

pub const CACHE_VERSION: u32 = 5;
