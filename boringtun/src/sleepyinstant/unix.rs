use crate::sleepyinstant::ClockDuration;
use chrono::TimeDelta;
use nix::time::{clock_gettime, ClockId};

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "freebsd",
    target_os = "netbsd"
))]
const CLOCK_ID: ClockId = ClockId::CLOCK_MONOTONIC;
#[cfg(not(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "freebsd",
    target_os = "netbsd"
)))]
const CLOCK_ID: ClockId = ClockId::CLOCK_BOOTTIME;

pub(super) fn now() -> ClockDuration {
    // std::time::Instant unwraps as well, so feel safe doing so here
    let t = clock_gettime(CLOCK_ID).unwrap();
    TimeDelta::new(t.tv_sec(), t.tv_nsec() as u32).unwrap()
}
