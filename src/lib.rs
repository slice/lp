use std::fmt;

pub mod formatting;

/// Overall ping statistics.
pub struct PingStats {
    /// Total number of pings performed.
    pub total: usize,

    /// Total number of pings dropped.
    pub dropped: usize,

    /// Total number of pings successfully sent.
    pub sent: usize,

    /// A vec of all ping latencies performed.
    pub durations: Vec<f64>,
}

impl PingStats {
    pub fn new() -> Self {
        Self {
            total: 0,
            dropped: 0,
            sent: 0,
            durations: Vec::new(),
        }
    }

    pub fn min(&self) -> f64 {
        // Can't use `.min` here because `f64` isn't `Ord`.
        self
            .durations
            .iter()
            .fold(0.0_f64, |l, &r| l.max(r))
    }

    pub fn max(&self) -> f64 {
        // Can't use `.max` here because `f64` isn't `Ord`.
        self
            .durations
            .iter()
            .fold(0.0_f64, |l, &r| l.max(r))
    }

    pub fn mean(&self) -> f64 {
        if self.durations.len() == 0 {
            0.0
        } else {
            let sum: f64 = self.durations.iter().sum();
            sum / self.durations.len() as f64
        }
    }

    pub fn percentage_dropped(&self) -> f64 {
        return (self.dropped as f64 / self.total as f64) * 100.0;
    }
}

impl fmt::Display for PingStats {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "packets    ▸ {sent}/{total} sent, {dropped}/{total} ({percentage_dropped:.2}%) dropped, {total} total
latency    ▸ {mean:.2}ms mean, {min:.2}ms min, {max:.2}ms max",
            sent = self.sent,
            dropped = self.dropped,
            percentage_dropped = self.percentage_dropped(),
            total = self.total,
            mean = self.mean(),
            min = self.min(),
            max = self.max(),
        )
    }
}
