//! K-sortability: lexicographic order matches chronological order.
//!
//! ```bash
//! cargo run --example sorting
//! ```

use chrono::{Datelike, TimeZone};

fn main() {
    let epochs: Vec<(i64, &str)> = vec![
        (0, "1970-01-01"),
        (1_000_000_000_000, "2001-09-09"),
        (1_700_000_000_000, "2023-11-14"),
        (1_784_060_036_000, "2026-07-08"),
    ];

    let mut rows: Vec<_> = epochs
        .iter()
        .map(|&(ms, label)| {
            let k = kalid::Kalid::from_epoch_ms(ms);
            let secs = ms / 1000;
            let nsecs = ((ms % 1000) * 1_000_000) as u32;
            let dt = chrono::Utc.timestamp_opt(secs, nsecs).unwrap();
            let weekday = match dt.weekday() {
                chrono::Weekday::Mon => "Mon",
                chrono::Weekday::Tue => "Tue",
                chrono::Weekday::Wed => "Wed",
                chrono::Weekday::Thu => "Thu",
                chrono::Weekday::Fri => "Fri",
                chrono::Weekday::Sat => "Sat",
                chrono::Weekday::Sun => "Sun",
            };
            (k.as_string(), label, weekday, dt.format("%Y-%m-%d %H:%M:%S").to_string())
        })
        .collect();

    rows.sort();

    println!("── K-sortability ──────────────────────────────────");
    println!("  {:<18} {:<14} {:<6} Full datetime", "Kalid", "Date", "Day");
    println!("  {}", "─".repeat(58));
    for (k, label, day, dt) in &rows {
        println!("  {k:18} {label:14} {day:6} {dt}");
    }
    println!();

    let dec31 = kalid::Kalid::from_epoch_ms(1_784_009_999_999);
    let jan1 = kalid::Kalid::from_epoch_ms(1_784_092_800_000);
    assert!(dec31.as_string() < jan1.as_string());
    println!("  Year boundary:  {}  <  {}  ✓", dec31.as_string(), jan1.as_string());
    println!("──────────────────────────────────────────────────");
}
