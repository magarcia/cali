mod agenda;
mod styles;

pub use agenda::{EventGroup, filter_events, filter_events_by_range, render_agenda};
pub use styles::{Color, Style, force_no_color};
