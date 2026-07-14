# Kalid

Calendar-based, K-sortable unique ID generator with optional UUID v7 interop.
Kalid encodes a Unix millisecond timestamp into a compact 16-character string:

```text
{ms_hex:012}{month:1}{week:02}{day:1}
```

## When to use

- **Human-readable** sortable IDs for logs, debug output, CLI
- **K-sortability** — lexicographic = chronological across all boundaries
- Seamless **UUID v7** interop (optional `uuid` feature)
- **Fast**, minimal-dependency, zero `unsafe`

## When not to use

- **Distributed ID generation** — only 3 random bits → collisions within same ms expected
- **Cryptographic security** — predictable from timestamp
- **Space-constrained** — always 16 ASCII chars, no binary encoding
- **Cross-ms sort stability** — only 3 random bits → ties likely within same ms

## Quick Start

```toml
[dependencies]
kalid = "0.0.3"
```

Features are additive. Default enables `uuid`:

```toml
# Minimal (no UUID, no async)
kalid = { version = "0.0.3", default-features = false }
# With tokio async
kalid = { version = "0.0.3", features = ["tokio"] }
# With smol async
kalid = { version = "0.0.3", features = ["smol"] }
```

```rust
use kalid::Kalid;

let k = Kalid::new();
let parsed = Kalid::parse("000000000000a01p").unwrap();
assert_eq!(parsed.epoch_ms(), 0);

// UUID v7 roundtrip (requires feature "uuid", enabled by default)
let uuid = k.to_uuid_v7();
let back = Kalid::from_uuid_v7(&uuid);
assert_eq!(back.as_string(), k.as_string());
```

Examples: `cargo run --example basic`, `from-epoch`, `uuid-interop`, `sorting`, `async`.

## Benchmarks

Apple M4, Rust 1.97.0, criterion.rs 100 samples. Sorted by speed.

| Operation                      | Time     |
|--------------------------------|----------|
| `from_epoch_ms`                | 0.33 ns  |
| `from_uuid_v7`                 | 0.58 ns  |
| `to_uuid_v7`                   | 27.3 ns  |
| `as_string`                    | 97.9 ns  |
| `parse`                        | 112.5 ns |
| `generate_kalid`               | 170.6 ns |
| `new_async` (tokio)            | 4.4 µs   |
| `generate_kalid_async` (tokio) | 5.5 µs   |
| `uuid::Uuid::now_v7`           | 875.8 ns |
| `nanoid::nanoid!(16)`          | 1.17 µs  |

```bash
make bench                # all sync benches (default features)
make bench --features tokio  # includes async benches
```

## Feature flags

| Feature | Default | Description                             |
|---------|---------|-----------------------------------------|
| `uuid`  | ✅ on    | `to_uuid_v7()` / `from_uuid_v7()`       |
| `tokio` | off     | Async via `tokio::task::spawn_blocking` |
| `smol`  | off     | Async via `smol::unblock`               |

`tokio` and `smol` are mutually exclusive.

## UUID v7 Interoperability

Kalid and [UUID v7](https://www.rfc-editor.org/rfc/rfc9562) share the same ms
timestamp. Week+day are encoded in `rand_a` (12 bits = `[week:6][day:3][random:3]`).

**Kalid → UUID v7**: timestamp → bytes 0-5, week+day → `rand_a`.
**UUID v7 → Kalid**: extract ms + week/day. Roundtrip is deterministic.

## Limitations

- **No global uniqueness** — only 3 random bits → collisions within same ms
- **Not cryptographically secure**
- **No variable-length** — always 16 ASCII chars
- **3-bit randomness** — at most 8 unique Kalids per ms
- **Chrono range** — finite date range for component extraction

## Contributing

We welcome contributions to make Kalid even better!

- Read our **[Contributing Guidelines](./CONTRIBUTING.md)**
- Fork and create a feature branch
- Submit a pull request
- [GitHub Issues](https://github.com/riipandi/kalid/issues)

## License

Licensed under either of [Apache License 2.0][license-apache] or [MIT license][license-mit] at your option.

---

<sub>Created by [Aris Ripandi](https://github.com/riipandi).</sub>

<!-- References -->

[license-apache]: https://www.tldrlegal.com/license/apache-license-2-0-apache-2-0
[license-mit]: https://www.tldrlegal.com/license/mit-license
