use crate::commands::config::setup_ctrlc;
use crate::commands::sync::spawn_background_sync;
use crate::error::{CaliError, Result};
use crate::storage::{CalendarSource, Config, ConfigLoader, Paths, SecureStorage};
use crate::sync::perform_sync_quick;
use inquire::{Confirm, Text};

pub async fn run_onboarding() -> Result<()> {
    println!("Welcome to Cali!");
    println!();
    println!("Cali is a minimalist CLI calendar that fetches events from ICS sources.");
    println!("Let's set up your first calendar source.");
    println!();

    let paths = Paths::new()?;
    let config_loader = ConfigLoader::new(paths.clone());

    let name = Text::new("Calendar name:")
        .with_placeholder("e.g., work, personal")
        .with_default("my-calendar")
        .prompt()
        .map_err(|e| CaliError::Io {
            message: "Failed to read calendar name".to_string(),
            source: e.into(),
        })?;

    let url = Text::new("ICS URL:")
        .with_placeholder("https://calendar.google.com/calendar/ical/...")
        .prompt()
        .map_err(|e| CaliError::Io {
            message: "Failed to read ICS URL".to_string(),
            source: e.into(),
        })?;

    validate_url(&url)?;

    let color = Text::new("Color (hex, optional):")
        .with_default("#ffffff")
        .with_placeholder("#ff6b6b")
        .prompt()
        .map_err(|e| CaliError::Io {
            message: "Failed to read color".to_string(),
            source: e.into(),
        })
        .unwrap_or_else(|_| "#ffffff".to_string());

    let secure_storage = SecureStorage::new(paths.config_dir());
    secure_storage.store_url(&name, &url)?;

    let source = CalendarSource {
        name: name.clone(),
        url: None,
        color,
        last_sync: None,
    };

    let config = Config {
        sources: vec![source],
        ..Default::default()
    };

    config_loader.save(&config)?;
    println!();
    println!("Configuration saved.");

    if Confirm::new("Sync now?")
        .with_default(true)
        .prompt()
        .unwrap_or(false)
    {
        setup_ctrlc();
        eprintln!("Syncing calendars (today first)...");

        match perform_sync_quick(&config, &paths).await {
            Ok(events) => {
                eprintln!("Synced {} events for today.", events.len());
                eprintln!("Continuing background sync...");
                spawn_background_sync();
            }
            Err(e) => {
                eprintln!("Sync failed: {e}");
                eprintln!("You can try syncing later with: cali sync");
            }
        }
    }

    println!();
    println!("You're all set! Try running:");
    println!("  cali          # Show today's agenda");
    println!("  cali tomorrow # Show tomorrow's events");
    println!("  cali week     # Show this week's events");
    println!();
    println!("For more options, run: cali --help");

    Ok(())
}

fn validate_url(url: &str) -> Result<()> {
    use crate::error::CaliError;

    if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("webcal://") {
        Ok(())
    } else {
        Err(CaliError::InvalidUrl {
            url: url.to_string(),
        })
    }
}
