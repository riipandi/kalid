# Changelog

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [0.0.3] - 2026-07-14

### Added

- Async support: `Kalid::new_async()`, `generate_kalid_async()`
- `tokio` / `smol` features (optional, disabled by default, mutually exclusive)
- Internal `rt` module with runtime-agnostic blocking pool dispatch
- Async benchmarks (tokio): `new_async` ~4.4µs, `generate_kalid_async` ~5.5µs
- Async test: `tests/async.rs` (cfg-gated, 1 test with tokio/smol)
- Property-based tests: epoch roundtrip, leap year boundaries, all days/months, format regex, trait bounds (`tests/properties.rs`)
- Error-handling example: `examples/error-handling.rs`
- Async example: `examples/async.rs`
- `#[cfg]` gates on bench file and uuid-interop example for `--no-default-features` support
- Compile-time error when both `tokio` and `smol` are selected

### Changed

- Bumped version to 0.0.3
- Updated README: 2-column benchmark table, async rows, simplified layout
- Updated CHANGELOG with full 0.0.3 entries

### Fixed

- Bench compilation with `--no-default-features` (stub `fn main`)
- Example compilation without `uuid` feature (cfg-gated main)
- Doc comments now include `// INVARIANT` justification on all `.unwrap()` calls

## [0.0.2] - 2026-07-14

### Added

- UUID v7 interop: `to_uuid_v7()`, `from_uuid_v7()` with deterministic roundtrip
- `uuid` feature flag (optional, enabled by default)
- Integration tests in `tests/` (basics, parse, uuid interop, edge, sorting, collision)
- Doc examples on all public items
- `make ci`, `make bench`, `make coverage`, `make publish-dry`, `make publish`
- Criterion benchmarks (9 benchmarks)
- `from-epoch` example, README with goals and non-goals

### Changed

- Extracted tests from `src/lib.rs` into `tests/*.rs`
- Made `MONTH_CHARS` and `DAY_CHARS` public
- Bumped MSRV to 1.90.0
- Renamed package from `calid` to `kalid`

### Fixed

- Deprecated `criterion::black_box` → `std::hint::black_box`

## [0.0.1] - 2026-07-04

### Added

- Initial release: `Kalid::new()`, `from_epoch()`, `from_epoch_ms()`, `parse()`
- `generate_kalid()` convenience function
- K-sortable 16-character ID format with encoded month/week/day
