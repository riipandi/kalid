#![cfg(any(feature = "tokio", feature = "smol"))]

use kalid::Kalid;

#[test]
fn async_tokio_runtest() {
    async fn inner() {
        let k = Kalid::new_async().await;
        assert_eq!(k.as_string().len(), 16);

        let id = kalid::generate_kalid_async().await;
        assert_eq!(id.len(), 16);
    }

    #[cfg(feature = "tokio")]
    tokio::runtime::Runtime::new().unwrap().block_on(inner());
    #[cfg(feature = "smol")]
    smol::block_on(inner());
}
