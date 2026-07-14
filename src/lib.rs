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
//! | Segment | Length | Encoding |
//! |---------|--------|----------|
//! | Ms      | 12 | Unix timestamp in milliseconds, lowercase hex |
//! | Month   | 1  | `a` (January) .. `l` (December) |
//! | Week    | 2  | ISO week number 01-53 |
//! | Day     | 1  | `m` (Monday) .. `s` (Sunday) |
//!
//! # K-sortability
//!
//! **Fully K-sortable** — lexicographic order matches chronological order
//! across all boundaries: same millisecond, day, month, year, and even the
//! December→January year boundary. No inversions or edge cases.
//!
//! # Feature flags
//!
//! | Feature | Description | Default |
//! |---------|-------------|---------|
//! | `uuid` | UUID v7 interop (`to_uuid_v7`, `from_uuid_v7`) | ✅ on |
//! | `tokio` | Async via `tokio::task::spawn_blocking` | off |
//! | `smol` | Async via `smol::unblock` | off |
//!
//! Features `tokio` and `smol` are mutually exclusive.
//!
//! # UUID v7 interoperability (requires `uuid` feature)
//!
//! Kalid and [UUID v7](https://www.rfc-editor.org/rfc/rfc9562#name-uuid-version-7)
//! share the exact same millisecond timestamp. Week and day are encoded in
//! `rand_a` (12 bits = `[week:6][day:3][random:3]`). Conversion is fully
//! deterministic: `kalid -> UUID v7 -> kalid` produces the exact same string.
//!
//! # Async (requires `tokio` or `smol` feature)
//!
//! When a feature is enabled, CPU-bound work is offloaded to the runtime's
//! blocking pool:
//!
//! ```
//! # #[cfg(feature = "tokio")] {
//! tokio::runtime::Runtime::new().unwrap().block_on(async {
//! let id = kalid::generate_kalid_async().await;
//! let k  = kalid::Kalid::new_async().await;
//! });
//! # }
//! ```
//!
//! # Example
//!
//! ```
//! use kalid::Kalid;
//!
//! let k = Kalid::new();
//! assert_eq!(k.as_string().len(), 16);
//!
//! // Roundtrip: string → parse → string
//! let parsed = Kalid::parse(&k.as_string()).unwrap();
//! assert_eq!(parsed.as_string(), k.as_string());
//!
//! // UUID v7 roundtrip (requires uuid feature)
//! #[cfg(feature = "uuid")] {
//!     let uuid = k.to_uuid_v7();
//!     let back = Kalid::from_uuid_v7(&uuid);
//!     assert_eq!(back.epoch_ms(), k.epoch_ms());
//! }
//! ```

use chrono::{TimeZone, Utc};

/// Month encoding: `a` = January .. `l` = December.
pub const MONTH_CHARS: [char; 12] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l'];

/// Day-of-week encoding: `m` = Monday .. `s` = Sunday.
pub const DAY_CHARS: [char; 7] = ['m', 'n', 'o', 'p', 'q', 'r', 's'];

/// Errors that can occur when parsing a kalid string.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum KalidParseError {
    /// Input length is not 16 characters.
    #[error("kalid must be exactly 16 characters")]
    InvalidLength,
    /// Timestamp segment is not valid 12-digit hex.
    #[error("timestamp must be 12 hex digits")]
    InvalidTimestamp,
    /// Month character not in range `a`..`l`.
    #[error("month must be a..l")]
    InvalidMonth,
    /// Week segment is not a valid two-digit number.
    #[error("week must be a 2-digit number")]
    InvalidWeek,
    /// Day character not in range `m`..`s`.
    #[error("day must be m..s")]
    InvalidDay,
    /// Parsed components don't match the embedded timestamp.
    #[error("kalid components don't match timestamp")]
    Mismatch,
}

