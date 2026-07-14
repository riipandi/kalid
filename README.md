# Kalid

Calendar-based, K-sortable unique ID generator with UUID v7 interoperability.
Kalid encodes a Unix millisecond timestamp into a compact 16-character string.

```text
{ms_hex:012}{month:1}{week:02}{day:1}
```

| Segment | Length | Encoding                                      |
|---------|--------|-----------------------------------------------|
| Ms      | 12     | Unix timestamp in milliseconds, lowercase hex |
| Month   | 1      | `a` (January) .. `l` (December)               |
| Week    | 2      | ISO week number 01-53                         |
| Day     | 1      | `m` (Monday) .. `s` (Sunday)                  |

## Goals

- **K-sortable** — lexicographic order = chronological order across all boundaries (same ms, day, month, year, and December→January).
- **Human-readable** — encoded month/week/day at a glance.
- **UUID v7 lossless interop** — deterministic roundtrip `kalid → UUID v7 → kalid` produces the exact same string. Week+day are embedded in `rand_a` (12 bits).
- **Fast** — see [benchmarks](#benchmarks).

## Non-goals

- **Global uniqueness** — not designed for distributed ID generation without coordination. Unlike UUID v7 `/ ULID`, Kalid has no dedicated random component for uniqueness (only 3 bits of randomness). Collisions within the same millisecond are expected.
- **Cryptographic randomness** — Kalid is not cryptographically secure. The random bits are generated with a CSPRNG (`rand::fill`) but the ID is short and predictable from the timestamp.
- **Sorting by month/week/day** — the month, week, and day suffix is for human readability only. Sort order is driven entirely by the millisecond hex prefix.
- **Variable-length / database-optimized** — Kalid is always 16 characters. No compact/binary encoding is provided.

## Quick Start

Add to `Cargo.toml`:

```toml
[dependencies]
kalid = "0.0.1"
```

```rust
use kalid::Kalid;

// Generate from system time
let id = Kalid::new();
assert_eq!(id.as_string().len(), 16);

// From a known epoch
let k = Kalid::from_epoch_ms(1_784_060_036_000);
println!("{}", k.as_string()); // e.g. "019f6243c3a0g29n"

// Parse
let parsed = Kalid::parse("000000000000a01p").unwrap();
assert_eq!(parsed.epoch_ms(), 0);

// UUID v7 roundtrip (lossless)
let uuid = k.to_uuid_v7();
let back = Kalid::from_uuid_v7(&uuid);
assert_eq!(back.as_string(), k.as_string());
```

Run the examples:

```bash
cargo run --example basic
cargo run --example from-epoch
cargo run --example uuid-interop
cargo run --example sorting
```

## Benchmarks

Results measured on Apple M4 (10 cores, 2024), macOS, Rust 1.97.0, criterion.rs (100 samples each).

| Operation                         | Time                         |
|-----------------------------------|------------------------------|
| `kalid::from_epoch_ms`            | **~0.33 ns**                 |
| `kalid::from_uuid_v7`             | **~0.65 ns**                 |
| `kalid::to_uuid_v7`               | **~29.4 ns**                 |
| `ulid::Ulid::r#gen().to_string()` | **~56.5 ns** (≈1.9× slower)  |
| `kalid::as_string`                | **~104.6 ns**                |
| `kalid::parse`                    | **~120.9 ns**                |
| `kalid::generate_kalid`           | **~159.5 ns**                |
| `uuid::Uuid::now_v7`              | **~874.7 ns** (≈5.5× slower) |
| `nanoid::nanoid!(16)`             | **~1.15 µs** (≈7.2× slower)  |

Run benchmarks yourself:

```bash
make bench
```

## UUID v7 Interoperability

Kalid and [UUID v7](https://www.rfc-editor.org/rfc/rfc9562) (RFC 9562) share the exact same millisecond timestamp. Week and day are encoded in `rand_a`:

```text
rand_a (12 bit) = [week:6][day:3][random:3]
```

**Kalid → UUID v7**: timestamp hex → bytes 0-5 (48 bits), week+day → `rand_a` (9 bits).
Only 62 bits (`rand_b`) remain random for UUID uniqueness.

**UUID v7 → Kalid**: extracts ms timestamp + week/day from `rand_a`.
Roundtrip produces the exact same string.

Externally-generated UUID v7s (e.g. `Uuid::now_v7()`) decode to a valid Kalid — timestamp is accurate, but week+day are derived from the timestamp (not from `rand_a`).

## Contributing

We welcome contributions!

- Read our **[Contributing Guidelines](./CONTRIBUTING.md)**
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

<sub>Created by [Aris Ripandi](https://github.com/riipandi).</sub>

<!-- References -->

[license-apache]: https://www.tldrlegal.com/license/apache-license-2-0-apache-2-0
[license-mit]: https://www.tldrlegal.com/license/mit-license
