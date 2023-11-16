use std::time::Duration;

const MINUTE: u64 = 60;
const HOUR: u64 = MINUTE * 60;
const DAY: u64 = HOUR * 24;

pub fn format_duration(dur: &Duration) -> String {
    let secs = dur.as_secs();
    let within = |lower, upper| secs > lower && secs < upper;

    match dur.as_secs() {
        secs if within(MINUTE, HOUR) => format!("{}m, {}s", secs / MINUTE, secs % MINUTE),
        secs if within(HOUR, DAY) => format!("{}h, {}m", secs / HOUR, secs % HOUR),
        secs if secs > DAY => format!("{}d, {}h", secs / DAY, secs % DAY),
        secs => format!("{}s", secs),
    }
}
