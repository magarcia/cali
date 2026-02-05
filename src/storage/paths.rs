use crate::error::{CaliError, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Paths {
    config_dir: PathBuf,
    cache_dir: PathBuf,
}

impl Paths {
    pub fn new() -> Result<Self> {
        let proj_dirs =
            ProjectDirs::from("com", "github", "cali").ok_or_else(|| CaliError::Io {
                message: "Could not determine project directories".to_string(),
                source: "No home directory found".into(),
            })?;

        let config_dir = proj_dirs.config_dir().to_path_buf();
        let cache_dir = proj_dirs.cache_dir().to_path_buf();

        fs::create_dir_all(&config_dir).map_err(|e| CaliError::Io {
            message: format!(
                "Failed to create config directory: {}",
                config_dir.display()
            ),
            source: e.into(),
        })?;

        fs::create_dir_all(&cache_dir).map_err(|e| CaliError::Io {
            message: format!("Failed to create cache directory: {}", cache_dir.display()),
            source: e.into(),
        })?;

        Ok(Self {
            config_dir,
            cache_dir,
        })
    }

    pub fn config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    pub fn cache_file(&self) -> PathBuf {
        self.cache_dir.join("events.bin")
    }

    pub fn lock_file(&self) -> PathBuf {
        self.cache_dir.join("sync.lock")
    }

    /// Create a new Paths instance with a custom base directory (for testing)
    pub fn with_base(base: &std::path::Path) -> Self {
        Self {
            config_dir: base.to_path_buf(),
            cache_dir: base.join("cache"),
        }
    }
}
