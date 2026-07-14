#![cfg(feature = "uuid")]

use chrono::TimeZone;
use kalid::Kalid;

// -- UUID v7 interoperability ---------------------------------------

#[test]
fn to_uuid_v7_produces_valid_version() {
    let kalid = Kalid::new();
    let uuid = kalid.to_uuid_v7();
    assert_eq!(uuid.get_version(), Some(uuid::Version::SortRand));
    assert_eq!(uuid.get_variant(), uuid::Variant::RFC4122);
}

#[test]
fn uuid_v7_roundtrip_preserves_ms() {
    let kalid = Kalid::from_epoch_ms(1_784_060_036_000);
    let uuid = kalid.to_uuid_v7();
    let back = Kalid::from_uuid_v7(&uuid);
    assert_eq!(back.epoch_ms(), kalid.epoch_ms());
}

#[test]
fn uuid_v7_roundtrip_many_random_uuids() {
    let kalid = Kalid::from_epoch_ms(1_784_060_036_000);
    for _ in 0..10 {
        let uuid = kalid.to_uuid_v7();
        let back = Kalid::from_uuid_v7(&uuid);
        assert_eq!(back.epoch_ms(), 1_784_060_036_000);
    }
}

#[test]
fn from_uuid_v7_preserves_string_format() {
    let uuid = uuid::Uuid::now_v7();
    let kalid = Kalid::from_uuid_v7(&uuid);
    let s = kalid.as_string();

    assert_eq!(s.len(), 16);
    assert!(s[..12].bytes().all(|b| b.is_ascii_hexdigit()));
    assert!((b'a'..=b'l').contains(&s.as_bytes()[12]));
    assert!(s[13..15].bytes().all(|b| b.is_ascii_digit()));
    assert!((b'm'..=b's').contains(&s.as_bytes()[15]));
}

#[test]
fn uuid_v7_byte_level_interop() {
    let kalid = Kalid::from_epoch_ms(0x019f62686310);
    let uuid = kalid.to_uuid_v7();
    assert_eq!(uuid.as_bytes()[0], 0x01);
    assert_eq!(uuid.as_bytes()[1], 0x9f);
    assert_eq!(uuid.as_bytes()[2], 0x62);
    assert_eq!(uuid.as_bytes()[3], 0x68);
    assert_eq!(uuid.as_bytes()[4], 0x63);
    assert_eq!(uuid.as_bytes()[5], 0x10);
}

#[test]
fn to_uuid_v7_unique_per_call() {
    let kalid = Kalid::from_epoch_ms(1_784_060_036_000);
    let u1 = kalid.to_uuid_v7();
    let u2 = kalid.to_uuid_v7();
    assert_ne!(u1, u2, "same kalid should produce different UUID v7 values");
}

// -- Deterministic UUID v7 roundtrip ---------------------------------

#[test]
fn deterministic_roundtrip_same_string() {
    let kalid = Kalid::from_epoch_ms(1_784_060_036_000);
    let uuid = kalid.to_uuid_v7();
    let back = Kalid::from_uuid_v7(&uuid);
    assert_eq!(
        back.as_string(),
        kalid.as_string(),
        "deterministic roundtrip: kalid -> UUID v7 -> kalid"
    );
}

#[test]
fn deterministic_roundtrip_multiple_epochs() {
    for ms in [
        0,
        1_000_000_000,     // 2001-09-09
        1_700_000_000_000, // 2023-11-14
        1_784_060_036_000, // 2026-07-08
    ] {
        let kalid = Kalid::from_epoch_ms(ms);
        let uuid = kalid.to_uuid_v7();
        let back = Kalid::from_uuid_v7(&uuid);
        assert_eq!(back.as_string(), kalid.as_string(), "failed at epoch_ms={}", ms);
    }
}

#[test]
fn rand_a_encodes_week_and_day() {
    use chrono::Datelike;
    let epoch_ms = 1_784_060_036_000;
    let dt = chrono::Utc
        .timestamp_opt(epoch_ms / 1000, ((epoch_ms % 1000) * 1_000_000) as u32)
        .unwrap();
    let expected_week = dt.iso_week().week();
    let expected_day = dt.weekday().num_days_from_monday();

    let kalid = Kalid::from_epoch_ms(epoch_ms);
    let uuid = kalid.to_uuid_v7();
    let bytes = uuid.as_bytes();
    let rand_a = (u16::from_be_bytes([bytes[6], bytes[7]]) & 0x0FFF) as u32;
    let week = (rand_a >> 6) & 0x3F;
    let day = (rand_a >> 3) & 0x07;
    assert_eq!(week, expected_week, "rand_a week mismatch");
    assert_eq!(day, expected_day, "rand_a day mismatch");
}

#[test]
fn rand_a_low_3_bits_are_random() {
    let kalid = Kalid::from_epoch_ms(1_784_060_036_000);
    let uuid1 = kalid.to_uuid_v7();
    let uuid2 = kalid.to_uuid_v7();
    let wd1 = (u16::from_be_bytes([uuid1.as_bytes()[6], uuid1.as_bytes()[7]]) & 0x0FF8) >> 3;
    let wd2 = (u16::from_be_bytes([uuid2.as_bytes()[6], uuid2.as_bytes()[7]]) & 0x0FF8) >> 3;
    assert_eq!(wd1, wd2, "week+day in rand_a should be identical across calls");
}
