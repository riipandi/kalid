use kalid::Kalid;

// -- Basics ---------------------------------------------------------

#[test]
fn format_is_16_chars() {
    let id = kalid::generate_kalid();
    assert_eq!(id.len(), 16);
}

#[test]
fn as_string_format() {
    let kalid = Kalid::from_epoch_ms(0);
    let s = kalid.as_string();

    assert!(
        s[..12]
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "timestamp not lowercase hex"
    );

    // month [12]
    let m = s.as_bytes()[12];
    assert!((b'a'..=b'l').contains(&m), "month out of range");

    // week [13..15]
    assert!(s[13..15].bytes().all(|b| b.is_ascii_digit()), "week not numeric");

    // day [15]
    let d = s.as_bytes()[15];
    assert!((b'm'..=b's').contains(&d), "day out of range");

    assert_eq!(s.len(), 16);
}

#[test]
fn known_epoch_produces_expected_kalid() {
    // epoch 0 ms = Thursday, Jan 1, 1970 → month a, week 01, day p
    let kalid = Kalid::from_epoch_ms(0);
    assert_eq!(kalid.as_string(), "000000000000a01p");

    // epoch 0 seconds → same
    let kalid = Kalid::from_epoch(0);
    assert_eq!(kalid.as_string(), "000000000000a01p");
}

#[test]
fn month_mapping() {
    assert_eq!(kalid::MONTH_CHARS[0], 'a'); // January
    assert_eq!(kalid::MONTH_CHARS[11], 'l'); // December
}

#[test]
fn day_mapping() {
    assert_eq!(kalid::DAY_CHARS[0], 'm'); // Monday
    assert_eq!(kalid::DAY_CHARS[6], 's'); // Sunday
}
