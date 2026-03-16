use std::time::Duration;

/// Formats a number of seconds to a duration string
/// 90f64 would become "1:30"
pub fn format_duration(seconds: f64) -> String {
    let duration = Duration::from_secs_f64(seconds);
    let total_secs = duration.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    format!("{}:{:02}", minutes, seconds)
}