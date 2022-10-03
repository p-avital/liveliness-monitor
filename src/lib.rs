use std::sync::Arc;

pub mod support;
use support::{AtomicInstant, LivelinessMonitorFuture};

/// A liveliness monitor for asynchronous runtimes.
///
/// Its only constructor ([`LivelinessMonitor::start()`]) returns it wrapped in an [`Arc`] with strong count 1.
/// Should that strong count reach 0 (due to dropping all the clones you may have made of that [`Arc`], or by
/// using [`Arc::try_unwrap()`]), the associated task spawned in your runtime will end next time upon its next
/// scheduling.
#[non_exhaustive]
pub struct LivelinessMonitor {
    /// The instant of the latest liveliness report.
    pub latest_report: AtomicInstant,
}
impl LivelinessMonitor {
    /// Starts a liveliness monitor on your asynchronous runtime (of which you must pass the `spawn` method),
    /// returning both the handle the runtime may have returned, as well as a reference counted [`LivelinessMonitor`].
    ///
    /// Please refer to the examples to learn more about its usage.
    pub fn start<T, SpawnFunction: Fn(LivelinessMonitorFuture) -> T>(
        spawn: SpawnFunction,
    ) -> (T, Arc<LivelinessMonitor>) {
        let this = Arc::new(LivelinessMonitor {
            latest_report: AtomicInstant::default(),
        });
        (
            spawn(LivelinessMonitorFuture {
                monitor: Arc::downgrade(&this),
            }),
            this,
        )
    }

    /// The instant of the latest liveliness report, as an [`std::time::Instant`].
    ///
    /// Keep in mind its resolution is limited to that of [`crate::support::AtomicDuration`],
    /// and that a busy executor may provide updates at rather low frequencies.
    ///
    /// You can probably expect that if the report hasn't been updated in the last 5 seconds,
    /// your executor is indeed stalled.
    pub fn latest_report(&self) -> std::time::Instant {
        self.latest_report
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}
