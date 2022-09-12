use std::time::Duration;

/// Taken from [https://github.com/cars10/human-duration/blob/master/src/lib.rs]

static SECONDS_PER_YEAR: u64 = 31_536_000;
static SECONDS_PER_MONTH: u64 = 2_628_000;
static SECONDS_PER_DAY: u64 = 86400;
static SECONDS_PER_HOUR: u64 = 3600;
static SECONDS_PER_MINUTE: u64 = 60;

/// Takes a [`std::time::Duration`] and returns a formatted [`String`].
///
/// Examples:
/// ```
/// use human_duration::human_duration;
///
/// let duration = std::time::Duration::new(5, 0);
/// assert_eq!(human_duration(&duration), "5s 0ms");
///
/// let duration = std::time::Duration::new(125, 0);
/// assert_eq!(human_duration(&duration), "2m 5s 0ms");
///
/// let duration = std::time::Duration::new(45_000_000, 0);
/// assert_eq!(human_duration(&duration), "1y 5mon 3d 18h 0m 0s 0ms");
/// ```
pub fn human_duration(duration: &Duration) -> String {
    let seconds = duration.as_secs();
    let millis = duration.subsec_millis();

    let years = seconds / SECONDS_PER_YEAR;
    let mut remainder = seconds % SECONDS_PER_YEAR;

    let months = remainder / SECONDS_PER_MONTH;
    remainder %= SECONDS_PER_MONTH;

    let days = remainder / SECONDS_PER_DAY;
    remainder %= SECONDS_PER_DAY;

    let hours = remainder / SECONDS_PER_HOUR;
    remainder %= SECONDS_PER_HOUR;

    let minutes = remainder / SECONDS_PER_MINUTE;
    remainder %= SECONDS_PER_MINUTE;

    let mut output = Vec::with_capacity(7);

    if years > 0 {
        let f_years = format!("{years}y");
        output.push(f_years);
    }

    if !output.is_empty() || months > 0 {
        let f_months = format!("{months}mon");
        output.push(f_months);
    }

    if !output.is_empty() || days > 0 {
        let f_days = format!("{days}d");
        output.push(f_days);
    }

    if !output.is_empty() || hours > 0 {
        let f_hours = format!("{hours}h");
        output.push(f_hours);
    }

    if !output.is_empty() || minutes > 0 {
        let f_minutes = format!("{minutes}m");
        output.push(f_minutes);
    }

    if !output.is_empty() || seconds > 0 {
        let f_seconds = format!("{remainder}s");
        output.push(f_seconds);
    }

    let f_millis = format!("{millis}ms");
    output.push(f_millis);

    output.join(" ")
}
