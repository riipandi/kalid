// Copyright (c) Aris Ripandi <aris@duck.com>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Kalid: calendar-based, K-sortable unique ID generator with UUID v7
//! interoperability.
//!
//! Kalid encodes a Unix millisecond timestamp into a compact 16-character
//! string:
//!
//! ```text
//! {ms_hex:012}{month:1}{week:02}{day:1}
//! ```
//!
//! # Performance design
//!
//! The hot-path (`new()` + `as_string()`) is optimised to avoid every
//! source of unnecessary latency:
//!
//! - **`std::time::SystemTime`** instead of `chrono::Utc::now()` for
//!   clock acquisition — removes chrono overhead from `new()`.
//! - **`civil_from_days`** — a branchless O(1) Gregorian algorithm
//!   (Howard Hinnant, <https://howardhinnant.github.io/date_algorithms.html>)
//!   replaces `chrono::DateTime` calendar decode in the format path.
//! - **Thread-local calendar cache** keyed on the UTC day number — month,
//!   ISO week, and weekday are re-computed at most once per day per thread.
//! - **Stack-allocated hex encoding** via a 16-entry nibble lookup table —
//!   no `format!()` call, no heap allocation in the inner encode loop.
//! - **`as_str_buf()`** returns a `[u8; 16]` stack buffer so callers can
//!   avoid the `String` heap allocation entirely when they only need a `&str`.
//! - **`#[inline(always)]`** on every hot-path function so the compiler can
//!   see through call boundaries and eliminate redundant work.
//!
//! # K-sortability
//!
//! **Fully K-sortable** — lexicographic order matches chronological order
//! across all boundaries: same millisecond, day, month, year, and even the
//! December→January year boundary.
//!
//! # Feature flags
//!
//! | Feature | Default | Description                                    |
//! |---------|---------|------------------------------------------------|
//! | `uuid`  | on      | UUID v7 interop (`to_uuid_v7`, `from_uuid_v7`) |
//! | `tokio` | off     | Async via `tokio::task::spawn_blocking`        |
//! | `smol`  | off     | Async via `smol::unblock`                      |
//!
//! `tokio` and `smol` are mutually exclusive.
//!
//! # UUID v7 interoperability (requires `uuid` feature)
//!
//! Kalid and [UUID v7](https://www.rfc-editor.org/rfc/rfc9562) share the
//! exact same ms timestamp. Week+day are encoded in `rand_a` (12 bits).
//! Conversion is fully deterministic: `kalid -> UUID v7 -> kalid` produces
//! the exact same string.
//!
//! # Prefix (optional)
//!
//! Use [`KalidBuilder`] to add a prefix and separator:
//!
//! ```
//! let id = kalid::Kalid::builder().prefix("order").build();
//! assert!(id.starts_with("order_"));
//! ```
//!
//! # Zero-allocation output
//!
//! [`Kalid::as_str_buf`] returns a `[u8; 16]` without touching the heap.
//! Use it when you need a `&str` slice inside a hot loop:
//!
//! ```
//! use kalid::Kalid;
//! let k = Kalid::from_epoch_ms(1_784_060_036_000);
//! let buf = k.as_str_buf();
//! let s: &str = std::str::from_utf8(&buf).unwrap();
//! assert_eq!(s.len(), 16);
//! ```
//!
//! # Async (requires `tokio` or `smol` feature)
//!
//! ```
//! # #[cfg(feature = "tokio")] {
//! tokio::runtime::Runtime::new().unwrap().block_on(async {
//!     let id = kalid::generate_kalid_async().await;
//!     let k  = kalid::Kalid::new_async().await;
//! });
//! # }
//! ```
//!
//! # Example
//!
//! ```
//! use kalid::Kalid;
//! let k = Kalid::new();
//! assert_eq!(k.as_string().len(), 16);
//! let parsed = Kalid::parse(&k.as_string()).unwrap();
//! assert_eq!(parsed.as_string(), k.as_string());
//!
//! #[cfg(feature = "uuid")] {
//! let uuid = k.to_uuid_v7();
//! let back = Kalid::from_uuid_v7(&uuid);
//! assert_eq!(back.epoch_ms(), k.epoch_ms());
//! }
//! ```

