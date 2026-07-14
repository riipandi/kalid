use chrono::{TimeZone, Utc};
use kalid::Kalid;

#[test]
fn roundtrip_random_epochs() {
    let epochs = [
        0i64,
        999,
        1_000_000,
        1_000_000_000_000,
        1_234_567_890_123,
        1_536_000_000_000,
        1_700_000_000_000,
        1_784_060_036_000,
    ];

    for &ms in &epochs {
        let k = Kalid::from_epoch_ms(ms);
        let s = k.as_string();
        let parsed = Kalid::parse(&s).unwrap();
        assert_eq!(parsed, k, "roundtrip failed for epoch_ms={ms}");
        assert_eq!(parsed.as_string(), s, "string mismatch for epoch_ms={ms}");
    }
}

#[test]
fn roundtrip_leap_year_boundaries() {
    let cases = [
        (2023, 2, 28, 23, 59, 59, 999),
        (2023, 3, 1, 0, 0, 0, 0),
        (2024, 2, 28, 23, 59, 59, 999),
        (2024, 2, 29, 0, 0, 0, 0),
        (2024, 3, 1, 0, 0, 0, 0),
        (2025, 2, 28, 23, 59, 59, 999),
        (2025, 3, 1, 0, 0, 0, 0),
    ];

    for &(y, mo, d, h, mi, s, ms) in &cases {
        let dt = Utc.with_ymd_and_hms(y, mo, d, h, mi, s).unwrap();
        let epoch = dt.timestamp() * 1000 + ms;
        let k = Kalid::from_epoch_ms(epoch);
        assert_eq!(Kalid::parse(&k.as_string()).unwrap(), k);
    }
}

#[test]
fn roundtrip_all_days_of_week() {
    let start = Utc.with_ymd_and_hms(2026, 1, 5, 12, 0, 0).unwrap(); // Monday
    for day_offset in 0..7 {
        let ms = (start.timestamp() + day_offset * 86_400) * 1000;
        let k = Kalid::from_epoch_ms(ms);
        assert_eq!(Kalid::parse(&k.as_string()).unwrap(), k);
    }
}

#[test]
fn format_regex_pattern() {
    for ms in [0i64, 1_000_000, 1_700_000_000_000, 1_784_060_036_000] {
        let k = Kalid::from_epoch_ms(ms);
        let s = k.as_string();
        assert_eq!(s.len(), 16, "length != 16 for {ms}");
        assert!(s[..12].bytes().all(|b| b.is_ascii_hexdigit()), "timestamp not hex: {s}");
        assert!((b'a'..=b'l').contains(&s.as_bytes()[12]), "month out of range: {s}");
        assert!(s[13..15].bytes().all(|b| b.is_ascii_digit()), "week not digit: {s}");
        assert!((b'm'..=b's').contains(&s.as_bytes()[15]), "day out of range: {s}");
    }
}

#[test]
fn all_months_covered() {
    let mut chars = std::collections::HashSet::new();
    for month in 1..=12 {
        let dt = Utc.with_ymd_and_hms(2026, month, 15, 12, 0, 0).unwrap();
        let ms = dt.timestamp() * 1000;
        let s = Kalid::from_epoch_ms(ms).as_string();
        let m = s.as_bytes()[12];
        assert!(chars.insert(m), "duplicate month char {m} for month {month}");
    }
    assert_eq!(chars.len(), 12);
}

#[test]
fn all_days_covered() {
    let mut chars = std::collections::HashSet::new();
    let start = Utc.with_ymd_and_hms(2026, 1, 5, 12, 0, 0).unwrap(); // Monday
    for day_offset in 0..7 {
        let ms = (start.timestamp() + day_offset * 86_400) * 1000;
        let s = Kalid::from_epoch_ms(ms).as_string();
        let d = s.as_bytes()[15];
        assert!(chars.insert(d), "duplicate day char {d}");
    }
    assert_eq!(chars.len(), 7);
}

#[test]
fn trait_bounds() {
    fn assert_traits<T: std::fmt::Debug + Clone + PartialEq + Eq + Send + Sync>() {}
    assert_traits::<Kalid>();
    assert_traits::<kalid::KalidParseError>();
}
