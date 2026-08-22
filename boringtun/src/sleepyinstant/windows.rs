use crate::sleepyinstant::ClockDuration;
use chrono::TimeDelta;

/// `winsafe::GetTickCount64` is a safe wrapper around the Win32 API of the same name: the
/// number of milliseconds since system startup, which (like Linux's `CLOCK_BOOTTIME`) keeps
/// advancing across system sleep.
pub(super) fn now() -> ClockDuration {
    TimeDelta::milliseconds(winsafe::GetTickCount64() as i64)
}
