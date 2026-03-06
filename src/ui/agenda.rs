use crate::date::{now_utc, today};
use crate::storage::Event;
use crate::ui::styles::{Color, Style, dim, styled};
use chrono::{DateTime, Duration, Local, Timelike, Utc};
use std::collections::HashMap;

pub struct EventGroup {
    pub date: chrono::NaiveDate,
    pub events: Vec<Event>,
    pub is_past: bool,
    pub is_today: bool,
    pub is_future: bool,
}

pub fn render_agenda(
    events: &[Event],
    grep: Option<&str>,
    source_colors: &HashMap<String, String>,
) -> String {
    let use_ansi = crate::ui::styles::use_color();

    if events.is_empty() {
        let msg = "No events found";
        return if use_ansi {
            dim(msg).render(true)
        } else {
            msg.to_string()
        };
    }

    let grouped = group_events_by_date(events);
    let now = now_utc();
    let today = today();

    let mut output = String::new();

    for group in grouped {
        let is_today = group.date == today;
        let date_style = if is_today {
            Style::new().fg(Color::Cyan).bold()
        } else if group.is_past {
            Style::new().dim()
        } else {
            Style::new()
        };

        let date_str = if is_today {
            format!("Today ({})", group.date.format("%a, %b %-d"))
        } else {
            group.date.format("%a, %b %-d").to_string()
        };

        output.push_str(&styled(&date_str, date_style).render(use_ansi));
        output.push('\n');

        for event in &group.events {
            output.push_str(&render_event(event, &now, use_ansi, is_today, source_colors));
        }

        output.push('\n');
    }

    if let Some(term) = grep {
        output.push_str(&dim(&format!("(Filtered by: {term})")).render(use_ansi));
        output.push('\n');
    }

    output
}

fn render_event(
    event: &Event,
    now: &DateTime<Utc>,
    use_ansi: bool,
    _is_today: bool,
    source_colors: &HashMap<String, String>,
) -> String {
    let mut output = String::new();

    let is_past = event.end < *now;
    let is_current = event.start <= *now && event.end >= *now;

    let local_start = event.start.with_timezone(&Local);
    let local_end = event.end.with_timezone(&Local);

    // Format time with padding for alignment
    // Time format: "hh:mmam" or " h:mmam" (space-padded for single digit hours)
    // Full range: "hh:mmam - hh:mmam" = 7 + " - " + 7 = 17 chars
    // "all day" padded to same width for alignment
    let time_str = if event.all_day {
        "all day          ".to_string()
    } else {
        let format_time = |dt: &DateTime<Local>| -> String {
            let (_, hour) = dt.hour12(); // hour12() returns (is_pm, hour)
            let minute = dt.minute();
            let am_pm = if dt.hour() < 12 { "am" } else { "pm" };
            format!("{hour:>2}:{minute:02}{am_pm}")
        };
        format!(
            "{} - {}",
            format_time(&local_start),
            format_time(&local_end)
        )
    };

    let dot_style = match source_colors.get(&*event.source).and_then(|c| Color::from_hex(c)) {
        Some(color) => Style::new().fg(color),
        None => Style::new().dim(),
    };

    let (prefix, title_style) = if is_current {
        ("  > ", Style::new().bold().fg(Color::Green))
    } else if is_past {
        ("    ", Style::new().dim())
    } else {
        ("    ", Style::new())
    };

    output.push_str(prefix);
    output.push_str(&styled("●", dot_style).render(use_ansi));
    output.push(' ');
    output.push_str(&styled(&time_str, title_style).render(use_ansi));
    output.push_str("  ");
    output.push_str(&styled(&event.title, title_style).render(use_ansi));

    let source_tag = format!(" [{}]", event.source);
    output.push_str(&styled(&source_tag, Style::new().dim()).render(use_ansi));

    if is_current && use_ansi {
        output.push_str(&styled(" ◀ NOW", Style::new().bold().fg(Color::Yellow)).render(true));
    }

    output.push('\n');

    if let Some(ref location) = event.location {
        output.push_str("      │ ");
        output.push_str(&styled(location, Style::new().dim()).render(use_ansi));
        output.push('\n');
    }

    if is_current {
        let remaining = event.end.signed_duration_since(*now);
        let remaining_str = format_duration(remaining);
        output.push_str("      │ ");
        output.push_str(
            &styled(&format!("{remaining_str} remaining"), Style::new().dim()).render(use_ansi),
        );
        output.push('\n');
    }

    output
}

fn format_duration(duration: Duration) -> String {
    let total_minutes = duration.num_minutes();
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

pub fn group_events_by_date(events: &[Event]) -> Vec<EventGroup> {
    let today = today();
    let mut grouped: HashMap<chrono::NaiveDate, Vec<Event>> = HashMap::new();

    for event in events {
        let date = event.start.with_timezone(&Local).date_naive();
        grouped.entry(date).or_default().push(event.clone());
    }

    let mut dates: Vec<_> = grouped.into_iter().collect();
    dates.sort_by_key(|(date, _)| *date);

    dates
        .into_iter()
        .map(|(date, events)| {
            let is_past = date < today;
            let is_today = date == today;
            let is_future = date > today;

            EventGroup {
                date,
                events,
                is_past,
                is_today,
                is_future,
            }
        })
        .collect()
}

pub fn filter_events(events: &[Event], grep: Option<&str>) -> Vec<Event> {
    match grep {
        Some(term) => {
            let term_lower = term.to_lowercase();
            events
                .iter()
                .filter(|e| {
                    e.title.to_lowercase().contains(&term_lower)
                        || e.location
                            .as_ref()
                            .is_some_and(|l| l.to_lowercase().contains(&term_lower))
                        || e.description
                            .as_ref()
                            .is_some_and(|d| d.to_lowercase().contains(&term_lower))
                })
                .cloned()
                .collect()
        }
        None => events.to_vec(),
    }
}

pub fn filter_events_by_range(
    events: &[Event],
    start: chrono::DateTime<Utc>,
    end: chrono::DateTime<Utc>,
) -> Vec<Event> {
    events
        .iter()
        .filter(|e| e.start <= end && e.end >= start)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_events_with_grep() {
        let events = vec![
            Event::new(
                "1".to_string(),
                "Team Meeting".to_string(),
                now_utc(),
                now_utc(),
                "work".to_string(),
            )
            .with_location(Some("Room A".to_string())),
            Event::new(
                "2".to_string(),
                "Lunch".to_string(),
                now_utc(),
                now_utc(),
                "personal".to_string(),
            ),
        ];

        let filtered = filter_events(&events, Some("meeting"));
        assert_eq!(filtered.len(), 1);
        assert_eq!(&*filtered[0].title, "Team Meeting");
    }

    #[test]
    fn test_filter_events_empty_grep() {
        let events = vec![Event::new(
            "1".to_string(),
            "Team Meeting".to_string(),
            now_utc(),
            now_utc(),
            "work".to_string(),
        )];

        let filtered = filter_events(&events, None);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_group_events_by_date() {
        let today = crate::date::today();
        let tomorrow = today + Duration::days(1);
        let now = crate::date::start_of_day(today);
        let tomorrow_start = crate::date::start_of_day(tomorrow);

        let events = vec![
            Event::new(
                "1".to_string(),
                "Event 1".to_string(),
                now,
                now,
                "test".to_string(),
            ),
            Event::new(
                "2".to_string(),
                "Event 2".to_string(),
                tomorrow_start,
                tomorrow_start,
                "test".to_string(),
            ),
        ];

        let grouped = group_events_by_date(&events);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].date, today);
        assert_eq!(grouped[1].date, tomorrow);
    }
}
