use chrono_tz::Tz;

const IANA_PREFIXES: [&str; 9] = [
    "Africa/",
    "America/",
    "Antarctica/",
    "Asia/",
    "Atlantic/",
    "Australia/",
    "Europe/",
    "Indian/",
    "Pacific/",
];

pub fn resolve_tzid(tzid: &str) -> Option<Tz> {
    let trimmed = tzid.trim();

    if let Ok(tz) = trimmed.parse::<Tz>() {
        return Some(tz);
    }

    if let Some(mapped) = map_windows_tzid(trimmed) {
        if let Ok(tz) = mapped.parse::<Tz>() {
            return Some(tz);
        }
    }

    if let Some(iana) = extract_iana_substring(trimmed) {
        if let Ok(tz) = iana.parse::<Tz>() {
            return Some(tz);
        }
    }

    if trimmed.eq_ignore_ascii_case("utc") || trimmed.eq_ignore_ascii_case("gmt") {
        return Some(chrono_tz::UTC);
    }

    None
}

fn extract_iana_substring(tzid: &str) -> Option<&str> {
    for prefix in IANA_PREFIXES {
        if let Some(idx) = tzid.find(prefix) {
            return Some(&tzid[idx..]);
        }
    }

    None
}

fn map_windows_tzid(tzid: &str) -> Option<&'static str> {
    const WINDOWS_TZ_MAP: &[(&str, &str)] = &[
        // US & Canada
        ("Pacific Standard Time", "America/Los_Angeles"),
        ("Mountain Standard Time", "America/Denver"),
        ("US Mountain Standard Time", "America/Phoenix"),
        ("Central Standard Time", "America/Chicago"),
        ("Eastern Standard Time", "America/New_York"),
        ("Alaskan Standard Time", "America/Anchorage"),
        ("Hawaiian Standard Time", "Pacific/Honolulu"),
        ("Atlantic Standard Time", "America/Halifax"),
        ("Newfoundland Standard Time", "America/St_Johns"),
        ("Central America Standard Time", "America/Guatemala"),
        // Mexico & South America
        ("Mexico Standard Time", "America/Mexico_City"),
        ("Pacific SA Standard Time", "America/Santiago"),
        ("SA Pacific Standard Time", "America/Bogota"),
        ("SA Western Standard Time", "America/La_Paz"),
        ("SA Eastern Standard Time", "America/Cayenne"),
        ("Argentina Standard Time", "America/Buenos_Aires"),
        ("E. South America Standard Time", "America/Sao_Paulo"),
        // Europe
        ("GMT Standard Time", "Europe/London"),
        ("Greenwich Standard Time", "Atlantic/Reykjavik"),
        ("W. Europe Standard Time", "Europe/Berlin"),
        ("Central Europe Standard Time", "Europe/Budapest"),
        ("Romance Standard Time", "Europe/Paris"),
        ("Central European Standard Time", "Europe/Warsaw"),
        ("W. Central Africa Standard Time", "Africa/Lagos"),
        ("E. Europe Standard Time", "Europe/Chisinau"),
        ("FLE Standard Time", "Europe/Kiev"),
        ("GTB Standard Time", "Europe/Bucharest"),
        ("Russian Standard Time", "Europe/Moscow"),
        ("Turkey Standard Time", "Europe/Istanbul"),
        // Asia
        ("India Standard Time", "Asia/Kolkata"),
        ("China Standard Time", "Asia/Shanghai"),
        ("Hong Kong Standard Time", "Asia/Hong_Kong"),
        ("Tokyo Standard Time", "Asia/Tokyo"),
        ("Korea Standard Time", "Asia/Seoul"),
        ("Singapore Standard Time", "Asia/Singapore"),
        ("Taipei Standard Time", "Asia/Taipei"),
        ("SE Asia Standard Time", "Asia/Bangkok"),
        ("Myanmar Standard Time", "Asia/Rangoon"),
        ("Bangladesh Standard Time", "Asia/Dhaka"),
        ("Pakistan Standard Time", "Asia/Karachi"),
        ("West Asia Standard Time", "Asia/Tashkent"),
        ("Arabian Standard Time", "Asia/Dubai"),
        ("Iran Standard Time", "Asia/Tehran"),
        ("Israel Standard Time", "Asia/Jerusalem"),
        // Australia & Pacific
        ("AUS Eastern Standard Time", "Australia/Sydney"),
        ("E. Australia Standard Time", "Australia/Brisbane"),
        ("AUS Central Standard Time", "Australia/Darwin"),
        ("Cen. Australia Standard Time", "Australia/Adelaide"),
        ("W. Australia Standard Time", "Australia/Perth"),
        ("Tasmania Standard Time", "Australia/Hobart"),
        ("New Zealand Standard Time", "Pacific/Auckland"),
        ("Fiji Standard Time", "Pacific/Fiji"),
        // Africa & Middle East
        ("South Africa Standard Time", "Africa/Johannesburg"),
        ("Egypt Standard Time", "Africa/Cairo"),
        ("Morocco Standard Time", "Africa/Casablanca"),
        ("E. Africa Standard Time", "Africa/Nairobi"),
        ("Middle East Standard Time", "Asia/Beirut"),
    ];

    WINDOWS_TZ_MAP
        .iter()
        .find(|(win_tz, _)| tzid.eq_ignore_ascii_case(win_tz))
        .map(|(_, iana_tz)| *iana_tz)
}
