# Changelog

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [0.0.2] - 2026-07-14

### Added

- UUID v7 interop: `to_uuid_v7()`, `from_uuid_v7()` with deterministic roundtrip
- `uuid` feature flag (optional, enabled by default). Opt out with `default-features = false`
- Integration tests in `tests/` (basics, parse, uuid interop, edge, sorting, collision/benchmark)
- Doc examples on all public items (12 doc-tests)
- `make ci` pipeline: check, fmt, clippy, test (nextest), doc
- `make bench`, `make coverage`, `make publish-dry`, `make publish`
- Criterion benchmarks in `benches/bench.rs` (9 benchmarks)
- `from-epoch` example
- README: goals, non-goals, when to use / when not to use, benchmarks with multiplier table

### Changed

- Extracted tests from `src/lib.rs` into dedicated `tests/*.rs` files
- Made `MONTH_CHARS` and `DAY_CHARS` public constants
- Bumped MSRV to 1.90.0
- Updated all doc paths from `elph_core::utils::kalid` to `kalid`
- Renamed package from `calid` to `kalid`

### Fixed

- Deprecated `criterion::black_box` → `std::hint::black_box`
- Removed duplicate package keys in `Cargo.toml`

## [0.0.1] - 2026-07-04

### Added

- Initial release
- `Kalid::new()`, `Kalid::from_epoch()`, `Kalid::from_epoch_ms()`, `Kalid::parse()`
- `generate_kalid()` convenience function
- K-sortable 16-character ID format with encoded month/week/day
- UUID v7 basic interoperability
