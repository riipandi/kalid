//! Generate kalid IDs asynchronously (requires tokio or smol feature).
//!
//! ```bash
//! cargo run --example async --features tokio
//! cargo run --example async --features smol
//! ```

fn main() {
    #[cfg(feature = "tokio")]
    tokio_main();

    #[cfg(feature = "smol")]
    smol_main();

    #[cfg(not(any(feature = "tokio", feature = "smol")))]
    eprintln!("Requires `tokio` or `smol` feature.\n  cargo run --example async --features tokio");
}

#[cfg(feature = "tokio")]
fn tokio_main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let id = kalid::generate_kalid_async().await;
        println!("  generate_kalid_async()    {id}");

        let k = kalid::Kalid::new_async().await;
        println!("  Kalid::new_async()        {}  epoch_ms={}", k.as_string(), k.epoch_ms());
    });
}

#[cfg(feature = "smol")]
fn smol_main() {
    smol::block_on(async {
        let id = kalid::generate_kalid_async().await;
        println!("  generate_kalid_async()    {id}");

        let k = kalid::Kalid::new_async().await;
        println!("  Kalid::new_async()        {}  epoch_ms={}", k.as_string(), k.epoch_ms());
    });
}
