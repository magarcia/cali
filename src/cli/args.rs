use clap::{Parser, Subcommand, ValueEnum};

fn build_version() -> &'static str {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const COMMIT: Option<&str> = option_env!("CALI_COMMIT_HASH");
    const DATE: Option<&str> = option_env!("CALI_BUILD_DATE");

    match (COMMIT, DATE) {
        (Some(commit), Some(date)) => {
            // Leak is fine — called once, lives for the program duration
            Box::leak(format!("{VERSION} ({commit} {date})").into_boxed_str())
        }
        _ => VERSION,
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Llm,
}

#[derive(Parser, Debug)]
#[command(name = "cali")]
#[command(about = "A minimalist, offline-first CLI calendar", long_about = None)]
#[command(version = build_version())]
#[command(help_expected = true)]
#[command(after_help = "\
EXAMPLES:
    cali                          Show today's events
    cali tomorrow                 Show tomorrow
    cali week                     Show this week
    cali weekend                  Show Saturday & Sunday
    cali \"next friday\"            Show next Friday
    cali \"mon to wed\"             Show Monday through Wednesday
    cali -g standup               Filter events by search term
    cali -f 2026-01-01 -t 2026-01-31   Date range with ISO dates
    cali --output json            Machine-readable JSON output
    cali source add work <url>    Add a calendar source
    cali source list              List all calendar sources
    cali sync                     Refresh all calendars")]
pub struct Args {
    /// Natural language date filter (e.g., "tomorrow", "next friday", "weekend")
    #[arg(value_name = "DATE", conflicts_with_all = ["from", "to"])]
    pub date: Option<String>,

    /// Start date (ISO format: YYYY-MM-DD)
    #[arg(short = 'f', long, value_name = "DATE")]
    pub from: Option<String>,

    /// End date (ISO format: YYYY-MM-DD)
    #[arg(short = 't', long, value_name = "DATE")]
    pub to: Option<String>,

    /// Filter events by search term
    #[arg(short = 'g', long, value_name = "TERM")]
    pub grep: Option<String>,

    /// Output format
    #[arg(
        short = 'o',
        long = "output",
        value_name = "FORMAT",
        default_value = "text",
        global = true
    )]
    pub output_format: OutputFormat,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Show detailed output
    #[arg(short = 'v', long, global = true)]
    pub verbose: bool,

    /// Suppress non-essential output
    #[arg(short = 'q', long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Show internal sync information
    #[arg(long, hide = true)]
    pub debug_sync: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Manage calendar sources
    Source {
        #[command(subcommand)]
        action: SourceCommand,
    },
    /// Refresh all calendar sources
    Sync,
    /// Edit configuration file
    Config {
        #[command(subcommand)]
        action: ConfigCommand,
    },
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Internal: background sync worker
    #[command(name = "internal-sync", hide = true)]
    InternalSync,
}

#[derive(Subcommand, Debug)]
pub enum SourceCommand {
    /// Add a new calendar source
    Add {
        /// Calendar alias/name
        #[arg(value_name = "NAME")]
        name: Option<String>,

        /// ICS/WebCal URL
        #[arg(value_name = "URL")]
        url: Option<String>,

        /// Hex color code for display (e.g., "#ff6b6b")
        #[arg(short = 'c', long, value_name = "COLOR")]
        color: Option<String>,
    },

    /// Remove a calendar source
    Remove {
        /// Calendar alias/name
        #[arg(value_name = "NAME")]
        name: Option<String>,
    },

    /// List all calendar sources
    List {
        /// Show calendar URLs (hidden by default for security)
        #[arg(long)]
        show_urls: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Open configuration file in $EDITOR
    Edit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DateFilter {
    Today,
    Tomorrow,
    Yesterday,
    Week,
    Weekend,
    Month,
}
