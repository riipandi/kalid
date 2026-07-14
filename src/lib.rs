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
//! # Async (requires `tokio` or `smol` feature)
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

use chrono::{TimeZone, Utc};

/// Month encoding: `a` = January .. `l` = December.
pub const MONTH_CHARS: [char; 12] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l'];

/// Day-of-week encoding: `m` = Monday .. `s` = Sunday.
pub const DAY_CHARS: [char; 7] = ['m', 'n', 'o', 'p', 'q', 'r', 's'];

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

    /// Create a `Kalid` from a Unix epoch in seconds (fraction set to zero).
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

    /// Create a `Kalid` from a Unix epoch in milliseconds.
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
        if s != kalid.as_string() {
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

    /// Return the Unix epoch in seconds (sub-ms not available).
    ///
    /// ```
    /// use kalid::Kalid;
    /// let k = Kalid::from_epoch_ms(1_784_060_036_000);
    /// assert_eq!(k.epoch_secs(), 1_784_060_036);
    /// ```
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
// Helpers
// ---------------------------------------------------------------------------

use chrono::Datelike;

fn format_kalid(epoch_ms: i64) -> String {
    let secs = epoch_ms / 1000;
    let nsecs = ((epoch_ms % 1000) * 1_000_000) as u32;
    // INVARIANT: Any `i64` millis maps to a valid UTC datetime.
    let dt = Utc.timestamp_opt(secs, nsecs).unwrap();
    let month = MONTH_CHARS[dt.month0() as usize];
    let week = dt.iso_week().week();
    let day = DAY_CHARS[dt.weekday().num_days_from_monday() as usize];
    format!("{:012x}{month}{week:02}{day}", epoch_ms)
}

/// Generate a kalid string directly (`Kalid::new().as_string()`).
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
