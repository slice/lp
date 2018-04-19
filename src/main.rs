extern crate chan_signal;
use chan_signal::Signal;

extern crate oping;
use oping::{Ping, PingResult};
use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};

fn ping(host: &str) -> PingResult<f64> {
    let mut ping = Ping::new();
    ping.set_timeout(5.0)?;
    ping.add_host(host)?;
    let response = ping.send()?.next().expect("ping not found");
    println!(
        ">> {} ({}), {}ms",
        response.hostname, response.address, response.latency_ms
    );
    Ok(response.latency_ms)
}

fn main() {
    let total_pings = Arc::new(Mutex::new(0));
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);

    let ping_clone = total_pings.clone();
    thread::spawn(move || {
        loop {
            ping("8.8.8.8").expect("failed to ping, are you running with sudo?");
            let mut data = ping_clone.lock().unwrap();
            *data += 1;
            thread::sleep(Duration::from_millis(1000));
        }
    });

    let signal = signal.recv().unwrap();
    println!("died: {:?}", signal);
    let total_pings = total_pings.clone();
    let total_pings = total_pings.lock().unwrap();
    println!("Total pings: {}", total_pings);
}
