use std::time::Duration;

pub fn format_duration(dur: &Duration) -> String {
    let secs = dur.as_secs();

    if secs > 60 && secs < 60 * 60 {
        format!("{}m, {}s", secs / 60, secs % 60)
    } else if secs > 60 * 60 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}s", secs)
    }
}
