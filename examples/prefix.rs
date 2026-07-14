//! Prefix and separator via KalidBuilder.
//!
//! ```bash
//! cargo run --example prefix
//! ```

fn main() {
    let ids: Vec<(&str, String)> = vec![
        ("prefix(\"order\")", kalid::Kalid::builder().prefix("order").build()),
        (".separator('-')", kalid::Kalid::builder().prefix("user").separator('-').build()),
        (".separator('.')", kalid::Kalid::builder().prefix("log").separator('.').build()),
        (".separator(':')", kalid::Kalid::builder().prefix("obj").separator(':').build()),
        (".no_separator()", kalid::Kalid::builder().prefix("dbg").no_separator().build()),
    ];

    println!("── Prefix builder ──────────────────────────────────");
    println!("  Kalid::builder()");
    for (label, id) in &ids {
        println!("    {label:20}  {id}");
    }

    let k = kalid::Kalid::from_epoch_ms(0);
    let ep = kalid::Kalid::builder().prefix("epoch").separator(':').build_from(&k);
    println!("    build_from(&kalid)    {ep}");
    println!("──────────────────────────────────────────────────");
}
