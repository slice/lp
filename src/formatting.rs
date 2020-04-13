use std::time::Duration;

const ONE_MINUTE: u64 = 60;
const ONE_HOUR: u64 = ONE_MINUTE * 60;
const ONE_DAY: u64 = ONE_HOUR * 24;

pub fn format_duration(dur: &Duration) -> String {
    let secs = dur.as_secs();
    let within = |lower, upper| secs > lower && secs < upper;

    match dur.as_secs() {
        secs if within(ONE_MINUTE, ONE_HOUR) => format!("{}m, {}s", secs / 60, secs % 60),
        secs if within(ONE_HOUR, ONE_DAY) => format!("{}h, {}m", secs / ONE_HOUR, secs % ONE_HOUR),
        secs if secs > ONE_DAY => format!("{}d, {}h", secs / ONE_DAY, secs % ONE_DAY),
        secs => format!("{}s", secs),
    }
}
