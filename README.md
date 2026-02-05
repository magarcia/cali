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

TUESDAY, OCTOBER 24

   09:00  Standup meeting [work]
   09:30  Code review [work]
   12:00  Lunch [personal]

 | 13:00  PROJECT KICKOFF
 |        Duration: 1h (15m remaining)
 |        Location: Meeting Room A
 |        [work]

   14:00  Deep work block [work]
   18:00  Grocery run [personal]

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
    cali config add work "https://calendar.google.com/calendar/ical/..."

    # Add a personal calendar
    cali config add personal "https://p58-caldav.icloud.com/..."

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

### Calendar Management

-   **Refresh manually:** `cali config refresh`

-   **List sources:** `cali config list`

-   **Show URLs:** `cali config list --show-urls` (URLs are hidden by default for security)

-   **Remove a source:** `cali config remove <name>` or `cali config remove` (interactive)

-   **Edit config:** `cali config edit` (opens in $EDITOR)

### Security

Calendar URLs contain authentication tokens and should be kept secure. `cali` automatically stores them using:

-   **macOS:** System Keychain
-   **Linux:** Secret Service (gnome-keyring, KWallet)
-   **Fallback:** AES-256-GCM encrypted file (if keychain unavailable)

When you add a calendar, the URL is stored securely and removed from the config file. Existing configs are automatically migrated on first run.

URLs are **hidden by default** in `cali config list`. Use `--show-urls` to display them when needed.

### Data Storage

-   **Config:** `~/.config/cali/config.toml` (calendar metadata, no URLs)
-   **Cache:** `~/.cache/cali/events.bin` (event data for fast reads)
-   **Credentials:** System keychain or `~/.config/cali/credentials.enc` (encrypted)

Performance Philosophy
----------------------

`cali` is built on a "Read/Sync" split architecture to ensure it **never** hangs your terminal.

1.  **The Read Path (`cali`):** Reads a local binary cache. Zero logic. Zero network. <50ms execution time.

2.  **The Sync Path (Background):** Handles the heavy lifting of parsing ICS recurrence rules and normalizing timezones.

License
-------

MIT
