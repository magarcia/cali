use super::{Paths, SecureStorage};
use crate::error::{CaliError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarSource {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default)]
    pub last_sync: Option<DateTime<Utc>>,
}

fn default_color() -> String {
    "white".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DisplayConfig {
    #[serde(default = "default_time_format")]
    pub time_format: String,
    #[serde(default = "default_date_format")]
    pub date_format: String,
}

fn default_time_format() -> String {
    "%-I:%M%P".to_string()
}

fn default_date_format() -> String {
    "%a, %b %-d".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncConfig {
    #[serde(default = "default_sync_interval")]
    pub sync_interval_minutes: u64,
    #[serde(default = "default_cache_window")]
    pub cache_window_days: i64,
}

fn default_sync_interval() -> u64 {
    15
}

fn default_cache_window() -> i64 {
    365
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub sources: Vec<CalendarSource>,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub sync: SyncConfig,
    #[serde(default)]
    pub credentials_migrated: bool,
}

pub struct ConfigLoader {
    paths: Paths,
}

impl ConfigLoader {
    pub fn new(paths: Paths) -> Self {
        Self { paths }
    }

    pub fn load(&self) -> Result<Config> {
        let config_path = self.paths.config_file();

        if !config_path.exists() {
            return Err(CaliError::ConfigNotFound);
        }

        let contents = fs::read_to_string(&config_path)
            .map_err(|_e| CaliError::config_read(config_path.display().to_string()))?;

        let mut config: Config =
            toml::from_str(&contents).map_err(|e| CaliError::ConfigParse { source: e.into() })?;

        // Sanitize defaults for values that were explicitly set to 0
        if config.sync.sync_interval_minutes == 0 {
            config.sync.sync_interval_minutes = default_sync_interval();
        }
        if config.sync.cache_window_days == 0 {
            config.sync.cache_window_days = default_cache_window();
        }

        // Migrate credentials to secure storage if needed
        if !config.credentials_migrated {
            #[cfg(test)]
            let secure_storage = SecureStorage::new_for_testing(self.paths.config_dir());
            #[cfg(not(test))]
            let secure_storage = SecureStorage::new(self.paths.config_dir());
            let mut needs_save = false;

            for source in &mut config.sources {
                if let Some(url) = source.url.take() {
                    secure_storage.store_url(&source.name, &url)?;
                    needs_save = true;
                }
            }

            config.credentials_migrated = true;

            if needs_save {
                self.save(&config)?;

                if secure_storage.backend() == super::CredentialBackend::EncryptedFile {
                    eprintln!(
                        "Warning: System keychain not available. Calendar URLs are stored in an encrypted file.\n\
                         The file is encrypted using your machine ID, but for maximum security, consider using a system with keychain support."
                    );
                }
            }
        }

        Ok(config)
    }

    pub fn save(&self, config: &Config) -> Result<()> {
        let config_path = self.paths.config_file();
        let temp_path = config_path.with_extension("tmp");

        let toml = toml::to_string_pretty(config)
            .map_err(|e| CaliError::config_write(temp_path.display().to_string(), e))?;

        fs::write(&temp_path, toml)
            .map_err(|e| CaliError::config_write(temp_path.display().to_string(), e))?;

        fs::rename(&temp_path, &config_path)
            .map_err(|e| CaliError::config_write(config_path.display().to_string(), e))?;

        Ok(())
    }

    pub fn exists(&self) -> bool {
        self.paths.config_file().exists()
    }

    pub fn get_source_with_url(&self, name: &str) -> Result<(CalendarSource, String)> {
        let config = self.load()?;
        let source = config
            .sources
            .iter()
            .find(|s| s.name == name)
            .ok_or_else(|| CaliError::SourceNotFound {
                name: name.to_string(),
            })?
            .clone();

        #[cfg(test)]
        let secure_storage = SecureStorage::new_for_testing(self.paths.config_dir());
        #[cfg(not(test))]
        let secure_storage = SecureStorage::new(self.paths.config_dir());
        let url = secure_storage
            .get_url(name)?
            .ok_or_else(|| CaliError::credential_not_found(name.to_string()))?;

        Ok((source, url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_color_function() {
        assert_eq!(default_color(), "white");
    }

    #[test]
    fn test_default_time_format_function() {
        assert_eq!(default_time_format(), "%-I:%M%P");
    }

    #[test]
    fn test_default_date_format_function() {
        assert_eq!(default_date_format(), "%a, %b %-d");
    }

    #[test]
    fn test_default_sync_interval_function() {
        assert_eq!(default_sync_interval(), 15);
    }

    #[test]
    fn test_default_cache_window_function() {
        assert_eq!(default_cache_window(), 365);
    }

    #[test]
    fn test_calendar_source_deserialize_default_color() {
        let toml = "name = \"test\"\nurl = \"https://example.com\"";
        let source: CalendarSource = toml::from_str(toml).unwrap();
        assert_eq!(source.name, "test");
        assert_eq!(source.url, Some("https://example.com".to_string()));
        assert_eq!(source.color, "white");
    }

    #[test]
    fn test_calendar_source_deserialize_with_color() {
        let toml = "name = \"test\"\nurl = \"https://example.com\"\ncolor = \"#ff0000\"";
        let source: CalendarSource = toml::from_str(toml).unwrap();
        assert_eq!(source.name, "test");
        assert_eq!(source.url, Some("https://example.com".to_string()));
        assert_eq!(source.color, "#ff0000");
    }

    #[test]
    fn test_display_config_default() {
        let config = DisplayConfig::default();
        assert_eq!(config.time_format, "");
        assert_eq!(config.date_format, "");
    }

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert_eq!(config.sync_interval_minutes, 0);
        assert_eq!(config.cache_window_days, 0);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.sources.is_empty());
        assert_eq!(config.display.time_format, "");
        assert_eq!(config.sync.sync_interval_minutes, 0);
        assert!(!config.credentials_migrated);
    }

    #[test]
    fn test_calendar_source_with_optional_url() {
        let toml = "name = \"test\"\ncolor = \"#ff0000\"";
        let source: CalendarSource = toml::from_str(toml).unwrap();
        assert_eq!(source.name, "test");
        assert_eq!(source.url, None);
        assert_eq!(source.color, "#ff0000");
    }

    #[test]
    fn test_migration_from_old_config() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let paths = Paths::with_base(temp_dir.path());
        let config_loader = ConfigLoader::new(paths.clone());

        let old_config_toml = r##"
[[sources]]
name = "work"
url = "https://example.com/work.ics"
color = "#ff0000"

[[sources]]
name = "personal"
url = "https://example.com/personal.ics"
color = "#00ff00"
"##;

        fs::write(paths.config_file(), old_config_toml).unwrap();

        let config = config_loader.load().unwrap();

        assert_eq!(config.sources.len(), 2);
        assert!(config.sources[0].url.is_none());
        assert!(config.sources[1].url.is_none());
        assert!(config.credentials_migrated);

        let secure_storage = SecureStorage::new_for_testing(paths.config_dir());
        let work_url = secure_storage.get_url("work").unwrap();
        let personal_url = secure_storage.get_url("personal").unwrap();

        assert_eq!(work_url, Some("https://example.com/work.ics".to_string()));
        assert_eq!(
            personal_url,
            Some("https://example.com/personal.ics".to_string())
        );
    }

    #[test]
    fn test_migration_idempotent() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let paths = Paths::with_base(temp_dir.path());
        let config_loader = ConfigLoader::new(paths.clone());

        let old_config_toml = r##"
[[sources]]
name = "work"
url = "https://example.com/work.ics"
color = "#ff0000"
"##;

        fs::write(paths.config_file(), old_config_toml).unwrap();

        let config1 = config_loader.load().unwrap();
        assert!(config1.credentials_migrated);

        let config2 = config_loader.load().unwrap();
        assert!(config2.credentials_migrated);
        assert_eq!(config1.sources.len(), config2.sources.len());
    }

    #[test]
    fn test_migration_marks_complete_even_without_urls() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let paths = Paths::with_base(temp_dir.path());
        let config_loader = ConfigLoader::new(paths.clone());

        let new_config_toml = r##"
credentials_migrated = false

[[sources]]
name = "work"
color = "#ff0000"
"##;

        fs::write(paths.config_file(), new_config_toml).unwrap();

        let config = config_loader.load().unwrap();
        assert!(config.credentials_migrated);
    }

    #[test]
    fn test_get_source_with_url() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let paths = Paths::with_base(temp_dir.path());
        let config_loader = ConfigLoader::new(paths.clone());
        let secure_storage = SecureStorage::new_for_testing(paths.config_dir());

        secure_storage
            .store_url("test", "https://example.com/test.ics")
            .unwrap();

        let config = Config {
            sources: vec![CalendarSource {
                name: "test".to_string(),
                url: None,
                color: "#ff0000".to_string(),
                last_sync: None,
            }],
            credentials_migrated: true,
            ..Default::default()
        };

        config_loader.save(&config).unwrap();

        let (source, url) = config_loader.get_source_with_url("test").unwrap();
        assert_eq!(source.name, "test");
        assert_eq!(url, "https://example.com/test.ics");
    }

    #[test]
    fn test_get_source_with_url_not_found() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let paths = Paths::with_base(temp_dir.path());
        let config_loader = ConfigLoader::new(paths.clone());

        let config = Config {
            sources: vec![CalendarSource {
                name: "test-no-creds".to_string(),
                url: None,
                color: "#ff0000".to_string(),
                last_sync: None,
            }],
            credentials_migrated: true,
            ..Default::default()
        };

        config_loader.save(&config).unwrap();

        let result = config_loader.get_source_with_url("test-no-creds");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CaliError::CredentialNotFound { .. }
        ));
    }
}