/// Month encoding: `a` = January .. `l` = December.
pub const MONTH_CHARS: [char; 12] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l'];

/// Day-of-week encoding: `m` = Monday .. `s` = Sunday.
pub const DAY_CHARS: [char; 7] = ['m', 'n', 'o', 'p', 'q', 'r', 's'];

// ---------------------------------------------------------------------------
// Internal constants
// ---------------------------------------------------------------------------

/// ASCII hex digits for nibble-at-a-time encoding (no `format!`).
const HEX: &[u8; 16] = b"0123456789abcdef";

/// Byte equivalents of MONTH_CHARS for the hot encode path.
const MONTH_BYTES: [u8; 12] = *b"abcdefghijkl";

/// Byte equivalents of DAY_CHARS for the hot encode path.
const DAY_BYTES: [u8; 7] = *b"mnopqrs";

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors that can occur when parsing a kalid string.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum KalidParseError {
    #[error("kalid must be exactly 16 characters")]
    InvalidLength,
    #[error("timestamp must be 12 hex digits")]
    InvalidTimestamp,
    #[error("month must be a..l")]
    InvalidMonth,
    #[error("week must be a 2-digit number")]
    InvalidWeek,
    #[error("day must be m..s")]
    InvalidDay,
    #[error("kalid components don't match timestamp")]
    Mismatch,
}

// ---------------------------------------------------------------------------
// Kalid
// ---------------------------------------------------------------------------

/// A calendar-based unique ID with optional UUID v7 interoperability.
///
/// ```
/// use kalid::Kalid;
/// let k = Kalid::new();
/// assert_eq!(k.as_string().len(), 16);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kalid {
    epoch_ms: i64,
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

impl Kalid {
    /// Create a new `Kalid` from the current system time.
    ///
    /// Uses `std::time::SystemTime` directly to avoid chrono overhead on the
    /// clock-acquisition path.
    ///
    /// ```
    /// use kalid::Kalid;
    /// let k = Kalid::new();
    /// assert_eq!(k.as_string().len(), 16);
    /// ```
    #[inline(always)]
    pub fn new() -> Self {
        Kalid { epoch_ms: now_ms() }
    }

    /// Create a `Kalid` from a Unix epoch in seconds (fraction set to zero).
    ///
    /// ```
    /// use kalid::Kalid;
    /// let k = Kalid::from_epoch(1_784_060_036);
    /// assert_eq!(k.epoch_secs(), 1_784_060_036);
    /// ```
    #[inline(always)]
    pub fn from_epoch(epoch_secs: i64) -> Self {
        Kalid {
            epoch_ms: epoch_secs * 1000,
        }
    }

    /// Create a `Kalid` from a Unix epoch in milliseconds.
    ///
    /// This is a zero-cost constructor — it stores `epoch_ms` directly.
    ///
    /// ```
    /// use kalid::Kalid;
    /// let k = Kalid::from_epoch_ms(1_784_060_036_000);
    /// assert_eq!(k.epoch_ms(), 1_784_060_036_000);
    /// ```
    #[inline(always)]
    pub fn from_epoch_ms(epoch_ms: i64) -> Self {
        Kalid { epoch_ms }
    }

    /// Parse a 16-character kalid string.
    ///
    /// Validates each field in the format `{ms_hex:012}{month}{week:02}{day}`
    /// and checks that the components are consistent with the timestamp.
    ///
    /// ```
    /// use kalid::Kalid;
    /// let k = Kalid::parse("000000000000a01p").unwrap();
    /// assert_eq!(k.epoch_ms(), 0);
    /// ```
    pub fn parse(s: &str) -> Result<Self, KalidParseError> {
        if s.len() != 16 {
            return Err(KalidParseError::InvalidLength);
        }
        let epoch_ms = i64::from_str_radix(&s[..12], 16).map_err(|_| KalidParseError::InvalidTimestamp)?;
        let month_char = s.as_bytes()[12];
        if !(b'a'..=b'l').contains(&month_char) {
            return Err(KalidParseError::InvalidMonth);
        }
        if !s[13..15].bytes().all(|b| b.is_ascii_digit()) {
            return Err(KalidParseError::InvalidWeek);
        }
        let day_char = s.as_bytes()[15];
        if !(b'm'..=b's').contains(&day_char) {
            return Err(KalidParseError::InvalidDay);
        }
        let kalid = Kalid { epoch_ms };
        // Verify that the calendar fields in `s` actually match epoch_ms.
        if s.as_bytes() != kalid.as_str_buf() {
            return Err(KalidParseError::Mismatch);
        }
        Ok(kalid)
    }

