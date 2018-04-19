extern crate chan_signal;
use chan_signal::Signal;

extern crate oping;
use oping::{Ping, PingResult};
use std::sync::{Arc, Mutex};
use std::{env, thread, time::Duration};

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

fn main() {
    let ip = env::args().skip(1).next().unwrap_or("8.8.8.8".to_string());
    let total_pings = Arc::new(Mutex::new(0));
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);

    let ping_clone = total_pings.clone();
    thread::spawn(move || loop {
        ping(&ip).expect("failed to ping, are you running with sudo?");
        *ping_clone.lock().unwrap() += 1;
        thread::sleep(Duration::from_millis(1000));
    });

    signal.recv().unwrap();
    let total_pings = total_pings.clone();
    let total_pings = total_pings.lock().unwrap();
    println!("Total pings: {}", total_pings);
}
