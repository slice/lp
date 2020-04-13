use std::sync::{Arc, Mutex};
use std::{
    env, thread,
    time::{Duration, Instant},
};

use oping::{Ping, PingItem, PingResult};
use signal_hook::iterator::Signals;
use chrono::{Local, DateTime};

use lp::{formatting::format_duration, PingStats};

fn send_ping(host: &str) -> PingResult<PingItem> {
    let mut ping = Ping::new();
    ping.set_timeout(2.0)?;
    ping.add_host(host)?;

    let response = ping
        .send()?
        .next()
        .expect("ping got thrown into the void somehow");

    Ok(response)
}

fn ping(ip: &str, stats: &Arc<Mutex<PingStats>>) {
    let mut stats = stats.lock().expect("failed to lock stats");

    match send_ping(&ip) {
        Err(error) => {
            eprintln!("  ERROR | {} ▸ {}", ip, error);
            stats.record_dropped();
        }
        Ok(ref response) if response.latency_ms == -1.0 => {
            eprintln!("  ERROR | {} ▸ timed out", ip);
            stats.record_dropped();
        }
        Ok(ref response) => {
            let latency = response.latency_ms;
            stats.record(latency);

            let target = if response.hostname == response.address {
                response.address.clone()
            } else {
                format!("{} ({})", response.hostname, response.address)
            };
            println!("{:>7} | {} ▸ {}ms", stats.total_sent, target, latency);
        }
    }
}

fn main() {
    let now = Instant::now();
    let now_dt: DateTime<Local> = Local::now();
    let ip = env::args().nth(1).unwrap_or_else(|| "8.8.8.8".to_string());
    let stats = Arc::new(Mutex::new(PingStats::new()));

    let ping_stats_handle = Arc::clone(&stats);
    let ping_thread = thread::spawn(move || loop {
        ping(&ip, &ping_stats_handle);
        thread::sleep(Duration::from_secs(1));
    });

    let signals = Signals::new(&[signal_hook::SIGINT]).expect("failed to create sigint handler");
    let signal_stats_handle = Arc::clone(&stats);
    thread::spawn(move || {
        let _ = signals.into_iter().next();

        let final_stats = signal_stats_handle.lock().expect("failed to lock stats");
        let started = now_dt.format("%Y-%m-%d %H:%M:%S").to_string();

        println!(
            "\n---\n{stats}\npinged for {spent} (started {started})",
            stats = final_stats,
            spent = format_duration(&now.elapsed()),
            started = started,
        );

        std::process::exit(0);
    });

    ping_thread.join().unwrap();
}