/// A calendar-based unique ID with optional UUID v7 interoperability.
///
/// Encodes a Unix millisecond timestamp into a 16-character string.
/// See the [module-level documentation](self) for format details.
///
/// ```
/// use kalid::Kalid;
///
/// let k = Kalid::new();
/// assert_eq!(k.as_string().len(), 16);
/// assert_eq!(Kalid::parse(&k.as_string()).unwrap(), k);
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
    /// ```
    /// use kalid::Kalid;
    /// let k = Kalid::new();
    /// assert_eq!(k.as_string().len(), 16);
    /// ```
    pub fn new() -> Self {
        Kalid {
            epoch_ms: Utc::now().timestamp_millis(),
        }
    }

    /// Create a `Kalid` from a Unix epoch in **seconds**.
    ///
    /// The sub-second fraction is set to zero.
    ///
    /// ```
    /// use kalid::Kalid;
    /// let k = Kalid::from_epoch(1_784_060_036);
    /// assert_eq!(k.epoch_secs(), 1_784_060_036);
    /// ```
    pub fn from_epoch(epoch_secs: i64) -> Self {
        Kalid {
            epoch_ms: epoch_secs * 1000,
        }
    }

    /// Create a `Kalid` from a Unix epoch in **milliseconds**.
    ///
    /// ```
    /// use kalid::Kalid;
    /// let k = Kalid::from_epoch_ms(1_784_060_036_000);
    /// assert_eq!(k.epoch_ms(), 1_784_060_036_000);
    /// ```
    pub fn from_epoch_ms(epoch_ms: i64) -> Self {
        Kalid { epoch_ms }
    }

    /// Parse a 16-character kalid string.
    ///
    /// Every segment is validated and the month/week/day are verified
    /// against the embedded timestamp.
    ///
    /// ```
    /// use kalid::Kalid;
    ///
    /// // epoch 0 ms = Jan 1, 1970 (Thu) → month=a week=01 day=p
    /// let k = Kalid::parse("000000000000a01p").unwrap();
    /// assert_eq!(k.epoch_ms(), 0);
    /// assert_eq!(k.as_string(),   "000000000000a01p");
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
        if s != kalid.as_string() {
            return Err(KalidParseError::Mismatch);
        }
        Ok(kalid)
    }

    /// Create a `Kalid` from a UUID v7. Requires `uuid` feature.
    ///
    /// ```
    /// # #[cfg(feature = "uuid")] {
    /// # use kalid::Kalid;
    /// let uuid = uuid::Uuid::now_v7();
    /// let k = Kalid::from_uuid_v7(&uuid);
    /// assert_eq!(k.as_string().len(), 16);
    /// # }
    /// ```
    #[cfg(feature = "uuid")]
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
    /// Return the kalid as a 16-character string.
    ///
    /// Format: `{ms_hex:012}{month}{week:02}{day}`.
    ///
    /// ```
    /// use kalid::Kalid;
    /// let k = Kalid::from_epoch_ms(0);
    /// assert_eq!(k.as_string(), "000000000000a01p");
    /// ```
    pub fn as_string(&self) -> String {
        format_kalid(self.epoch_ms)
    }

    /// Return the Unix epoch timestamp in seconds.
    ///
    /// Sub-millisecond precision is not available.
    ///
    /// ```
    /// use kalid::Kalid;
    /// let k = Kalid::from_epoch_ms(1_784_060_036_000);
    /// assert_eq!(k.epoch_secs(), 1_784_060_036);
    /// ```
    pub fn epoch_secs(&self) -> i64 {
        self.epoch_ms / 1000
    }

    /// Return the Unix epoch timestamp in milliseconds.
    ///
    /// ```
    /// use kalid::Kalid;
    /// let k = Kalid::from_epoch(1_784_060_036);
    /// assert_eq!(k.epoch_ms(), 1_784_060_036_000);
    /// ```
    pub fn epoch_ms(&self) -> i64 {
        self.epoch_ms
    }

    /// Convert to a UUID v7 with week+day encoded in `rand_a`.
    ///
    /// Requires the `uuid` feature.
    ///
    /// ```
    /// # #[cfg(feature = "uuid")] {
    /// use kalid::Kalid;
    ///
    /// let k = Kalid::new();
    /// let uuid = k.to_uuid_v7();
    /// assert_eq!(uuid.get_version(), Some(uuid::Version::SortRand));
    ///
    /// // Deterministic roundtrip
    /// let back = Kalid::from_uuid_v7(&uuid);
    /// assert_eq!(back.as_string(), k.as_string());
    /// # }
    /// ```
    #[cfg(feature = "uuid")]
    pub fn to_uuid_v7(&self) -> uuid::Uuid {
        let mut bytes = [0u8; 10];
        rand::fill(&mut bytes[..]);
        let secs = self.epoch_ms / 1000;
        let nsecs = ((self.epoch_ms % 1000) * 1_000_000) as u32;
        // INVARIANT: Any `i64` millis maps to a valid UTC datetime within chrono's range.
        let dt = Utc.timestamp_opt(secs, nsecs).unwrap();
        let week = dt.iso_week().week();
        let day = dt.weekday().num_days_from_monday();
        bytes[0] = (bytes[0] & 0xF0) | ((week >> 2) as u8 & 0x0F);
        bytes[1] = (bytes[1] & 0x07) | (((week as u8 & 0x03) << 6) | ((day as u8 & 0x07) << 3));
        uuid::Builder::from_unix_timestamp_millis(self.epoch_ms as u64, &bytes).into_uuid()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

use chrono::Datelike;

fn format_kalid(epoch_ms: i64) -> String {
    let secs = epoch_ms / 1000;
    let nsecs = ((epoch_ms % 1000) * 1_000_000) as u32;
    // INVARIANT: Any `i64` millis maps to a valid UTC datetime within chrono's range.
    let dt = Utc.timestamp_opt(secs, nsecs).unwrap();
    let month = MONTH_CHARS[dt.month0() as usize];
    let week = dt.iso_week().week();
    let day = DAY_CHARS[dt.weekday().num_days_from_monday() as usize];
    format!("{:012x}{month}{week:02}{day}", epoch_ms)
}

/// Generate a kalid string directly.
///
/// Equivalent to `Kalid::new().as_string()`.
///
/// ```
/// use kalid::generate_kalid;
/// let id = generate_kalid();
/// assert_eq!(id.len(), 16);
/// ```
pub fn generate_kalid() -> String {
    Kalid::new().as_string()
}

// ---------------------------------------------------------------------------
// Async support (requires `tokio` or `smol` feature)
// ---------------------------------------------------------------------------

#[cfg(all(feature = "tokio", feature = "smol"))]
compile_error!("features `tokio` and `smol` are mutually exclusive; enable only one.");

/// Offloads CPU-bound work to the async runtime's blocking pool.
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

/// Generate a kalid string asynchronously.
///
/// Requires the `tokio` or `smol` feature.
///
/// ```
/// # #[cfg(feature = "tokio")] {
/// tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let id = kalid::generate_kalid_async().await;
/// assert_eq!(id.len(), 16);
/// });
/// # }
/// ```
#[cfg(any(feature = "tokio", feature = "smol"))]
pub async fn generate_kalid_async() -> String {
    rt::blocking(generate_kalid).await
}

#[cfg(any(feature = "tokio", feature = "smol"))]
impl Kalid {
    /// Create a new `Kalid` asynchronously.
    ///
    /// Offloads the system clock query to the runtime's blocking pool.
    /// Requires the `tokio` or `smol` feature.
    ///
    /// ```
    /// # #[cfg(feature = "tokio")] {
    /// tokio::runtime::Runtime::new().unwrap().block_on(async {
    /// let k = kalid::Kalid::new_async().await;
    /// assert_eq!(k.as_string().len(), 16);
    /// });
    /// # }
    /// ```
    pub async fn new_async() -> Self {
        rt::blocking(Kalid::new).await
    }
}
