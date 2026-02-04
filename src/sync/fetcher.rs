use crate::error::{CaliError, Result};
use crate::storage::{CalendarSource, Config};
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use tokio::time::timeout;

pub async fn fetch_calendars(config: &Config) -> Result<Vec<(String, String)>> {
    if config.sources.is_empty() {
        return Ok(Vec::new());
    }

    let pb = ProgressBar::new(config.sources.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:20.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );
    pb.set_message("Fetching calendars...");

    let fetch_tasks: Vec<_> = config
        .sources
        .iter()
        .map(|source| {
            let source = source.clone();
            let pb = pb.clone();
            async move {
                let result = fetch_single_calendar(&source).await;
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

async fn fetch_single_calendar(source: &CalendarSource) -> Result<(String, String)> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| CaliError::FetchFailure {
            name: source.name.clone(),
            source: Box::new(e),
        })?;

    let response = timeout(Duration::from_secs(10), client.get(&source.url).send())
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
