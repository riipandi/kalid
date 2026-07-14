use chrono::{TimeZone, Utc};
use kalid::Kalid;

// -- K-sortability --------------------------------------------------

#[test]
fn k_sortable_across_all_boundaries() {
    // Generate kalids for every day in July 2025 + cross year
    let start_ms = Utc.with_ymd_and_hms(2025, 7, 1, 0, 0, 0).unwrap().timestamp() * 1000;
    let mut prev = String::new();

    // 31 days in July
    for day_offset in 0..31 {
        let ms = start_ms + day_offset * 86_400_000;
        let kalid = Kalid::from_epoch_ms(ms);
        let s = kalid.as_string();

        if !prev.is_empty() {
            assert!(prev < s, "inversion: {prev} >= {s} at offset {day_offset}");
        }
        prev = s;
    }

    // Cross year: Dec 31 → Jan 1
    let dec31 = Utc.with_ymd_and_hms(2025, 12, 31, 23, 59, 59).unwrap().timestamp() * 1000 + 999;
    let jan1 = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap().timestamp() * 1000;
    let d31 = Kalid::from_epoch_ms(dec31).as_string();
    let j1 = Kalid::from_epoch_ms(jan1).as_string();
    assert!(d31 < j1, "year boundary inversion: {d31} >= {j1}");
}