    /// Create a `Kalid` from a UUID v7. Requires `uuid` feature.
    ///
    /// ```
    /// # #[cfg(feature = "uuid")] {
    /// use kalid::Kalid;
    /// let uuid = uuid::Uuid::now_v7();
    /// let k = Kalid::from_uuid_v7(&uuid);
    /// assert_eq!(k.as_string().len(), 16);
    /// # }
    /// ```
    #[cfg(feature = "uuid")]
    #[inline(always)]
    pub fn from_uuid_v7(uuid: &uuid::Uuid) -> Self {
        let bytes = uuid.as_bytes();
        let epoch_ms = u64::from_be_bytes([0, 0, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]]) as i64;
        Kalid { epoch_ms }
    }
}

impl Default for Kalid {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

impl Kalid {
    /// Return the kalid as a stack-allocated `[u8; 16]` byte buffer.
    ///
    /// This is the **zero-allocation** form. All bytes are valid ASCII so the
    /// result can be converted to `&str` without any UTF-8 validation:
    ///
    /// ```
    /// use kalid::Kalid;
    /// let k = Kalid::from_epoch_ms(0);
    /// let buf = k.as_str_buf();
    /// assert_eq!(&buf, b"000000000000a01p");
    /// let s: &str = std::str::from_utf8(&buf).unwrap();
    /// assert_eq!(s, "000000000000a01p");
    /// ```
    #[inline(always)]
    pub fn as_str_buf(&self) -> [u8; 16] {
        encode_kalid(self.epoch_ms)
    }

    /// Return the kalid as a 16-character heap-allocated [`String`].
    ///
    /// Internally calls [`as_str_buf`](Kalid::as_str_buf) and converts the
    /// result — prefer `as_str_buf` in hot loops to avoid heap allocation.
    ///
    /// Format: `{ms_hex:012}{month}{week:02}{day}`.
    ///
    /// ```
    /// use kalid::Kalid;
    /// let k = Kalid::from_epoch_ms(0);
    /// assert_eq!(k.as_string(), "000000000000a01p");
    /// ```
    #[inline(always)]
    pub fn as_string(&self) -> String {
        // SAFETY: encode_kalid writes only ASCII bytes (hex + a-s range).
        unsafe { String::from_utf8_unchecked(self.as_str_buf().to_vec()) }
    }

    /// Return the Unix epoch in seconds (sub-ms precision not available).
    ///
    /// ```
    /// use kalid::Kalid;
    /// let k = Kalid::from_epoch_ms(1_784_060_036_000);
    /// assert_eq!(k.epoch_secs(), 1_784_060_036);
    /// ```
    #[inline(always)]
    pub fn epoch_secs(&self) -> i64 {
        self.epoch_ms / 1000
    }

    /// Return the Unix epoch in milliseconds.
    ///
    /// ```
    /// use kalid::Kalid;
    /// let k = Kalid::from_epoch(1_784_060_036);
    /// assert_eq!(k.epoch_ms(), 1_784_060_036_000);
    /// ```
    #[inline(always)]
    pub fn epoch_ms(&self) -> i64 {
        self.epoch_ms
    }

