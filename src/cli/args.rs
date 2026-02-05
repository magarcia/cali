use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "cali")]
#[command(about = "A minimalist, offline-first CLI calendar", long_about = None)]
#[command(version)]
#[command(help_expected = true)]
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

    /// Show internal sync information
    #[arg(long, hide = true)]
    pub debug_sync: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Manage calendar sources
    Config {
        #[command(subcommand)]
        action: ConfigCommand,
    },
    /// Internal: background sync worker
    #[command(name = "internal-sync", hide = true)]
    InternalSync,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
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

    /// Refresh all calendar sources
    Refresh,

    /// Edit configuration file
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
