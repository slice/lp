use std::fmt;

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

    pub fn avg(&self) -> f64 {
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
            "{} sent, {} ({:.2}%) dropped ({} total, {:.2}ms avg)",
            self.sent,
            self.dropped,
            self.percentage_dropped(),
            self.total,
            self.avg()
        )
    }
}
