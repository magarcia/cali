use crate::error::{CaliError, Result};
use crate::storage::{CalendarSource, Config, Paths, SecureStorage};
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use tokio::time::timeout;

pub async fn fetch_calendars(config: &Config) -> Result<Vec<(String, String)>> {
    if config.sources.is_empty() {
        return Ok(Vec::new());
    }

    let paths = Paths::new()?;
    let secure_storage = SecureStorage::new(paths.config_dir());

    let mut sources_with_urls = Vec::new();
    for source in &config.sources {
        let url = secure_storage
            .get_url(&source.name)?
            .ok_or_else(|| CaliError::credential_not_found(source.name.clone()))?;
        sources_with_urls.push((source.clone(), url));
    }

    let pb = ProgressBar::new(sources_with_urls.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:20.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );
    pb.set_message("Fetching calendars...");

    let fetch_tasks: Vec<_> = sources_with_urls
        .iter()
        .map(|(source, url)| {
            let source = source.clone();
            let url = url.clone();
            let pb = pb.clone();
            async move {
                let result = fetch_single_calendar(&source, &url).await;
                pb.inc(1);
                result
            }
        })
        .collect();

    let results = join_all(fetch_tasks).await;

    pb.finish_with_message("Fetch complete");

    let mut successful = Vec::new();
    for result in results {
        match result {
            Ok(data) => successful.push(data),
            Err(e) => {
                eprintln!("Warning: {e}");
            }
        }
    }

    Ok(successful)
}

async fn fetch_single_calendar(source: &CalendarSource, url: &str) -> Result<(String, String)> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| CaliError::FetchFailure {
            name: source.name.clone(),
            source: Box::new(e),
        })?;

    let response = timeout(Duration::from_secs(10), client.get(url).send())
        .await
        .map_err(|_| CaliError::FetchFailure {
            name: source.name.clone(),
            source: "Request timed out".into(),
        })?
        .map_err(|e| CaliError::FetchFailure {
            name: source.name.clone(),
            source: Box::new(e),
        })?;

    if !response.status().is_success() {
        return Err(CaliError::FetchFailure {
            name: source.name.clone(),
            source: format!("HTTP {}", response.status()).into(),
        });
    }

    let text = response.text().await.map_err(|e| CaliError::FetchFailure {
        name: source.name.clone(),
        source: Box::new(e),
    })?;

    Ok((source.name.clone(), text))
}
