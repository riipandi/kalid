use kalid::Kalid;

#[test]
fn builder_default() {
    let id = Kalid::builder().build();
    assert!(!id.contains('_'));
    assert_eq!(id.len(), 16); // no prefix = just kalid
}

#[test]
fn builder_with_prefix() {
    let id = Kalid::builder().prefix("order").build();
    assert!(id.starts_with("order_"));
    assert_eq!(id.len(), 22); // "order_" + 16
}

#[test]
fn builder_custom_separator() {
    let id = Kalid::builder().prefix("user").separator('-').build();
    assert!(id.starts_with("user-"));
    assert!(!id.contains('_'));
}

#[test]
fn builder_no_separator() {
    let id = Kalid::builder().prefix("user").no_separator().build();
    assert!(id.starts_with("user"));
    assert!(!id.contains('_'));
    assert!(!id.contains('.'));
}

#[test]
fn builder_empty_prefix() {
    let id = Kalid::builder().prefix("").build();
    assert_eq!(id.len(), 17); // "" + "_" + 16
    assert!(id.starts_with('_'));
}

#[test]
fn builder_build_from() {
    let k = Kalid::from_epoch_ms(0);
    let formatted = Kalid::builder().prefix("test").separator(':').build_from(&k);
    assert_eq!(formatted, "test:000000000000a01p");
}

#[test]
fn builder_no_prefix_with_separator_setting() {
    // Setting separator without prefix has no effect
    let id = Kalid::builder().separator('|').build();
    assert!(!id.contains('|'));
    assert_eq!(id.len(), 16);
}

#[test]
fn builder_chain_all() {
    let id = Kalid::builder()
        .prefix("log")
        .separator('.')
        .no_separator()
        .prefix("dbg")
        .build();
    // Last prefix wins, no_separator removes separator
    assert!(id.starts_with("dbg"));
    assert!(!id.contains('.'));
}
