use crate::cli::{ConfigCommand, OutputFormat};
use crate::commands::sync::spawn_background_sync;
use crate::error::{CaliError, Result};
use crate::storage::{CalendarSource, Config, ConfigLoader, Paths, SecureStorage};
use crate::sync::perform_sync_quick;
use inquire::{Confirm, Text};
use std::process::Command;

pub async fn handle_config(action: ConfigCommand, output_format: OutputFormat) -> Result<()> {
    let paths = Paths::new()?;
    let config_loader = ConfigLoader::new(paths.clone());

    match action {
        ConfigCommand::Add { name, url, color } => {
            let config = if config_loader.exists() {
                config_loader.load()?
            } else {
                Config::default()
            };

            let name = name.unwrap_or_else(|| {
                Text::new("Calendar name:")
                    .with_placeholder("e.g., work, personal")
                    .prompt()
                    .unwrap()
            });

            if config.sources.iter().any(|s| s.name == name) {
                return Err(CaliError::SourceExists { name });
            }

            let url = url.unwrap_or_else(|| {
                Text::new("ICS URL:")
                    .with_placeholder("https://calendar.google.com/calendar/ical/...")
                    .prompt()
                    .unwrap()
            });

            validate_url(&url)?;

            let color = color.unwrap_or_else(|| {
                Text::new("Color (hex, optional):")
                    .with_default("#ffffff")
                    .with_placeholder("#ff6b6b")
                    .prompt()
                    .unwrap()
            });

            let source = CalendarSource {
                name: name.clone(),
                url: None,
                color,
                last_sync: None,
            };

            let mut new_config = config;
            new_config.sources.push(source);
            new_config.sources.sort_by(|a, b| a.name.cmp(&b.name));

            config_loader.save(&new_config)?;

            let secure_storage = SecureStorage::new(paths.config_dir());
            if let Err(e) = secure_storage.store_url(&name, &url) {
                new_config.sources.retain(|s| s.name != name);
                config_loader.save(&new_config).ok();
                return Err(e);
            }
            println!("Calendar '{name}' added successfully.");

            if Confirm::new("Sync now?")
                .with_default(true)
                .prompt()
                .unwrap_or(false)
            {
                sync_and_exit(new_config, paths).await?;
            }
        }

        ConfigCommand::Remove { name } => {
            let mut config = config_loader.load()?;

            let name = name.unwrap_or_else(|| {
                let names: Vec<_> = config.sources.iter().map(|s| s.name.as_str()).collect();
                if names.is_empty() {
                    eprintln!("No calendar sources configured.");
                    std::process::exit(0);
                }
                inquire::Select::new("Select calendar to remove:", names)
                    .prompt()
                    .unwrap()
                    .to_string()
            });

            let index = config
                .sources
                .iter()
                .position(|s| s.name == name)
                .ok_or_else(|| CaliError::SourceNotFound { name: name.clone() })?;

            config.sources.remove(index);
            config_loader.save(&config)?;

            let secure_storage = SecureStorage::new(paths.config_dir());
            if let Err(e) = secure_storage.delete_url(&name) {
                eprintln!("Warning: Failed to delete credentials for '{}': {}", name, e);
            }

            println!("Calendar '{name}' removed.");
        }

        ConfigCommand::List { show_urls } => {
            if !config_loader.exists() {
                return Err(CaliError::ConfigNotFound);
            }

            let config = config_loader.load()?;

            if output_format == OutputFormat::Json {
                let secure_storage = SecureStorage::new(paths.config_dir());
                let sources_json: Vec<_> = config
                    .sources
                    .iter()
                    .map(|s| {
                        let mut obj = serde_json::json!({
                            "name": s.name,
                            "color": s.color,
                            "last_sync": s.last_sync,
                        });
                        if show_urls {
                            let url = secure_storage
                                .get_url(&s.name)
                                .ok()
                                .flatten()
                                .unwrap_or_default();
                            obj["url"] = serde_json::json!(url);
                        }
                        obj
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&sources_json).unwrap());
                return Ok(());
            }

            if config.sources.is_empty() {
                println!("No calendar sources configured.");
                println!("Add one with: cali config add <name> <url>");
                return Ok(());
            }

            println!("Calendar sources:");

            if show_urls {
                let secure_storage = SecureStorage::new(paths.config_dir());
                for source in &config.sources {
                    let last_sync = source
                        .last_sync
                        .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| "Never".to_string());
                    println!("  [{}]", source.name);

                    let url = secure_storage
                        .get_url(&source.name)?
                        .unwrap_or_else(|| "<not found>".to_string());
                    println!("    URL: {}", url);

                    println!("    Color: {}", source.color);
                    println!("    Last sync: {last_sync}");
                    println!();
                }
            } else {
                for source in &config.sources {
                    let last_sync = source
                        .last_sync
                        .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| "Never".to_string());
                    println!("  [{}]", source.name);
                    println!("    Color: {}", source.color);
                    println!("    Last sync: {last_sync}");
                    println!();
                }
            }
        }

        ConfigCommand::Refresh => {
            let config = config_loader.load()?;
            sync_and_exit(config, paths).await?;
        }

        ConfigCommand::Edit => {
            let config_path = paths.config_file();
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

            let status = Command::new(&editor)
                .arg(&config_path)
                .status()
                .map_err(|e| CaliError::Io {
                    message: format!("Failed to open editor '{editor}'"),
                    source: e.into(),
                })?;

            if !status.success() {
                eprintln!("Editor exited with non-zero status");
            }

            println!("Config file: {}", config_path.display());
        }
    }

    Ok(())
}

fn validate_url(url: &str) -> Result<()> {
    if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("webcal://") {
        Ok(())
    } else {
        Err(CaliError::InvalidUrl {
            url: url.to_string(),
        })
    }
}

pub async fn sync_and_exit(config: Config, paths: Paths) -> Result<()> {
    setup_ctrlc();

    eprintln!("Syncing calendars (today first)...");

    match perform_sync_quick(&config, &paths).await {
        Ok(events) => {
            eprintln!("Synced {} events for today.", events.len());
            eprintln!("Continuing background sync...");
            spawn_background_sync();
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Sync failed: {e}");
            std::process::exit(1);
        }
    }
}

pub fn setup_ctrlc() {
    let _ = ctrlc::set_handler(|| {
        std::process::exit(130);
    });
}
