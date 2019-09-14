use std::sync::{Arc, Mutex};
use std::{
    env, thread,
    time::{Duration, Instant},
};

use oping::{Ping, PingResult};
use signal_hook::iterator::Signals;

use lp::PingStats;

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
