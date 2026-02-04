use crate::error::Result;
use crate::storage::{Config, Paths};
use std::process::Command;

pub async fn sync_full_and_exit(config: Config, paths: Paths) -> Result<()> {
    crate::commands::config::setup_ctrlc();

    eprintln!("Syncing calendars...");

    match crate::sync::perform_sync(&config, &paths).await {
        Ok(events) => {
            eprintln!("Synced {} events.", events.len());
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Sync failed: {e}");
            std::process::exit(1);
        }
    }
}

pub async fn sync_and_exit(config: Config, paths: Paths) -> Result<()> {
    sync_full_and_exit(config, paths).await
}

pub fn spawn_background_sync() {
    let _ = Command::new(std::env::current_exe().unwrap_or_else(|_| "cali".into()))
        .arg("internal-sync")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}
