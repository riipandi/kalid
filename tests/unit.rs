// Integration tests for the public Kalid API.
//
// These tests exercise only the public surface (`Kalid` and its `pub` methods)
// and therefore live here as integration tests rather than as inline unit tests
// (which are reserved for the private calendar/encoding helpers in `src/lib.rs`).

use kalid::Kalid;

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