    /// Convert to UUID v7 with week+day in `rand_a`. Requires `uuid` feature.
    ///
    /// ```
    /// # #[cfg(feature = "uuid")] {
    /// use kalid::Kalid;
    /// let k = Kalid::new();
    /// let uuid = k.to_uuid_v7();
    /// assert_eq!(uuid.get_version(), Some(uuid::Version::SortRand));
    /// let back = Kalid::from_uuid_v7(&uuid);
    /// assert_eq!(back.as_string(), k.as_string());
    /// # }
    /// ```
    #[cfg(feature = "uuid")]
    pub fn to_uuid_v7(&self) -> uuid::Uuid {
        use chrono::{Datelike, TimeZone, Utc};
        let mut bytes = [0u8; 10];
        rand::fill(&mut bytes[..]);
        let secs = self.epoch_ms / 1000;
        let nsecs = ((self.epoch_ms % 1000) * 1_000_000) as u32;
        // INVARIANT: Any `i64` millis maps to a valid UTC datetime.
        let dt = Utc.timestamp_opt(secs, nsecs).unwrap();
        let week = dt.iso_week().week();
        let day = dt.weekday().num_days_from_monday();
        bytes[0] = (bytes[0] & 0xF0) | ((week >> 2) as u8 & 0x0F);
        bytes[1] = (bytes[1] & 0x07) | (((week as u8 & 0x03) << 6) | ((day as u8 & 0x07) << 3));
        uuid::Builder::from_unix_timestamp_millis(self.epoch_ms as u64, &bytes).into_uuid()
    }

    /// Create a [`KalidBuilder`] for configuring prefix and separator.
    pub fn builder() -> KalidBuilder {
        KalidBuilder::new()
    }
}

// ---------------------------------------------------------------------------
// Clock acquisition (OPT-4)
// ---------------------------------------------------------------------------

/// Return the current Unix time in milliseconds.
///
/// Uses `std::time::SystemTime` directly to skip the chrono calendar decode
/// overhead present in `Utc::now().timestamp_millis()`.
#[inline(always)]
fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        // SAFETY: system clock is always after the Unix epoch on supported
        // platforms. If it isn't, a panic here is the correct behaviour.
        .expect("system clock before Unix epoch")
        .as_millis() as i64
}

// ---------------------------------------------------------------------------
// Branchless Gregorian calendar decode (OPT-1)
// ---------------------------------------------------------------------------

/// Decompose a Unix day number into `(year, month1, day1)` using Howard
/// Hinnant's *civil_from_days* algorithm — O(1), branch-free, no lookup
/// tables.
///
/// Reference: <https://howardhinnant.github.io/date_algorithms.html>
#[inline(always)]
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era: i64 = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u32; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y } as i32;
    (y, m, d)
}

/// Compute the ISO week number (1–53) and ISO weekday (0 = Mon .. 6 = Sun)
/// from a Unix day number.
///
/// Algorithm follows ISO 8601: week 1 is the week containing the first
/// Thursday of the year.
#[inline(always)]
fn iso_week_and_day(unix_day: i64) -> (u32, u32) {
    // Day-of-week: Unix epoch (1970-01-01) is a Thursday (ISO dow = 3).
    // We want Mon=0 .. Sun=6.
    let dow = ((unix_day + 3).rem_euclid(7)) as u32; // 0=Mon .. 6=Sun
    // ISO week: shift to nearest Thursday then divide.
    let thursday_day = unix_day + (3 - dow as i64);
    let (thu_y, _, _) = civil_from_days(thursday_day);
    // First day of ISO year: Monday of the week containing Jan 4.
    let jan4 = days_from_civil(thu_y, 1, 4);
    let jan4_dow = (jan4 + 3).rem_euclid(7); // 0=Mon
    let week1_mon = jan4 - jan4_dow;
    let week = ((thursday_day - week1_mon) / 7 + 1) as u32;
    (week, dow)
}

/// Inverse of `civil_from_days`: return the Unix day number for a given date.
#[inline(always)]
fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y as i64 - 1 } else { y as i64 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe as i64 - 719_468
}

// ---------------------------------------------------------------------------
// Thread-local calendar cache (OPT-3)
// ---------------------------------------------------------------------------

/// Cached calendar fields for a single UTC day, keyed by the Unix day number.
///
/// The cache is per-thread so there is zero contention and no atomic ops.
/// Month, ISO week, and weekday are recomputed at most once per day per thread.
#[derive(Clone, Copy)]
struct DayCache {
    /// UTC day number this cache entry is valid for.
    unix_day: i64,
    /// 0-indexed month (0 = January ... 11 = December).
    month0: u8,
    /// ISO 8601 week number (1-53).
    week: u8,
    /// Weekday (0 = Monday ... 6 = Sunday).
    weekday: u8,
}

