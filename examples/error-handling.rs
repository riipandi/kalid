//! Parse errors: demonstration of invalid kalid strings and their error messages.
//!
//! ```bash
//! cargo run --example error-handling
//! ```

fn main() {
    let bad_inputs = [
        ("", "empty string"),
        ("abc", "too short"),
        ("000000000000a01p00", "too long"),
        ("000000000000a01p", "valid — control case"),
        ("zzzzzzzzzzzza01p", "non-hex timestamp"),
        ("000000000000m01p", "month out of range (m > l)"),
        ("000000000000aabp", "invalid week (ab)"),
        ("000000000000a01x", "invalid day (x > s)"),
        ("000000000000b01m", "valid format, components mismatch"),
    ];

    println!("── Parse error messages ───────────────────────────");
    for &(input, desc) in &bad_inputs {
        match kalid::Kalid::parse(input) {
            Ok(k) => println!("  {desc:40}  OK  {}", k.as_string()),
            Err(e) => println!("  {desc:40}  {}", e),
        }
    }
    println!("──────────────────────────────────────────────────");
}
