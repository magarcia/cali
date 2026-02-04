use super::Paths;
use crate::error::{CaliError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarSource {
    pub name: String,
    pub url: String,
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
        assert_eq!(source.color, "white");
    }

    #[test]
    fn test_calendar_source_deserialize_with_color() {
        let toml = "name = \"test\"\nurl = \"https://example.com\"\ncolor = \"#ff0000\"";
        let source: CalendarSource = toml::from_str(toml).unwrap();
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
    }
}
