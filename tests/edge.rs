use kalid::Kalid;

#[test]
fn default_is_new() {
    let a = Kalid::default();
    let b = Kalid::new();
    assert!((a.epoch_ms() - b.epoch_ms()).abs() <= 1);
}

#[test]
fn epoch_zero_is_jan_1970() {
    let kalid = Kalid::from_epoch_ms(0);
    assert_eq!(kalid.epoch_ms(), 0);
    assert_eq!(kalid.epoch_secs(), 0);
    assert_eq!(kalid.as_string().len(), 16);
}

#[test]
fn from_epoch_sets_correct_ms() {
    let kalid = Kalid::from_epoch(1_784_060_036);
    assert_eq!(kalid.epoch_ms(), 1_784_060_036_000);
    assert_eq!(kalid.epoch_secs(), 1_784_060_036);
}

#[test]
fn generate_kalid_convenience() {
    let a = kalid::generate_kalid();
    let b = Kalid::new().as_string();
    assert_eq!(a.len(), 16);
    assert_eq!(b.len(), 16);
}
