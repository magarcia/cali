use cali::storage::{CalendarSource, Config, ConfigLoader, Paths};
use chrono::Utc;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_config_default() {
    let config = Config::default();

    assert!(config.sources.is_empty());
    // DisplayConfig uses Default trait, which gives empty strings
    assert_eq!(config.display.time_format, "");
    assert_eq!(config.display.date_format, "");
    // SyncConfig uses Default trait, which gives 0 values
    assert_eq!(config.sync.sync_interval_minutes, 0);
    assert_eq!(config.sync.cache_window_days, 0);
}

#[test]
fn test_config_loader_write_and_load() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let paths = Paths::with_base(temp_dir.path());
    let loader = ConfigLoader::new(paths.clone());

    let mut config = Config::default();
    config.credentials_migrated = true;
    config.sources.push(CalendarSource {
        name: "work".to_string(),
        url: Some("https://example.com/calendar.ics".to_string()),
        color: "#ff6b6b".to_string(),
        last_sync: Some(Utc::now()),
    });

    // Save config
    loader.save(&config)?;

    // Load config
    let loaded = loader.load()?;

    assert_eq!(loaded.sources.len(), 1);
    assert_eq!(loaded.sources[0].name, "work");
    assert_eq!(
        loaded.sources[0].url,
        Some("https://example.com/calendar.ics".to_string())
    );
    assert_eq!(loaded.sources[0].color, "#ff6b6b");
    assert!(loaded.sources[0].last_sync.is_some());

    Ok(())
}

#[test]
fn test_config_loader_returns_error_when_missing() {
    let temp_dir = TempDir::new().unwrap();
    let paths = Paths::with_base(temp_dir.path());
    let loader = ConfigLoader::new(paths);

    let result = loader.load();
    assert!(result.is_err());
}

#[test]
fn test_config_loader_exists() {
    let temp_dir = TempDir::new().unwrap();
    let paths = Paths::with_base(temp_dir.path());
    let loader = ConfigLoader::new(paths.clone());

    assert!(!loader.exists());

    // Create empty config
    let config = Config::default();
    loader.save(&config).unwrap();

    assert!(loader.exists());
}

#[test]
fn test_config_loader_sanitizes_zero_values() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let paths = Paths::with_base(temp_dir.path());
    let loader = ConfigLoader::new(paths.clone());

    // Write config with zeros
    let config_path = paths.config_file();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        &config_path,
        r#"
sources = []
[sync]
sync_interval_minutes = 0
cache_window_days = 0
"#,
    )?;

    let loaded = loader.load()?;

    // Should sanitize to defaults
    assert_eq!(loaded.sync.sync_interval_minutes, 15);
    assert_eq!(loaded.sync.cache_window_days, 365);

    Ok(())
}

#[test]
fn test_config_loader_preserves_non_zero_values() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let paths = Paths::with_base(temp_dir.path());
    let loader = ConfigLoader::new(paths.clone());

    // Write config with custom values
    let config_path = paths.config_file();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        &config_path,
        r#"
sources = []
[sync]
sync_interval_minutes = 30
cache_window_days = 180
"#,
    )?;

    let loaded = loader.load()?;

    assert_eq!(loaded.sync.sync_interval_minutes, 30);
    assert_eq!(loaded.sync.cache_window_days, 180);

    Ok(())
}

#[test]
fn test_config_loader_invalid_toml() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let paths = Paths::with_base(temp_dir.path());
    let loader = ConfigLoader::new(paths.clone());

    // Write invalid TOML
    let config_path = paths.config_file();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&config_path, "this is not valid toml [[[")?;

    let result = loader.load();
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_config_with_multiple_sources() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let paths = Paths::with_base(temp_dir.path());
    let loader = ConfigLoader::new(paths.clone());

    let mut config = Config::default();
    config.credentials_migrated = true;
    config.sources.push(CalendarSource {
        name: "work".to_string(),
        url: Some("https://example.com/work.ics".to_string()),
        color: "#ff6b6b".to_string(),
        last_sync: None,
    });
    config.sources.push(CalendarSource {
        name: "personal".to_string(),
        url: Some("https://example.com/personal.ics".to_string()),
        color: "#4ecdc4".to_string(),
        last_sync: None,
    });

    loader.save(&config)?;
    let loaded = loader.load()?;

    assert_eq!(loaded.sources.len(), 2);

    Ok(())
}

#[test]
fn test_calendar_source_default_color() {
    // Test deserialization with default color
    let toml = r#"
name = "test"
url = "https://example.com/calendar.ics"
"#;

    let source: CalendarSource = toml::from_str(toml).unwrap();
    assert_eq!(source.color, "white");
}

#[test]
fn test_config_display_defaults() {
    let config = Config::default();
    // DisplayConfig uses Default trait
    assert_eq!(config.display.time_format, "");
    assert_eq!(config.display.date_format, "");
}
