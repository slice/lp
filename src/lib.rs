use std::fmt;

pub mod formatting;

#[derive(Clone)]
pub struct PingStats {
    pub total: usize,
    pub total_dropped: usize,
    pub total_sent: usize,
    pub latency_min: f64,
    pub latency_max: f64,
    pub latency_sum: f64,
}

impl PingStats {
    pub fn new() -> Self {
        Self {
            total: 0,
            total_dropped: 0,
            total_sent: 0,
            latency_min: 0.0,
            latency_max: 0.0,
            latency_sum: 0.0,
        }
    }

    pub fn record_dropped(&mut self) {
        self.total += 1;
        self.total_dropped += 1;
    }

    pub fn record(&mut self, latency: f64) {
        self.total += 1;
        self.total_sent += 1;

        self.latency_sum += latency;

        self.latency_min = if self.latency_min == 0.0 {
            latency
        } else {
            self.latency_min.min(latency)
        };
        self.latency_max = self.latency_max.max(latency);
    }

    pub fn mean(&self) -> f64 {
        if self.latency_sum == 0.0 {
            0.0
        } else {
            self.latency_sum / self.total_sent as f64
        }
    }

    pub fn percentage_dropped(&self) -> Option<f64> {
        if self.total == 0 {
            None
        } else {
            Some((self.total_dropped as f64 / self.total as f64) * 100.0)
        }
    }
}

impl fmt::Display for PingStats {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let percentage_dropped = self.percentage_dropped().map_or_else(
            || String::new(),
            |percentage| format!(" ({percentage:.2}%)"),
        );

        write!(
            formatter,
            "  stats: attempted {total} pings, {sent}/{total} OK, {dropped}/{total}\
            {percentage_dropped} dropped\n\
            latency: {mean:.2}ms mean, {min:.2}ms min, {max:.2}ms max",
            sent = self.total_sent,
            dropped = self.total_dropped,
            total = self.total,
            mean = self.mean(),
            min = self.latency_min,
            max = self.latency_max,
        )
    }
}
