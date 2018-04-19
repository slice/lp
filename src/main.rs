extern crate chan_signal;
use chan_signal::Signal;

extern crate oping;
use oping::{Ping, PingResult};
use std::sync::{Arc, Mutex};
use std::{fmt, env, thread, time::{Duration, Instant}};

struct PingStats {
    total: u32,
    dropped: u32,
    passed: u32,
    durations: Vec<f64>,
}

impl PingStats {
    fn new() -> PingStats {
        PingStats { total: 0, dropped: 0, passed: 0, durations: Vec::new() }
    }

    fn avg(&self) -> f64 {
        let sum: f64 = self.durations.iter().sum();
        sum / self.durations.len() as f64
    }
}

impl fmt::Display for PingStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} passed, {} dropped ({} total, {:.2}ms avg)", self.passed, self.dropped, self.total, self.avg())
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
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);

    let stats_clone = stats.clone();
    thread::spawn(move || loop {
        let mut stats = stats_clone.lock().unwrap();
        match ping(&ip) {
            Err(e) => {
                eprintln!("failed to ping {} ({})", ip, e);
                (*stats).dropped += 1;
            },
            Ok(latency) if latency == -1.0_f64 => {
                eprintln!("failed to ping {}, timed out", ip);
                (*stats).dropped += 1;
            },
            Ok(latency) => {
                (*stats).passed += 1;
                (*stats).durations.push(latency);
            }
        }
        (*stats).total += 1;
        thread::sleep(Duration::from_millis(1000));
    });

    signal.recv().unwrap();
    let stats = stats.clone();
    let stats = stats.lock().unwrap();
    println!("Statistics: {}, spent {} pinging", stats, pretty_duration(&now.elapsed()));
}