impl DayCache {
    const INVALID: Self = DayCache {
        unix_day: i64::MIN,
        month0: 0,
        week: 0,
        weekday: 0,
    };
}

std::thread_local! {
    static CACHE: std::cell::Cell<DayCache> = const { std::cell::Cell::new(DayCache::INVALID) };
}

/// Return `(month0, iso_week, weekday)` for the given millisecond timestamp.
///
/// Hits the thread-local cache on the common path (same day as last call),
/// otherwise recomputes via branchless Gregorian arithmetic.
#[inline(always)]
fn calendar_fields(epoch_ms: i64) -> (u8, u8, u8) {
    let unix_day = epoch_ms.div_euclid(86_400_000);
    CACHE.with(|cell| {
        let cached = cell.get();
        if cached.unix_day == unix_day {
            return (cached.month0, cached.week, cached.weekday);
        }
        let (_, m, _) = civil_from_days(unix_day);
        let (week, dow) = iso_week_and_day(unix_day);
        // Clamp week to [1,53] — should never exceed 53 for valid dates.
        let week = week.min(53) as u8;
        let entry = DayCache {
            unix_day,
            month0: (m - 1) as u8,
            week,
            weekday: dow as u8,
        };
        cell.set(entry);
        (entry.month0, entry.week, entry.weekday)
    })
}

// ---------------------------------------------------------------------------
// Core encode (OPT-2 + OPT-5)
// ---------------------------------------------------------------------------

/// Encode `epoch_ms` into a 16-byte stack buffer.
///
/// - Bytes `[0..12]`: lowercase hex of the timestamp (nibble lookup table,
///   no heap allocation, no `format!`).
/// - Byte `[12]`: month character `a`-`l`.
/// - Bytes `[13..15]`: zero-padded decimal ISO week number.
/// - Byte `[15]`: weekday character `m`-`s`.
///
/// All bytes are valid ASCII, so the buffer can be transmuted to `&str`
/// without UTF-8 validation.
#[inline(always)]
fn encode_kalid(epoch_ms: i64) -> [u8; 16] {
    let mut buf = [0u8; 16];

    // --- Timestamp: 12 lowercase hex digits (OPT-5) ---
    let mut v = epoch_ms as u64;
    // Write right-to-left so the MSB ends up at index 0.
    for i in (0..12).rev() {
        buf[i] = HEX[(v & 0xF) as usize];
        v >>= 4;
    }

    // --- Calendar fields via cache (OPT-1 + OPT-3) ---
    let (month0, week, weekday) = calendar_fields(epoch_ms);

    buf[12] = MONTH_BYTES[month0 as usize];

    // Zero-padded 2-digit decimal week (week in 1..=53).
    buf[13] = b'0' + week / 10;
    buf[14] = b'0' + week % 10;

    buf[15] = DAY_BYTES[weekday as usize];

    buf
}

// ---------------------------------------------------------------------------
// Public convenience function
// ---------------------------------------------------------------------------

/// Generate a kalid string directly (equivalent to `Kalid::new().as_string()`).
///
/// ```
/// use kalid::generate_kalid;
/// let id = generate_kalid();
/// assert_eq!(id.len(), 16);
/// ```
#[inline(always)]
pub fn generate_kalid() -> String {
    Kalid::new().as_string()
}

// ---------------------------------------------------------------------------
// Builder (prefix + separator)
// ---------------------------------------------------------------------------

/// Configurable builder for kalid strings with an optional prefix.
///
/// Default separator is `_` (URL-safe). Use [`no_separator`](KalidBuilder::no_separator)
/// to remove it, or [`separator`](KalidBuilder::separator) to customise.
///
/// ```
/// use kalid::Kalid;
/// let id = Kalid::builder().prefix("order").build();
/// assert!(id.starts_with("order_"));
/// assert_eq!(id.len(), 22);
/// ```
#[derive(Debug, Clone)]
pub struct KalidBuilder {
    prefix: Option<String>,
    separator: Option<char>,
}

impl KalidBuilder {
    fn new() -> Self {
        KalidBuilder {
            prefix: None,
            separator: Some('_'),
        }
    }

    /// Set the prefix string.
    pub fn prefix(mut self, s: &str) -> Self {
        self.prefix = Some(s.to_string());
        self
    }

