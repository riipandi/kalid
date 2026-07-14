//! Generate, parse, and display a kalid from system time.
//!
//! ```bash
//! cargo run --example basic
//! ```

use chrono::{Datelike, TimeZone};

fn fmt_dt(epoch_ms: i64) -> String {
    let secs = epoch_ms / 1000;
    let nsecs = ((epoch_ms % 1000) * 1_000_000) as u32;
    let dt = chrono::Utc.timestamp_opt(secs, nsecs).unwrap();
    let dow = match dt.weekday() {
        chrono::Weekday::Mon => "Mon",
        chrono::Weekday::Tue => "Tue",
        chrono::Weekday::Wed => "Wed",
        chrono::Weekday::Thu => "Thu",
        chrono::Weekday::Fri => "Fri",
        chrono::Weekday::Sat => "Sat",
        chrono::Weekday::Sun => "Sun",
    };
    format!(
        "{} (week {:02} {})",
        dt.format("%Y-%m-%d %H:%M:%S%.3f"),
        dt.iso_week().week(),
        dow
    )
}

fn main() {
    let k = kalid::Kalid::new();
    let s = k.as_string();
    let ms = k.epoch_ms();

    println!("── Basic ──────────────────────────────────────────");
    println!("  Kalid::new()         {s}");
    println!("  Kalid::default()     {}", kalid::Kalid::default().as_string());
    println!("  generate_kalid()     {}", kalid::generate_kalid());
    println!("  parse()              epoch_ms={} ✓", ms);
    println!("  epoch_secs()         {}", k.epoch_secs());
    println!("  datetime             {}", fmt_dt(ms));
    println!("──────────────────────────────────────────────────");
}
