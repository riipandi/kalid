use kalid::{Kalid, KalidParseError};

#[test]
fn parse_roundtrip() {
    let original = Kalid::new();
    let s = original.as_string();
    let parsed = Kalid::parse(&s).unwrap();
    assert_eq!(parsed, original);
}

#[test]
fn parse_known_string() {
    let kalid = Kalid::parse("000000000000a01p").unwrap();
    assert_eq!(kalid.epoch_ms(), 0);
}

#[test]
fn parse_invalid_length() {
    assert_eq!(Kalid::parse("short"), Err(KalidParseError::InvalidLength));
    assert_eq!(Kalid::parse("000000000000a01p00"), Err(KalidParseError::InvalidLength));
}

#[test]
fn parse_invalid_timestamp() {
    assert_eq!(Kalid::parse("zzzzzzzzzzzza01p"), Err(KalidParseError::InvalidTimestamp));
}

#[test]
fn parse_invalid_month() {
    assert_eq!(Kalid::parse("000000000000m01p"), Err(KalidParseError::InvalidMonth));
    assert_eq!(Kalid::parse("000000000000001p"), Err(KalidParseError::InvalidMonth));
}

#[test]
fn parse_invalid_week() {
    assert_eq!(Kalid::parse("000000000000aabp"), Err(KalidParseError::InvalidWeek));
}

#[test]
fn parse_invalid_day() {
    assert_eq!(Kalid::parse("000000000000a01x"), Err(KalidParseError::InvalidDay));
}

#[test]
fn parse_mismatch() {
    // Valid format but components don't match
    assert_eq!(Kalid::parse("000000000000b01m"), Err(KalidParseError::Mismatch));
}
