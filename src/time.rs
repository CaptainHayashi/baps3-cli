//! Utilities for mapping between human-usable time units and BAPS3's
//! preferred time units.

/// Enum of available time units.
///
/// This does not contain every possible time unit anyone may want to use with
/// a BAPS3 client, but covers the main possibilities.
///
/// Each unit specified in terms of its equivalent in microseconds, which is
/// the unit used 'over the wire' when talking to BAPS3.
#[derive(Copy)]
pub enum TimeUnit {
    /// Hours (1 hour = 60 minutes)
    Hours,
    /// Minutes (1 minute = 60 seconds).
    Minutes,
    /// Seconds (1 second = 1,000 milliseconds).
    Seconds,
    /// Milliseconds (1 millisecond = 1,000 microseconds).
    Milliseconds,
    /// Microseconds (the BAPS3 base unit).
    Microseconds
}

impl TimeUnit {
    /// Returns the suffix of the given unit.
    ///
    /// This is mainly for use in human-readable times.
    pub fn suffix(&self) -> &'static str {
        match *self {
            TimeUnit::Hours        => "h",
            TimeUnit::Minutes      => "m",
            TimeUnit::Seconds      => "s",
            TimeUnit::Milliseconds => "ms",
            TimeUnit::Microseconds => "us"
        }
    }

    /// Returns the equivalent of `n` of the given unit in microseconds.
    pub fn as_micros(&self, n: u64) -> u64 {
        match *self {
            TimeUnit::Hours        => n * 1000 * 1000 * 60 * 60,
            TimeUnit::Minutes      => n * 1000 * 1000 * 60,
            TimeUnit::Seconds      => n * 1000 * 1000,
            TimeUnit::Milliseconds => n * 1000,
            TimeUnit::Microseconds => n
        }
    }

    /// Returns the equivalent of `n` microseconds in the given unit.
    ///
    /// As the return value is an integer, there may be some rounding down.
    ///
    /// # Examples
    ///
    /// 1 million microseconds is equivalent to 1 second:
    ///
    /// ```rust
    /// use baps3_cli::time::TimeUnit;
    /// assert_eq!(TimeUnit::Seconds.from_micros(1000000), 1)
    /// ```
    ///
    /// Translating one hour of time to microseconds and back is the identity:
    ///
    /// ```rust
    /// use baps3_cli::time::TimeUnit;
    /// let hour_us = TimeUnit::Hours.as_micros(1);
    /// assert_eq!(TimeUnit::Hours.from_micros(hour_us), 1)
    /// ```
    pub fn from_micros(&self, n: u64) -> u64 {
        match *self {
            TimeUnit::Hours        => n / 1000 / 1000 / 60 / 60,
            TimeUnit::Minutes      => n / 1000 / 1000 / 60,
            TimeUnit::Seconds      => n / 1000 / 1000,
            TimeUnit::Milliseconds => n / 1000,
            TimeUnit::Microseconds => n
        }
    }

    /// Multiplexes a series of unit flags into a TimeUnit.
    /// Larger units take precedence.
    pub fn from_flags(h: bool, m: bool, s: bool, ms: bool) -> TimeUnit {
        if      h  { TimeUnit::Hours        }
        else if m  { TimeUnit::Minutes      }
        else if s  { TimeUnit::Seconds      }
        else if ms { TimeUnit::Milliseconds }
        else       { TimeUnit::Microseconds }
    }
}