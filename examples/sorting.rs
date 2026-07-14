//! K-sortability: lexicographic order matches chronological order.
//!
//! ```bash
//! cargo run --example sorting
//! ```

fn main() {
    let epochs = [
        (0, "1970-01-01"),
        (1_000_000_000_000, "2001-09-09"),
        (1_700_000_000_000, "2023-11-14"),
        (1_784_060_036_000, "2026-07-08"),
    ];

    let mut rows: Vec<_> = epochs
        .iter()
        .map(|&(ms, date)| {
            let k = kalid::Kalid::from_epoch_ms(ms);
            (k.as_string(), ms, date)
        })
        .collect();

    // Already chrono, but sort proves lexicographic stability
    rows.sort();

    println!("── K-sortability ──────────────────────────────");
    println!("  {:16}  {:>15}  {:14}", "Kalid", "epoch_ms", "date");
    println!("  ────────────────  ───────────────  ──────────────");
    for (k, ms, date) in &rows {
        println!("  {k:16}  {ms:15}  {date:14}");
    }
    println!();

    // Year boundary: Dec 31 → Jan 1
    let dec31 = kalid::Kalid::from_epoch_ms(1_784_009_999_999);
    let jan1 = kalid::Kalid::from_epoch_ms(1_784_092_800_000);
    let ok = dec31.as_string() < jan1.as_string();
    let mark = if ok { "✓" } else { "✗" };
    println!("  Year boundary: {}  <  {}    {mark}", dec31.as_string(), jan1.as_string());
    println!("──────────────────────────────────────────────");
}
