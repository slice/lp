use std::sync::{Arc, Mutex};
use std::{
    env, thread,
    time::{Duration, Instant},
};

use oping::{Ping, PingResult, PingItem};
use signal_hook::iterator::Signals;

use lp::{PingStats, formatting::format_duration};

fn ping(host: &str) -> PingResult<PingItem> {
    let mut ping = Ping::new();
    ping.set_timeout(2.0)?;
    ping.add_host(host)?;

    let response = ping
        .send()?
        .next()
        .expect("ping got thrown into the void somehow");

    Ok(response)
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
                    eprintln!("  ERROR | {} ▸ {}", ip, error);
                    stats.dropped += 1;
                }
                Ok(ref response) if response.latency_ms == -1.0 => {
                    eprintln!("  ERROR | {} ▸ timed out", ip);
                    stats.dropped += 1;
                }
                Ok(ref response) => {
                    stats.sent += 1;

                    let target = if response.hostname == response.address {
                        response.address.clone()
                    } else {
                        format!("{} ({})", response.hostname, response.address)
                    };
                    println!("{:>7} | {} ▸ {}ms", stats.sent, target, response.latency_ms);

                    stats.durations.push(response.latency_ms);
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

            println!();
            println!(
                "ping statistics: {}, spent {}",
                final_stats,
                format_duration(&now.elapsed())
            );

            std::process::exit(0);
        }
    });

    ping_thread.join().unwrap();
}
