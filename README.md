# Kalid

Calendar-based, K-sortable unique ID generator with UUID v7 interoperability.
Kalid encodes a Unix millisecond timestamp into a compact 16-character string with optional prefix.

## When to use

- **Human-readable** sortable IDs with optional prefix (`"order_019f..."`, `"user_019f..."`)
- **K-sortability** — lexicographic = chronological across all boundaries
- Seamless **UUID v7** interop (optional `uuid` feature)
- **Fast**, minimal-dependency, zero `unsafe`

## When not to use

- **Distributed ID generation** — only 3 random bits → collisions within same ms
- **Cryptographic security** — predictable from timestamp
- **Space-constrained** — always 16 ASCII chars + prefix
- **Cross-ms sort stability** — prefix doesn't affect sort order

## Quick Start

```toml
[dependencies]
kalid = "0.0.5"
```

```rust
use kalid::Kalid;

// Basic
let k = Kalid::new();
println!("{}", k.as_string()); // "019f6243c3a0g29n"

// With prefix (builder)
let id = Kalid::builder().prefix("order").build();
println!("{id}"); // "order_019f6243c3a0g29n"

// Custom separator / no separator
Kalid::builder().prefix("user").separator('-').build();
Kalid::builder().prefix("dbg").no_separator().build();
```

## Prefix (KalidBuilder)

| Method                | Description                         |
|-----------------------|-------------------------------------|
| `.prefix("...")`      | Set prefix string                   |
| `.separator('-')`     | Custom separator (default `_`)      |
| `.no_separator()`     | Remove separator entirely           |
| `.build()`            | Generate new kalid with this config |
| `.build_from(&kalid)` | Format an existing `Kalid`          |

Separator defaults to `_` (URL-safe per RFC3986). Valid URL-safe chars: `- . _ ~`.

## Examples

```sh
cargo run --example basic
cargo run --example uuid-interop
cargo run --example async --features tokio
cargo run --example async --features smol
cargo run --example error-handling
cargo run --example from-epoch
cargo run --example prefix
cargo run --example sorting
```

## Benchmarks

- **Hardware:** Apple M2 Pro (10 cores), 16 GB RAM, macOS 26.5.2, Rust 1.97.0.
- **Tool:** criterion.rs, 100 samples per benchmark. `kalid::generate_kalid` is the 1.0× baseline.

### Kalid API operations

| Operation                      | Time (mean) | × of `generate_kalid` |
|--------------------------------|-------------|-----------------------|
| `kalid::from_epoch_ms`         | 0.36 ns     | 0.007×                |
| `kalid::from_uuid_v7`          | 0.57 ns     | 0.012×                |
| `kalid::parse`                 | 15.3 ns     | 0.32×                 |
| `kalid::as_string`             | 22.6 ns     | 0.47×                 |
| `kalid::to_uuid_v7`            | 29.8 ns     | 0.62×                 |
| `kalid::generate_kalid` (base) | 48.2 ns     | 1.0×                  |

### vs other 16-char ID generators

| Generator                         | Time (mean) | vs Kalid        |
|-----------------------------------|-------------|-----------------|
| `kalid::generate_kalid`           | 48.2 ns     | 1.0× (baseline) |
| `ulid::Ulid::r#gen().to_string()` | 61.4 ns     | 1.27× slower    |
| `uuid::Uuid::now_v7`              | 896 ns      | 18.6× slower    |
| `nanoid::nanoid!(16)`             | 1,211 ns    | 25.1× slower    |

### Async (requires `tokio` feature)

| Operation                     | Time (mean) |
|-------------------------------|-------------|
| `kalid::Kalid::new_async`     | 4.99 µs     |
| `kalid::generate_kalid_async` | 5.12 µs     |

> Kalid generates a 16-character ID **1.3× faster than ULID**, **~19× faster than
> UUID v7**, and **~25× faster than nanoid**. Every Kalid component operation stays
> under 30 ns; the async path adds a ~5 µs runtime overhead from the tokio executor
> (`spawn_blocking`).

```bash
make bench                       # sync (default features)
make bench -- --features tokio   # sync + async
```

## Feature flags

| Feature | Default | Description                             |
|---------|---------|-----------------------------------------|
| `uuid`  | on      | `to_uuid_v7()` / `from_uuid_v7()`       |
| `tokio` | off     | Async via `tokio::task::spawn_blocking` |
| `smol`  | off     | Async via `smol::unblock`               |

`tokio` and `smol` are mutually exclusive.

## UUID v7 Interoperability

Kalid and [UUID v7](https://www.rfc-editor.org/rfc/rfc9562) share the same ms
timestamp. Week+day encoded in `rand_a` (12 bits). Roundtrip deterministic.

Use [UUID Timestamp extraction tool](https://www.authgear.com/tools/uuidv7-generator)
from Authgear to validate the Unix Epoch timestamp.

## Limitations

- **No global uniqueness** — only 3 random bits → collisions within same ms
- **Not cryptographically secure**
- **No variable-length** — always 16 chars + prefix
- **3-bit randomness** — at most 8 unique Kalids per ms
- **Chrono range** — finite date range

## Contributing

We welcome contributions to make Kalid even better!

- Read our **[Contributing Guidelines](./CONTRIBUTING.md)** for detailed guidelines
- Fork the repository and create a feature branch
- Submit a pull request with a clear title and description
- Join the discussion on [GitHub Issues](https://github.com/riipandi/kalid/issues)

## License

Licensed under either of [Apache License 2.0][license-apache] or [MIT license][license-mit] at your option.

> Unless you explicitly state otherwise, any contribution intentionally submitted
> for inclusion in this project by you, as defined in the Apache-2.0 license, shall
> be dual licensed as above, without any additional terms or conditions.

Copyrights in this project are retained by their contributors.

See the [LICENSE-APACHE](./LICENSE-APACHE) and [LICENSE-MIT](./LICENSE-MIT) files for more information.

---

<sub>🤫 Psst! If you like my work you can support me via [GitHub sponsors](https://github.com/sponsors/riipandi).</sub>

[![CreatorBadge](https://badgen.net/badge/icon/Aris%20Ripandi?label=Made+by&color=black&labelColor=black)](https://x.com/intent/follow?screen_name=riipandi)

<!-- References -->

[license-apache]: https://www.tldrlegal.com/license/apache-license-2-0-apache-2-0
[license-mit]: https://www.tldrlegal.com/license/mit-license
