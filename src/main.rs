use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::Mutex;
use std::{borrow::Cow, ops::DerefMut};
use std::{
    env, thread,
    time::{Duration, Instant},
};

use chrono::{DateTime, Local};
use oping::{Ping, PingItem, PingResult};
use signal_hook::{consts::SIGINT, iterator::Signals};

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

fn record_and_echo(
    user_provided_target: &str,
    ping_result: PingResult<PingItem>,
    mut stats: impl DerefMut<Target = PingStats>,
) {
    // "im not owned!  im not owned!!"
    let mut tag: String = "  ERROR".to_owned();
    let divider = '\u{25b8}';

    let resolved_target = match ping_result {
        Ok(ref result) if result.hostname != result.address => {
            // If we resolved a hostname that differs from the address, make it
            // visible to the user.
            Cow::Owned(format!(
                "{hostname} ({address})",
                hostname = result.hostname,
                address = result.address
            ))
        }
        _ => Cow::Borrowed(user_provided_target),
    };

    let remark: Cow<'_, str>;

    match ping_result {
        Err(error) => {
            remark = Cow::Owned(error.to_string());
            stats.record_dropped();
        }
        Ok(ref response) if response.latency_ms == -1.0 => {
            remark = Cow::Borrowed("timed out");
            stats.record_dropped();
        }
        Ok(ref response) => {
            let latency = response.latency_ms;
            stats.record(latency);

            tag = format!("{:>7}", stats.total_sent);
            remark = Cow::Owned(format!("{latency}ms"));
        }
    }

    println!("{tag} | {resolved_target} {divider} {remark}");
}

fn main() {
    let now = Instant::now();
    let now_datetime: DateTime<Local> = Local::now();

    let ping_target = env::args().nth(1).unwrap_or_else(|| "8.8.8.8".to_string());
    let stats = Mutex::new(PingStats::new());

    let next_sigint_exits = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register_conditional_shutdown(SIGINT, 1, Arc::clone(&next_sigint_exits))
        .expect("failed to register SIGINT conditional shutdown");
    signal_hook::flag::register(SIGINT, Arc::clone(&next_sigint_exits))
        .expect("failed to register SIGINT arming");

    let mut signals =
        Signals::new([signal_hook::consts::SIGINT]).expect("failed to create SIGINT handler");

    std::thread::scope(|scope| {
        scope.spawn(|| loop {
            let ping_result = send_ping(&ping_target);
            record_and_echo(
                &ping_target,
                ping_result,
                stats.lock().expect("stats mutex poisoned"),
            );
            thread::sleep(Duration::from_secs(1));
        });

        scope.spawn(|| {
            let _ = signals.into_iter().next();
            let started = now_datetime.format("%Y-%m-%d %H:%M:%S").to_string();

            // ^C doesn't drop us to the next line
            println!();

            println!(
                "{stats}\n   \
                time: pinged for {spent} (began {started})",
                stats = stats.lock().unwrap(),
                spent = format_duration(&now.elapsed()),
                started = started,
            );

            std::process::exit(0);
        });
    });
}
