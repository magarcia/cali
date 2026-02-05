// These fields ARE used by the thiserror macro's format strings,
// but rustc doesn't see the usage and emits false positive warnings.
#![allow(unused_assignments)]

use miette::Diagnostic;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, CaliError>;

#[derive(Error, Debug, Diagnostic)]
pub enum CaliError {
    #[error("Configuration not found. Run 'cali' to start the setup wizard.")]
    #[diagnostic(
        code(cali::no_config),
        help("Run 'cali' to set up your calendar sources.")
    )]
    ConfigNotFound,

    #[error("Failed to read config file: {path}")]
    #[diagnostic(code(cali::config_read))]
    ConfigRead { path: String },

    #[error("Failed to write config file: {path}")]
    #[diagnostic(code(cali::config_write))]
    ConfigWrite {
        path: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to parse config file")]
    #[diagnostic(code(cali::config_parse))]
    ConfigParse {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to read cache file: {path}")]
    #[diagnostic(code(cali::cache_read))]
    CacheRead { path: String },

    #[error("Failed to write cache file: {path}")]
    #[diagnostic(code(cali::cache_write))]
    CacheWrite {
        path: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Cache file is corrupted or incompatible. Run 'cali config refresh' to rebuild.")]
    #[diagnostic(
        code(cali::cache_corrupt),
        help("Run 'cali config refresh' to fetch fresh data.")
    )]
    CacheCorrupt,

    #[error("Failed to fetch calendar '{name}': {source}")]
    #[diagnostic(code(cali::fetch_failure))]
    FetchFailure {
        name: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to parse ICS data from '{name}': {source}")]
    #[diagnostic(code(cali::parse_failure))]
    ParseFailure {
        name: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to expand recurrence rules for '{name}': {source}")]
    #[diagnostic(code(cali::rrule_failure))]
    RruleFailure {
        name: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Invalid URL: {url}")]
    #[diagnostic(code(cali::invalid_url))]
    InvalidUrl { url: String },

    #[error("Calendar source '{name}' already exists")]
    #[diagnostic(
        code(cali::source_exists),
        help("Use 'cali config rm {name}' to remove the existing source first.")
    )]
    SourceExists { name: String },

    #[error("Calendar source '{name}' not found")]
    #[diagnostic(
        code(cali::source_not_found),
        help("Run 'cali config list' to see available sources.")
    )]
    SourceNotFound { name: String },

    #[error("No calendar sources configured")]
    #[diagnostic(
        code(cali::no_sources),
        help("Run 'cali config add <name> <url>' to add a calendar source.")
    )]
    NoSources,

    #[error("Failed to parse date: {input}")]
    #[diagnostic(
        code(cali::date_parse),
        help("Try formats like: 'today', 'tomorrow', 'next friday', '2025-12-25'")
    )]
    DateParse { input: String },

    #[error("IO error: {message}")]
    #[diagnostic(code(cali::io))]
    Io {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Sync already in progress (file lock held)")]
    #[diagnostic(
        code(cali::sync_locked),
        help("Wait for the current sync to complete or remove the lock file manually.")
    )]
    SyncLocked,

    #[error("Failed to access credential storage: {message}")]
    #[diagnostic(code(cali::credential_storage))]
    CredentialStorage {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Credential not found for calendar '{name}'")]
    #[diagnostic(
        code(cali::credential_not_found),
        help("The calendar URL may not have been stored. Try removing and re-adding the calendar.")
    )]
    CredentialNotFound { name: String },
}

impl CaliError {
    pub fn config_read(path: String) -> Self {
        Self::ConfigRead { path }
    }

    pub fn config_write(
        path: String,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::ConfigWrite {
            path,
            source: source.into(),
        }
    }

    pub fn cache_read(path: String) -> Self {
        Self::CacheRead { path }
    }

    pub fn cache_write(
        path: String,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::CacheWrite {
            path,
            source: source.into(),
        }
    }

    pub fn credential_storage(
        message: String,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::CredentialStorage {
            message,
            source: source.into(),
        }
    }

    pub fn credential_not_found(name: String) -> Self {
        Self::CredentialNotFound { name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_config_not_found() {
        let err = CaliError::ConfigNotFound;
        assert!(err.to_string().contains("Configuration not found"));
    }

    #[test]
    fn test_error_display_cache_corrupt() {
        let err = CaliError::CacheCorrupt;
        assert!(err.to_string().contains("Cache file is corrupted"));
    }

    #[test]
    fn test_error_display_no_sources() {
        let err = CaliError::NoSources;
        assert!(err.to_string().contains("No calendar sources"));
    }

    #[test]
    fn test_error_display_sync_locked() {
        let err = CaliError::SyncLocked;
        assert!(err.to_string().contains("Sync already in progress"));
    }

    #[test]
    fn test_error_display_source_exists() {
        let err = CaliError::SourceExists {
            name: "test".to_string(),
        };
        assert!(err.to_string().contains("already exists"));
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_error_display_source_not_found() {
        let err = CaliError::SourceNotFound {
            name: "test".to_string(),
        };
        assert!(err.to_string().contains("not found"));
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_error_display_date_parse() {
        let err = CaliError::DateParse {
            input: "baddate".to_string(),
        };
        assert!(err.to_string().contains("Failed to parse date"));
        assert!(err.to_string().contains("baddate"));
    }

    #[test]
    fn test_error_helpers() {
        let err = CaliError::config_read("/path/to/config".to_string());
        assert!(matches!(err, CaliError::ConfigRead { .. }));

        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let err = CaliError::config_write("/path/to/config".to_string(), io_err);
        assert!(matches!(err, CaliError::ConfigWrite { .. }));

        let err = CaliError::cache_read("/path/to/cache".to_string());
        assert!(matches!(err, CaliError::CacheRead { .. }));

        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test");
        let err = CaliError::cache_write("/path/to/cache".to_string(), io_err);
        assert!(matches!(err, CaliError::CacheWrite { .. }));
    }
}
