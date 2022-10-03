use async_liveliness_monitor::LivelinessMonitor;

use std::time::Duration;

const ONE_SEC: Duration = Duration::from_millis(1000);
fn main() {
    let (_, monitor) = LivelinessMonitor::start(async_std::task::spawn);
    for _ in 0..1000 {
        // Artificially parking all the threads on the executor (assuming you have less than 1000 cpu cores)
        async_std::task::spawn(async move { std::thread::park() });
    }
    loop {
        std::thread::sleep(ONE_SEC);
        if monitor.latest_report().elapsed() > ONE_SEC {
            println!(
                "async_std executor hasn't reported liveliness in the last {} seconds, it's likely stalled :(",
                monitor.latest_report().elapsed().as_secs_f32()
            )
        }
        if monitor.latest_report().elapsed() > 5 * ONE_SEC {
            println!("Giving up on this stalled executor.");
            return;
        }
    }
}
