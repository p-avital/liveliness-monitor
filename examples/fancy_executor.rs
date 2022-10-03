use async_liveliness_monitor::LivelinessMonitor;

const ONE_SEC: std::time::Duration = std::time::Duration::from_millis(1000);

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread().build().unwrap();
    let (task, monitor) = LivelinessMonitor::start(|f| runtime.spawn(f));
    for _ in 0..4 {
        std::thread::sleep(ONE_SEC);
        if monitor.latest_report().elapsed() < ONE_SEC {
            println!("tokio runtime is still going strong!")
        }
    }
    // Dropping the monitor's last shared owner kills the monitoring task.
    std::mem::drop(monitor);
    runtime.block_on(task).unwrap(); // the task will end as soon as the monitor is dropped.
    println!("monitor task joined, exiting.")
}
