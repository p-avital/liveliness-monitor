use async_liveliness_monitor::LivelinessMonitor;

const ONE_SEC: std::time::Duration = std::time::Duration::from_millis(1000);

#[async_std::main]
async fn main() {
    let (task, monitor) = LivelinessMonitor::start(async_std::task::spawn);
    for _ in 0..4 {
        std::thread::sleep(ONE_SEC);
        if monitor.latest_report().elapsed() < ONE_SEC {
            println!("async_std executor is still going strong!")
        }
    }
    // Dropping the monitor's last shared owner kills the monitoring task.
    std::mem::drop(monitor);
    task.await; // the task will end as soon as the monitor is dropped.
    println!("monitor task joined, exiting.")
}
