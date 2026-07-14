//! Lossless Kalid ↔ UUID v7 interop (requires uuid feature).
//!
//! ```bash
//! cargo run --example uuid-interop
//! ```

fn main() {
    #[cfg(not(feature = "uuid"))]
    {
        eprintln!("This example requires the `uuid` feature (enabled by default).");
        eprintln!("  cargo run --example uuid-interop --features uuid");
        std::process::exit(1);
    }

    #[cfg(feature = "uuid")]
    run();
}

#[cfg(feature = "uuid")]
fn run() {
    use chrono::{Datelike, TimeZone};

    let k = kalid::Kalid::from_epoch_ms(1_784_060_036_000);
    let uuid = k.to_uuid_v7();
    let back = kalid::Kalid::from_uuid_v7(&uuid);

    let uuid_hex = uuid.as_bytes();
    let rand_a = (u16::from_be_bytes([uuid_hex[6], uuid_hex[7]]) & 0x0FFF) as u32;
    let w = (rand_a >> 6) & 0x3F;
    let d = (rand_a >> 3) & 0x07;

    println!("── Roundtrip ────────────────────────────────────");
    println!("  Kalid             {}", k.as_string());
    println!("  ↓ to_uuid_v7()");
    println!("  UUID v7           {uuid}");
    println!("    √ rand_a encodes week={w} day={d}");
    println!("  ↓ from_uuid_v7()");
    println!("  Kalid             {}  ✓", back.as_string());
    println!();

    let ext = uuid::Uuid::now_v7();
    let k3 = kalid::Kalid::from_uuid_v7(&ext);
    let secs = k3.epoch_ms() / 1000;
    let nsecs = ((k3.epoch_ms() % 1000) * 1_000_000) as u32;
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

    println!("── External UUID v7 → Kalid ──────────────────────");
    println!("  UUID v7           {ext}");
    println!("  ↓ from_uuid_v7()");
    println!("  Kalid             {}", k3.as_string());
    println!("  epoch_ms          {}", k3.epoch_ms());
    println!(
        "  datetime          {} (week {:02} {})",
        dt.format("%Y-%m-%d %H:%M:%S%.3f"),
        dt.iso_week().week(),
        dow
    );
    println!("─────────────────────────────────────────────────");
}
