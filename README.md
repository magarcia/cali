<div align="center">

cali
====

**The terminal is for text.** A minimalist, offline-first CLI calendar designed for flow state.

</div>

`cali` is a command-line agenda that respects your intelligence and your system resources. It aggregates multiple calendar sources (Google, iCloud, Outlook) into a unified, linear stream.

It is engineered for **speed (<50ms startup)** and **natural language** interaction. It rejects complex grids and interactive widgets in favor of instant, pretty-printed text.

Examples
--------

`cali` uses typographic hierarchy (bold, dim, indentation) to focus your attention on the *now*.

```
$ cali

Today (Tue, Oct 24)

   09:00am - 09:30am  Standup meeting
   09:30am - 10:30am  Code review
   12:00pm - 01:00pm  Lunch

 > 01:00pm - 02:00pm  PROJECT KICKOFF ◀ NOW
   │ Meeting Room A
   │ 15m remaining

   02:00pm - 05:00pm  Deep work block
   06:00pm - 07:00pm  Grocery run

```

Query dates as you speak them:

```
cali                # Show agenda for today
cali tomorrow       # Show agenda for tomorrow
cali weekend        # Show agenda for this coming Saturday & Sunday
cali next friday    # Show agenda for the specific upcoming date
cali "mon to wed"   # Show a range of days
cali -g "standup"   # Filter events containing "standup"
cali -f 2026-01-01 -t 2026-01-31  # Show events in January 2026

```

Getting Started
---------------

1.  **Install `cali`** (see Installation below).

2.  **Add your calendars:**

    ```
    # Add a work calendar
    cali source add work "https://calendar.google.com/calendar/ical/..."

    # Add a personal calendar
    cali source add personal "https://p58-caldav.icloud.com/..."

    ```

    Calendar URLs are automatically stored securely in your system keychain.

3.  **Run it:**

    ```
    cali

    ```

Installation
------------

### Homebrew (Recommended)

```
brew install magarcia/tap/cali

```

### From Source

Clone the repository and build with Cargo:

```
git clone https://github.com/magarcia/cali
cd cali
cargo install --path .

```

Usage
-----

### Output Formats

```
cali                      # Human-readable (default)
cali --output json        # Machine-readable JSON
cali --output llm         # Concise format for LLM/agent consumption
```

JSON output emits an array of event objects with RFC 3339 timestamps:

```json
[
  {
    "id": "abc123",
    "title": "Standup",
    "start": "2026-03-06T09:00:00+01:00",
    "end": "2026-03-06T09:30:00+01:00",
    "source": "work",
    "all_day": false,
    "location": "Room A"
  }
]
```

### Global Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--output <FORMAT>` | `-o` | Output format: `text`, `json`, `llm` |
| `--verbose` | `-v` | Show detailed output (paths, sync status) |
| `--quiet` | `-q` | Suppress non-essential messages |
| `--no-color` | | Disable colored output |
| `--version` | | Show version with commit hash and build date |
| `--help` | | Show help with usage examples |

The `NO_COLOR` environment variable is also respected.

### Shell Completions

Generate completions for your shell:

```
cali completions bash > /etc/bash_completion.d/cali
cali completions zsh > ~/.zfunc/_cali
cali completions fish > ~/.config/fish/completions/cali.fish
```

### Calendar Management

-   **Add a source:** `cali source add <name> <url>` (or interactive: `cali source add`)

-   **List sources:** `cali source list`

-   **Show URLs:** `cali source list --show-urls` (URLs are hidden by default for security)

-   **Remove a source:** `cali source remove <name>` (or interactive: `cali source remove`)

-   **Sync calendars:** `cali sync`

-   **Edit config:** `cali config edit` (opens in $EDITOR)

Configuration
-------------

`cali` is zero-config by default, but you can tweak behaviors in `~/.config/cali/config.toml`.

```toml
[display]
# Time format using chrono format strings (default: "%-I:%M%P")
time_format = "%-I:%M%P"  # 12-hour format with lowercase am/pm
# time_format = "%H:%M"   # 24-hour format

# Date format using chrono format strings (default: "%a, %b %-d")
date_format = "%a, %b %-d"  # e.g., "Mon, Jan 5"

[sync]
# How often to sync calendars in the background (default: 15 minutes)
sync_interval_minutes = 15

# How many days of events to cache (default: 365 days)
cache_window_days = 365

```

For chrono format strings reference, see: https://docs.rs/chrono/latest/chrono/format/strftime/

### Security

Calendar URLs contain authentication tokens and should be kept secure. `cali` automatically stores them using:

-   **macOS:** System Keychain
-   **Linux:** Secret Service (gnome-keyring, KWallet)
-   **Fallback:** AES-256-GCM encrypted file (if keychain unavailable)

When you add a calendar, the URL is stored securely and removed from the config file. Existing configs are automatically migrated on first run.

URLs are **hidden by default** in `cali source list`. Use `--show-urls` to display them when needed.

### Data Storage

-   **Config:** `~/.config/cali/config.toml` (calendar metadata, no URLs)
-   **Cache:** `~/.cache/cali/events.bin` (event data for fast reads)
-   **Credentials:** System keychain or `~/.config/cali/credentials.enc` (encrypted)

### Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error |
| `2` | Usage error (invalid arguments or dates) |
| `3` | Configuration error |
| `4` | Sync/network error |
| `130` | Interrupted (Ctrl+C) |

Performance Philosophy
----------------------

`cali` is built on a "Read/Sync" split architecture to ensure it **never** hangs your terminal.

1.  **The Read Path (`cali`):** Reads a local binary cache. Zero logic. Zero network. <50ms execution time.

2.  **The Sync Path (Background):** Handles the heavy lifting of parsing ICS recurrence rules and normalizing timezones.

License
-------

MIT
