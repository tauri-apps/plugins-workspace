use std::{convert::TryInto, num::ParseIntError, str::FromStr, time::Duration};

/// Describes the timeout of a notification
///
/// # `FromStr`
/// You can also parse a `Timeout` from a `&str`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Timeout {
    /// Expires according to server default.
    ///
    /// Whatever that might be...
    Default,

    /// Do not expire, user will have to close this manually.
    Never,

    /// Expire after n milliseconds.
    Milliseconds(u32),
}

impl Default for Timeout {
    fn default() -> Self {
        Timeout::Default
    }
}

#[test]
fn timeout_from_i32() {
    assert_eq!(Timeout::from(234), Timeout::Milliseconds(234));
    assert_eq!(Timeout::from(-234), Timeout::Default);
    assert_eq!(Timeout::from(0), Timeout::Never);
}

impl From<i32> for Timeout {
    fn from(int: i32) -> Timeout {
        use std::cmp::Ordering::*;
        match int.cmp(&0) {
            Greater => Timeout::Milliseconds(int as u32),
            Less => Timeout::Default,
            Equal => Timeout::Never,
        }
    }
}

impl From<Duration> for Timeout {
    fn from(duration: Duration) -> Timeout {
        if duration.is_zero() {
            Timeout::Never
        } else if duration.as_millis() > u32::MAX.into() {
            Timeout::Default
        } else {
            Timeout::Milliseconds(duration.as_millis().try_into().unwrap_or(u32::MAX))
        }
    }
}

impl From<Timeout> for i32 {
    fn from(timeout: Timeout) -> Self {
        match timeout {
            Timeout::Default => -1,
            Timeout::Never => 0,
            Timeout::Milliseconds(ms) => ms as i32,
        }
    }
}

impl FromStr for Timeout {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(Timeout::Default),
            "never" => Ok(Timeout::Never),
            milliseconds => Ok(Timeout::Milliseconds(u32::from_str(milliseconds)?)),
        }
    }
}

pub struct TimeoutMessage(Timeout);

impl From<Timeout> for TimeoutMessage {
    fn from(hint: Timeout) -> Self {
        TimeoutMessage(hint)
    }
}

impl std::ops::Deref for TimeoutMessage {
    type Target = Timeout;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(all(feature = "dbus", unix, not(target_os = "macos")))]
impl TryFrom<&dbus::arg::messageitem::MessageItem> for TimeoutMessage {
    type Error = ();

    fn try_from(mi: &dbus::arg::messageitem::MessageItem) -> Result<TimeoutMessage, ()> {
        mi.inner::<i32>().map(|i| TimeoutMessage(i.into()))
    }
}