    /// Set a custom separator (default `_`). Recommended: URL-safe chars `- . _ ~`.
    pub fn separator(mut self, c: char) -> Self {
        self.separator = Some(c);
        self
    }

    /// Remove separator — prefix is prepended directly.
    pub fn no_separator(mut self) -> Self {
        self.separator = None;
        self
    }

    /// Generate a kalid from the current system time with this config.
    pub fn build(&self) -> String {
        self.format(&Kalid::new())
    }

    /// Format an existing `Kalid` with this config.
    pub fn build_from(&self, kalid: &Kalid) -> String {
        self.format(kalid)
    }

    fn format(&self, kalid: &Kalid) -> String {
        let base = kalid.as_string();
        match (&self.prefix, &self.separator) {
            (Some(p), Some(c)) => format!("{}{}{}", p, c, base),
            (Some(p), None) => format!("{}{}", p, base),
            (None, _) => base,
        }
    }
}

impl Default for KalidBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Async support (requires `tokio` or `smol` feature)
// ---------------------------------------------------------------------------

#[cfg(all(feature = "tokio", feature = "smol"))]
compile_error!("features `tokio` and `smol` are mutually exclusive; enable only one.");

#[cfg(any(feature = "tokio", feature = "smol"))]
mod rt {
    pub(crate) async fn blocking<F, T>(f: F) -> T
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        #[cfg(feature = "tokio")]
        {
            tokio::task::spawn_blocking(f).await.expect("blocking task panicked")
        }
        #[cfg(feature = "smol")]
        {
            smol::unblock(f).await
        }
    }
}

/// Generate a kalid asynchronously. Requires `tokio` or `smol` feature.
#[cfg(any(feature = "tokio", feature = "smol"))]
pub async fn generate_kalid_async() -> String {
    rt::blocking(generate_kalid).await
}

#[cfg(any(feature = "tokio", feature = "smol"))]
impl Kalid {
    /// Create a new `Kalid` asynchronously. Requires `tokio` or `smol` feature.
    pub async fn new_async() -> Self {
        rt::blocking(Kalid::new).await
    }
}

