use std::{
    sync::{
        atomic::{AtomicI64, Ordering},
        Weak,
    },
    time::{Duration, Instant},
};

use crate::LivelinessMonitor;

/// A signed [`Duration`] stored as an [`AtomicI64`].
///
/// Resolution: 1µs
/// Maximum offset: 278737 years (signed)
pub struct AtomicDuration {
    t: AtomicI64,
}
const SHIFT: i64 = 20;
const MASK: i64 = 0xfffff;
/// A sign to be paired with a [`Duration`].
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    #[default]
    Positive,
    Negative,
}
impl AtomicDuration {
    const fn i64_to_duration(mut t: i64) -> (Duration, Sign) {
        let sign = if t.is_negative() {
            t = -t;
            Sign::Negative
        } else {
            Sign::Positive
        };
        let micros = (t & MASK) as u32;
        let secs = t >> SHIFT;
        (Duration::new(secs as u64, micros * 1000), sign)
    }
    const fn duration_to_i64(t: Duration, sign: Sign) -> i64 {
        let t = ((t.as_secs() as i64) << SHIFT) + (t.subsec_micros() as i64);
        match sign {
            Sign::Positive => t,
            Sign::Negative => -t,
        }
    }
    /// This type's time resolution.
    pub const RESOLUTION: Duration = Self::i64_to_duration(1).0;
    /// This type's maximum value.
    pub const MAX: Duration = Self::i64_to_duration(i64::MAX).0;
    /// Atomically loads the stored value, converting it to a duration-sign tuple.
    ///
    /// The [`Ordering`] is used in a single `load` operation.
    pub fn load(&self, ord: Ordering) -> (Duration, Sign) {
        Self::i64_to_duration(self.t.load(ord))
    }
    /// Converts the duration-sign tuple into before storing it atomically.
    ///
    /// The [`Ordering`] is used in a single `store` operation.
    pub fn store(&self, duration: Duration, sign: Sign, ord: Ordering) {
        self.t.store(Self::duration_to_i64(duration, sign), ord)
    }
    pub fn new(duration: Duration, sign: Sign) -> Self {
        Self {
            t: AtomicI64::new(Self::duration_to_i64(duration, sign)),
        }
    }
}

/// An [`Instant`], stored as an [`AtomicDuration`] offset from an arbitrary epoch.
///
/// Due to [`Instant`] not having any const epoch, that epoch is taken by calling [`Instant::now()`] at construction.
/// [`AtomicInstant`]'s range and resolution are bound by those of [`AtomicDuration`].
///
/// Defaults to [`Instant::now()`].
pub struct AtomicInstant {
    epoch: Instant,
    since_epoch: AtomicDuration,
}
impl Default for AtomicInstant {
    fn default() -> Self {
        Self::new(Instant::now())
    }
}
impl AtomicInstant {
    /// Constructs a new [`AtomicInstant`], using [`Instant::now()`] as epoch.
    pub fn new(instant: Instant) -> Self {
        let epoch = Instant::now();
        let (duration, sign) = if epoch > instant {
            (epoch - instant, Sign::Negative)
        } else {
            (instant - epoch, Sign::Positive)
        };
        AtomicInstant {
            epoch,
            since_epoch: AtomicDuration::new(duration, sign),
        }
    }
    /// Atomically loads the internal atomic using the specified [`Ordering`], and uses it to reconstruct the corresponding [`Instant`].
    pub fn load(&self, ord: Ordering) -> Instant {
        let (duration, sign) = self.since_epoch.load(ord);
        match sign {
            Sign::Positive => self.epoch + duration,
            Sign::Negative => self.epoch - duration,
        }
    }
    /// Converts the [`Instant`] into an atomically storeable value, and stores it atomically using the specified [`Ordering`].
    pub fn store(&self, instant: Instant, ord: Ordering) {
        match instant.checked_duration_since(self.epoch) {
            Some(duration) => {
                self.since_epoch.store(duration, Sign::Positive, ord);
            }
            None => self
                .since_epoch
                .store(self.epoch - instant, Sign::Negative, ord),
        }
    }
    /// A shortcut for `self.store(std::time::Instant::now(), ord)`.
    pub fn store_now(&self, ord: Ordering) {
        self.store(Instant::now(), ord)
    }
}

pub struct LivelinessMonitorFuture {
    pub(crate) monitor: Weak<LivelinessMonitor>,
}
impl std::future::Future for LivelinessMonitorFuture {
    type Output = ();
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.get_mut().monitor.upgrade() {
            Some(monitor) => {
                monitor.latest_report.store_now(Ordering::Relaxed);
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            }
            None => std::task::Poll::Ready(()),
        }
    }
}
