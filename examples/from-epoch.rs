//! Create a kalid from a known epoch (seconds or milliseconds).
//!
//! ```bash
//! cargo run --example from-epoch
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
    let from_s = kalid::Kalid::from_epoch(1_784_060_036);
    let from_ms = kalid::Kalid::from_epoch_ms(1_784_060_036_000);

    println!("── From epoch ──────────────────────────────────────");
    println!("  from_epoch(secs)         {}", from_s.as_string());
    println!("    epoch_secs()           {}", from_s.epoch_secs());
    println!("    epoch_ms()             {}", from_s.epoch_ms());
    println!("    datetime               {}", fmt_dt(from_s.epoch_ms()));
    println!();
    println!("  from_epoch_ms(ms)        {}", from_ms.as_string());
    println!("    epoch_ms()             {}", from_ms.epoch_ms());
    println!("    epoch_secs()           {}", from_ms.epoch_secs());
    println!("    datetime               {}", fmt_dt(from_ms.epoch_ms()));
    println!("──────────────────────────────────────────────────");
}
