use crate::cli::{Args, OutputFormat};
use crate::commands::sync::spawn_background_sync;
use crate::date::parse_date;
use crate::error::{CaliError, Result};
use crate::storage::{ConfigLoader, Event, EventCacheLoader, Paths};
use crate::ui::{filter_events, filter_events_by_range, render_agenda};

pub async fn show_agenda(args: Args) -> Result<()> {
    let paths = Paths::new()?;
    let config_loader = ConfigLoader::new(paths.clone());
    let cache_loader = EventCacheLoader::new(paths.clone());

    if !config_loader.exists() {
        return Err(CaliError::ConfigNotFound);
    }

    let config = config_loader.load()?;

    if config.sources.is_empty() {
        return Err(CaliError::NoSources);
    }

    if args.debug_sync {
        show_debug_info(&config, &cache_loader, &paths)?;
        return Ok(());
    }

    if args.verbose {
        eprintln!("Config: {}", paths.config_file().display());
        eprintln!("Cache: {}", paths.cache_file().display());
        eprintln!("Sources: {}", config.sources.len());
    }

    let date_range = if let Some(date_str) = args.date {
        parse_date(&date_str)?
    } else {
        crate::date::expand_to_range(args.from, args.to)?
    };

    let start = date_range.start_utc();
    let end = date_range.end_utc();

    let events = match cache_loader.load()? {
        Some(cache) if cache.is_valid_for(start, end) => {
            if args.debug_sync {
                eprintln!("Cache valid: {} events", cache.events.len());
                eprintln!(
                    "Cache window: {} to {}",
                    cache.window_start, cache.window_end
                );
            }
            if cache_loader.is_stale(config.sync.sync_interval_minutes * 60)? {
                if args.verbose {
                    eprintln!("Cache is stale, triggering background sync...");
                }
                spawn_background_sync();
            }
            cache.events
        }
        _ => {
            if args.verbose {
                eprintln!("No valid cache, triggering sync...");
            }
            spawn_background_sync();
            if args.output_format == OutputFormat::Json {
                println!("[]");
            } else if !args.quiet {
                eprintln!("Syncing calendars... Run 'cali' again in a moment.");
            }
            return Ok(());
        }
    };

    let filtered = filter_events_by_range(&events, start, end);
    let filtered = filter_events(&filtered, args.grep.as_deref());

    match args.output_format {
        OutputFormat::Json => print_json(&filtered),
        OutputFormat::Text => {
            let output = render_agenda(&filtered, args.grep.as_deref());
            println!("{output}");
        }
    }

    Ok(())
}

fn print_json(events: &[Event]) {
    use chrono::Local;
    use serde_json::json;

    let events_json: Vec<_> = events
        .iter()
        .map(|e| {
            let mut obj = json!({
                "id": &*e.id,
                "title": &*e.title,
                "start": e.start.with_timezone(&Local).to_rfc3339(),
                "end": e.end.with_timezone(&Local).to_rfc3339(),
                "source": &*e.source,
                "all_day": e.all_day,
            });
            if let Some(ref loc) = e.location {
                obj["location"] = json!(&**loc);
            }
            if let Some(ref desc) = e.description {
                obj["description"] = json!(&**desc);
            }
            obj
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&events_json).unwrap());
}

fn show_debug_info(
    config: &crate::storage::Config,
    cache_loader: &EventCacheLoader,
    paths: &Paths,
) -> Result<()> {
    use chrono::{DateTime, Utc};

    eprintln!("=== Cali Debug Info ===");
    eprintln!();

    eprintln!("Config path: {}", paths.config_file().display());
    eprintln!("Cache path: {}", paths.cache_file().display());
    eprintln!();

    eprintln!("Sources ({}):", config.sources.len());
    for source in &config.sources {
        eprintln!("  - [{}]", source.name);
        eprintln!("    URL: <stored in secure storage>");
        eprintln!("    Last sync: {:?}", source.last_sync);
    }
    eprintln!();

    match cache_loader.load()? {
        Some(cache) => {
            eprintln!("Cache:");
            eprintln!("  Version: {}", cache.version);
            eprintln!("  Generated: {}", cache.generated_at);
            eprintln!("  Window: {} to {}", cache.window_start, cache.window_end);
            eprintln!("  Total events: {}", cache.events.len());

            let now: DateTime<Utc> = Utc::now();
            let upcoming: Vec<_> = cache
                .events
                .iter()
                .filter(|e| e.start > now)
                .take(5)
                .collect();

            eprintln!();
            eprintln!("Next 5 upcoming events:");
            for event in upcoming {
                eprintln!("  - {} @ {}", event.title, event.start);
            }
        }
        None => {
            eprintln!("Cache: Not found or invalid");
        }
    }

    let is_stale = cache_loader.is_stale(config.sync.sync_interval_minutes * 60)?;
    if is_stale {
        eprintln!();
        eprintln!(
            "Cache is STALE (>{} minutes old)",
            config.sync.sync_interval_minutes
        );
    } else {
        eprintln!();
        eprintln!("Cache is fresh");
    }

    Ok(())
}
