use oping::{Ping, PingResult};
use signal_hook::iterator::Signals;
use std::sync::{Arc, Mutex};
use std::{
    env, fmt, thread,
    time::{Duration, Instant},
};

struct PingStats {
    total: u32,
    dropped: u32,
    sent: u32,
    durations: Vec<f64>,
}

impl PingStats {
    fn new() -> Self {
        Self {
            total: 0,
            dropped: 0,
            sent: 0,
            durations: Vec::new(),
        }
    }

    fn avg(&self) -> f64 {
        if self.durations.len() == 0 {
            0.0
        } else {
            let sum: f64 = self.durations.iter().sum();
            sum / self.durations.len() as f64
        }
    }

    fn percentage_dropped(&self) -> f64 {
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

fn ping(host: &str) -> PingResult<f64> {
    let mut ping = Ping::new();
    ping.set_timeout(5.0)?;
    ping.add_host(host)?;
    let response = ping.send()?.next().expect("ping got thrown into the void");
    let target = if response.hostname == response.address {
        response.address
    } else {
        format!("{} ({})", response.hostname, response.address)
    };
    println!(">> {}, {}ms", target, response.latency_ms);
    Ok(response.latency_ms)
}

fn pretty_duration(dur: &Duration) -> String {
    let secs = dur.as_secs();

    if secs > 60 && secs < 60 * 60 {
        format!("{}m, {}s", secs / 60, secs % 60)
    } else if secs > 60 * 60 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}s", secs)
    }
}

fn main() {
    let now = Instant::now();
    let ip = env::args().nth(1).unwrap_or_else(|| "8.8.8.8".to_string());
    let stats = Arc::new(Mutex::new(PingStats::new()));

    let signals = Signals::new(&[signal_hook::SIGINT]).expect("failed to create sigint handler");

    let ping_thread_stats_clone = Arc::clone(&stats);
    let ping_thread = thread::spawn(move || loop {
        {
            let mut stats = ping_thread_stats_clone
                .lock()
                .expect("failed to lock stats");

            match ping(&ip) {
                Err(error) => {
                    eprintln!("failed to ping {} ({})", ip, error);
                    stats.dropped += 1;
                }
                Ok(latency) if latency == -1.0 => {
                    eprintln!("failed to ping {}, timed out", ip);
                    stats.dropped += 1;
                }
                Ok(latency) => {
                    stats.sent += 1;
                    stats.durations.push(latency);
                }
            }

            stats.total += 1;
        }

        thread::sleep(Duration::from_secs(1));
    });

    let signal_thread_stats_clone = Arc::clone(&stats);
    thread::spawn(move || {
        for _ in &signals {
            let final_stats = signal_thread_stats_clone
                .lock()
                .expect("failed to lock stats");

            println!(
                "ping statistics: {}, spent {}",
                final_stats,
                pretty_duration(&now.elapsed())
            );

            std::process::exit(0);
        }
    });

    ping_thread.join().unwrap();
}