// ---------------------------------------------------------------------------
// Internal unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- civil_from_days / days_from_civil ---

    #[test]
    fn civil_epoch_zero_is_jan_1_1970() {
        let (y, m, d) = civil_from_days(0);
        assert_eq!((y, m, d), (1970, 1, 1));
    }

    #[test]
    fn civil_known_dates() {
        // 2026-07-08 = Unix day 20_642
        let (y, m, d) = civil_from_days(20_642);
        assert_eq!((y, m, d), (2026, 7, 8));
        // Round-trip
        assert_eq!(days_from_civil(y, m, d), 20_642);
    }

    #[test]
    fn civil_leap_day_2024() {
        // 2024-02-29
        let day = days_from_civil(2024, 2, 29);
        let (y, m, d) = civil_from_days(day);
        assert_eq!((y, m, d), (2024, 2, 29));
    }

    // --- iso_week_and_day ---

    #[test]
    fn iso_week_epoch_zero() {
        // 1970-01-01 is a Thursday (ISO dow = 3), week 1.
        let (week, dow) = iso_week_and_day(0);
        assert_eq!(week, 1);
        assert_eq!(dow, 3); // Thursday
    }

    #[test]
    fn iso_week_all_days_of_week_2026_w01() {
        // 2026-W01-1 is 2025-12-29 (Mon) through 2026-01-04 (Sun).
        let monday = days_from_civil(2025, 12, 29);
        for offset in 0i64..7 {
            let (w, dow) = iso_week_and_day(monday + offset);
            assert_eq!(w, 1, "not week 1 at offset {offset}");
            assert_eq!(dow, offset as u32, "dow mismatch at offset {offset}");
        }
    }

    // --- calendar_fields ---

    #[test]
    fn calendar_fields_epoch_zero_is_thursday_jan_week1() {
        // epoch 0 ms = Thu Jan 1 1970, ISO week 1
        let (month0, week, weekday) = calendar_fields(0);
        assert_eq!(month0, 0); // January
        assert_eq!(week, 1); // ISO week 1
        assert_eq!(weekday, 3); // Thursday
    }

    #[test]
    fn calendar_cache_same_day_hits() {
        // Two timestamps in the same UTC day must return identical fields.
        let base = 1_784_060_036_000i64; // 2026-07-08 ~11:13:56 UTC
        let (m0, w0, wd0) = calendar_fields(base);
        let (m1, w1, wd1) = calendar_fields(base + 999); // same day, ~1 second later
        assert_eq!((m0, w0, wd0), (m1, w1, wd1));
    }

    // --- encode_kalid ---

    #[test]
    fn encode_known_epoch_zero() {
        // epoch 0 ms -> Thu 1970-01-01, month=a (Jan), week=01, day=p (Thu)
        let buf = encode_kalid(0);
        assert_eq!(&buf, b"000000000000a01p");
    }

    #[test]
    fn encode_timestamp_hex_is_lowercase() {
        let buf = encode_kalid(0x019f62686310);
        // First 12 bytes must be lowercase hex
        for &b in &buf[..12] {
            assert!(b.is_ascii_digit() || (b'a'..=b'f').contains(&b), "not lowercase hex: {b}");
        }
    }

    #[test]
    fn encode_week_digits_are_decimal() {
        let buf = encode_kalid(1_784_060_036_000);
        assert!(buf[13].is_ascii_digit() && buf[14].is_ascii_digit());
    }

    // --- as_str_buf ---

    #[test]
    fn as_str_buf_matches_as_string() {
        for ms in [0i64, 1_000_000, 1_700_000_000_000, 1_784_060_036_000] {
            let k = Kalid::from_epoch_ms(ms);
            let buf = k.as_str_buf();
            let s = k.as_string();
            assert_eq!(buf.as_slice(), s.as_bytes(), "mismatch at ms={ms}");
        }
    }

    #[test]
    fn as_str_buf_is_valid_utf8() {
        let k = Kalid::from_epoch_ms(1_784_060_036_000);
        let buf = k.as_str_buf();
        assert!(std::str::from_utf8(&buf).is_ok());
    }

    // --- now_ms ---

    #[test]
    fn now_ms_is_reasonable() {
        let ms = now_ms();
        // Must be after 2020-01-01 and before 2100-01-01
        assert!(ms > 1_577_836_800_000);
        assert!(ms < 4_102_444_800_000);
    }

    // --- specification example ---

    /// Locks the exact example from the spec document:
    ///
    /// ```text
    /// UUID v7 : 019f6315-16e6-74b2-b49b-2d3b66ee06ba
    /// Kalid   : 019f631516e6g29o
    /// ```
    ///
    /// Timestamp `0x019f631516e6` = 1784073754342 ms
    /// = 2026-07-15T00:02:34.342 UTC (Wednesday, ISO week 29, July).
    ///
    /// Format breakdown:
    ///   `019f631516e6`  ← 12-char hex epoch_ms
    ///   `g`             ← July  (month index 6, 'a'=Jan)
    ///   `29`            ← ISO week 29 (zero-padded)
    ///   `o`             ← Wednesday (day index 2, 'm'=Mon)
    #[test]
    fn spec_example_from_uuid_v7() {
        // The spec example timestamp is the 48-bit ms field from UUID v7:
        //   019f6315-16e6-74b2-... → bytes [01,9f,63,15,16,e6] → 0x019f631516e6
        let epoch_ms: i64 = 0x019f631516e6;
        assert_eq!(epoch_ms, 1_784_073_754_342);

        let k = Kalid::from_epoch_ms(epoch_ms);
        let s = k.as_string();

        // Full string must match the spec example exactly.
        assert_eq!(s, "019f631516e6g29o", "kalid string does not match spec example");

        // Field-by-field breakdown.
        assert_eq!(&s[..12], "019f631516e6", "timestamp hex mismatch");
        assert_eq!(s.as_bytes()[12], b'g', "month: expected 'g' (July)");
        assert_eq!(&s[13..15], "29", "ISO week mismatch");
        assert_eq!(s.as_bytes()[15], b'o', "day: expected 'o' (Wednesday)");

        // as_str_buf must return the same bytes.
        assert_eq!(&k.as_str_buf(), b"019f631516e6g29o");
    }
}
