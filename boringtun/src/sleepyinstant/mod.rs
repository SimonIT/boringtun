#![forbid(unsafe_code)]
//! Attempts to provide the same functionality as std::time::Instant, except it
//! uses a timer which accounts for time when the system is asleep

#[cfg(not(feature = "std"))]
use chrono::NaiveDateTime;
use chrono::TimeDelta;

#[cfg(all(unix, feature = "std"))]
mod unix;
#[cfg(all(unix, feature = "std"))]
use unix::now as clock_now;

#[cfg(all(windows, feature = "std"))]
mod windows;
#[cfg(all(windows, feature = "std"))]
use windows::now as clock_now;

use core::error::Error;
#[cfg(not(feature = "std"))]
use lock_api::Mutex;
#[cfg(not(feature = "std"))]
use once_cell::sync::Lazy;
use rtcc::DateTimeAccess;

#[cfg(not(feature = "std"))]
type RawMutex = spin::Mutex<()>;

/// The unit used for measuring spans of time between two [`Instant`]s.
pub type ClockDuration = TimeDelta;

#[cfg(not(feature = "std"))]
static BORING_CLOCK: Lazy<
    Mutex<RawMutex, Option<&'static mut (dyn DateTimeAccess<Error = ()> + Send + Sync)>>,
> = Lazy::new(|| Mutex::new(None));

/// Register the wall clock used by [`Instant::now`] on targets without `std`.
///
/// Must be called once at startup before any tunnel is used; every subsequent
/// [`Instant::now`] call reads through this clock. Since this takes a `'static` reference
/// rather than owning the clock, callers without a heap allocator can obtain one via a
/// statically allocated cell (e.g. `static_cell::StaticCell`) instead of `Box::leak`.
#[cfg(not(feature = "std"))]
pub fn set_wall_clock(clock: &'static mut (dyn DateTimeAccess<Error = ()> + Send + Sync)) {
    *BORING_CLOCK.lock() = Some(clock);
}

#[cfg(not(feature = "std"))]
fn clock_now() -> ClockDuration {
    let mut clock = BORING_CLOCK.lock();
    let clock = clock
        .as_mut()
        .expect("no wall clock registered; call sleepyinstant::set_wall_clock() at startup");
    let now = clock.datetime().unwrap().and_utc();
    TimeDelta::new(now.timestamp(), now.timestamp_subsec_nanos()).unwrap()
}

/// A measurement of a monotonically nondecreasing clock.
/// Opaque and useful only with [`ClockDuration`].
///
/// Instants are always guaranteed, barring [platform bugs], to be no less than any previously
/// measured instant when created, and are often useful for tasks such as measuring
/// benchmarks or timing how long an operation takes.
///
/// Note, however, that instants are **not** guaranteed to be **steady**. In other
/// words, each tick of the underlying clock might not be the same length (e.g.
/// some seconds may be longer than others). An instant may jump forwards or
/// experience time dilation (slow down or speed up), but it will never go
/// backwards.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug)]
pub struct Instant(ClockDuration);

impl Instant {
    /// Returns an instant corresponding to "now".
    pub fn now() -> Self {
        Self(clock_now())
    }

    fn checked_duration_since(&self, earlier: Instant) -> Option<ClockDuration> {
        const NANOSECOND: i32 = 1_000_000_000;
        let self_nanos = self.0.subsec_nanos();
        let earlier_nanos = earlier.0.subsec_nanos();
        let (secs, nanos) = if self_nanos < earlier_nanos {
            (
                self.0.num_seconds() - earlier.0.num_seconds() - 1,
                self_nanos - earlier_nanos + NANOSECOND,
            )
        } else {
            (
                self.0.num_seconds() - earlier.0.num_seconds(),
                self_nanos - earlier_nanos,
            )
        };

        if secs < 0 {
            None
        } else {
            Some(TimeDelta::new(secs, nanos as u32).unwrap())
        }
    }

    /// Returns the amount of time elapsed from another instant to this one,
    /// or zero duration if that instant is later than this one.
    pub fn duration_since(&self, earlier: Instant) -> ClockDuration {
        self.checked_duration_since(earlier)
            .unwrap_or(TimeDelta::zero())
    }

    /// Returns the amount of time elapsed since this instant was created.
    pub fn elapsed(&self) -> ClockDuration {
        Self::now().duration_since(*self)
    }

    /// Returns the amount of time elapsed since the clock's epoch to this instant.
    ///
    /// The epoch is whatever fixed reference point the underlying clock source uses (e.g.
    /// system boot for the unix/windows monotonic clocks); it is not guaranteed to relate to
    /// the Unix epoch except on targets whose wall clock backs [`Instant`] directly.
    pub fn duration_since_epoch(&self) -> ClockDuration {
        self.0
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use chrono::TimeDelta;

    #[test]
    fn time_increments_after_sleep() {
        let sleep_time = TimeDelta::milliseconds(10);
        let start = Instant::now();
        std::thread::sleep(sleep_time.to_std().unwrap());
        assert!(start.elapsed() >= sleep_time);
    }
}
