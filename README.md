# Kalid

Calendar-based, K-sortable unique ID generator.

Kalid encodes a Unix millisecond timestamp into a compact 16-character string:

```text
{ms_hex:012}{month:1}{week:02}{day:1}
```

| Segment | Length | Encoding                                      |
|---------|--------|-----------------------------------------------|
| Ms      | 12     | Unix timestamp in milliseconds, lowercase hex |
| Month   | 1      | `a` (January) .. `l` (December)               |
| Week    | 2      | ISO week number 01-53                         |
| Day     | 1      | `m` (Monday) .. `s` (Sunday)                  |

## When to use

- You need **human-readable**, sortable IDs for logs, debug output, or CLI tools
- You want to see month/week/day encoded directly in the ID
- You need **K-sortability** — lexicographic order = chronological order, across all time boundaries including year rollover
- You need seamless **UUID v7** interop (optional `uuid` feature)
- You want a **fast**, minimal-dependency ID generator with zero `unsafe` code

## When not to use

- **Global uniqueness** — unlike UUID v7 or ULID, Kalid has no dedicated random component. Only 3 bits of `rand_a` are random. Collisions within the same millisecond are expected at high throughput.
- **Cryptographic security** — Kalid is not cryptographically secure. Do not use for session tokens, secrets, or anything requiring unpredictability.
- **Binary/compact encoding** — Kalid is always 16-character ASCII. No configurable length, no binary format. For space-constrained systems, consider ULID (128-bit UUID-compatible).
- **Timestamp ordering before 1970 or after ~292M years** — Kalid uses `i64` epoch milliseconds. The encoding scheme works, but chrono's range limits apply for date-component extraction.
- **Cross-millisecond sort stability** — IDs generated within the same millisecond share the same timestamp prefix. Month/week/day is identical. Sort stability depends on the hex suffix (random bits), but there are only 3 random bits — expect ties.

## Quick Start

```toml
[dependencies]
kalid = "0.0.3"
```

To also enable **UUID v7 interop**, ensure the `uuid` feature is enabled (it's on by default):

```toml
[dependencies]
kalid = { version = "0.0.3", features = ["uuid"] }
```

Opt out if you don't need UUID v7:

```toml
kalid = { version = "0.0.3", default-features = false }
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

// UUID v7 roundtrip (requires uuid feature)
let uuid = k.to_uuid_v7();
let back = Kalid::from_uuid_v7(&uuid);
assert_eq!(back.as_string(), k.as_string());
```

Run examples:

```bash
cargo run --example basic
cargo run --example from-epoch
cargo run --example uuid-interop
cargo run --example sorting
```

## Benchmarks

Results measured on **macOS, Apple M2 Pro**, Rust 1.97.0, criterion.rs (100 samples each).

| Operation                         | Time         | vs `generate_kalid` |
|-----------------------------------|--------------|---------------------|
| `kalid::from_epoch_ms`            | 0.33 ns      | **483× faster**     |
| `kalid::from_uuid_v7`             | 0.57 ns      | **279× faster**     |
| `kalid::to_uuid_v7`               | 28.8 ns      | **5.6× faster**     |
| `ulid::Ulid::r#gen().to_string()` | 62.6 ns      | **2.6× faster**     |
| `kalid::as_string`                | 104.6 ns     | 1.5× faster         |
| `kalid::parse`                    | 120.9 ns     | 1.3× faster         |
| **`kalid::generate_kalid`**       | **159.9 ns** | **1.0× (baseline)** |
| `uuid::Uuid::now_v7`              | 871.8 ns     | 5.5× slower         |
| `nanoid::nanoid!(16)`             | 1,147.6 ns   | 7.2× slower         |

```bash
make bench
```

## UUID v7 Interoperability

Requires the `uuid` feature (enabled by default). Kalid and [UUID v7](https://www.rfc-editor.org/rfc/rfc9562) share the exact same millisecond timestamp. Week and day are encoded in `rand_a`:

```text
rand_a (12 bit) = [week:6][day:3][random:3]
```

**Kalid → UUID v7**: timestamp hex → bytes 0-5 (48 bits), week+day → `rand_a` (9 bits). Only `rand_b` (62 bits) remains random for UUID uniqueness.

**UUID v7 → Kalid**: extracts ms timestamp + week/day from `rand_a`. Roundtrip is deterministic — produces the exact same string.

Externally-generated UUID v7s decode to a valid Kalid — timestamp is accurate, but week+day are derived from the timestamp (not from `rand_a`).

## Feature flags

| Feature | Description                                                                 | Default |
|---------|-----------------------------------------------------------------------------|---------|
| `uuid`  | Enables `to_uuid_v7()` / `from_uuid_v7()` methods + `uuid` crate dependency | ✅ on    |

## Limitations

- **No global uniqueness guarantee** — Kalid only has 3 random bits. At high throughput, same-millisecond collisions are expected. Use UUID v7 or ULID for distributed uniqueness.
- **Not cryptographically secure** — the output is predictable from the timestamp. Do not use for secrets, tokens, or anti-forgery.
- **No variable-length encoding** — always exactly 16 ASCII characters. No binary/compact representation is provided.
- **3-bit randomness** — only `rand_a[2:0]` is random. For any given millisecond, at most 8 unique Kalids can be generated.
- **Chrono range dependency** — date component extraction (month/week/day) depends on `chrono::Utc`, which covers a wide but finite range.

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

<sub>🤫 Psst! If you like my work you can support me via [GitHub sponsors](https://github.com/sponsors/riipandi).</sub>

[![CreatorBadge](https://badgen.net/badge/icon/Aris%20Ripandi?label=Made+by&color=black&labelColor=black)](https://x.com/intent/follow?screen_name=riipandi)

<!-- References -->

[license-apache]: https://www.tldrlegal.com/license/apache-license-2-0-apache-2-0
[license-mit]: https://www.tldrlegal.com/license/mit-license
