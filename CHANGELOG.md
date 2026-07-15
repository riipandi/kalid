# Changelog

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [0.1.0] - 2026-07-15

**Initial stable release.**

## [0.0.5] - 2026-07-15

### Added

- 100% test coverage across default, `tokio`, and `smol` feature builds
- `civil_negative_days_roundtrip` test exercising the pre-1970 (negative day/year) branches of the branchless calendar arithmetic

### Changed

- Moved public-API tests out of the inline `mod tests` in `src/lib.rs` into `tests/unit.rs`; private-function tests remain inline
- README benchmark section: Kalid as the 1.0× baseline, added async (`tokio`) benchmark subsection
- Bumped version to 0.0.5

## [0.0.4] - 2026-07-14

### Added

- `KalidBuilder` with `prefix()`, `separator()`, `no_separator()`, `build()`, `build_from()`
- Default separator `_` (URL-safe per RFC 3986). Customise via `.separator('-')` or `.no_separator()`
- `Kalid::builder()` convenience method
- 8 builder integration tests in `tests/builder.rs`
- 15 doc-tests covering all public items

### Changed

- Bumped version to 0.0.4
- README: accurate hardware specs (Apple M2 Pro, 10 cores, macOS 15.6.5, Rust 1.97.0)
- README benchmark table: 3 columns with multiplier vs `generate_kalid` baseline
- README benchmark data from fresh run with criterion.rs 100 samples
- CHANGELOG updated

## [0.0.3] - 2026-07-14

### Added

- Async support: `Kalid::new_async()`, `generate_kalid_async()`
- `tokio` / `smol` features (optional, disabled by default, mutually exclusive)
- Internal `rt` module with runtime-agnostic blocking pool dispatch
- Async benchmarks (tokio): `new_async` ~4.4µs, `generate_kalid_async` ~5.5µs
- Async test (cfg-gated), property-based tests, error-handling example, async example
- `#[cfg]` gates on bench file and uuid-interop example for `--no-default-features`

### Changed

- Bumped version to 0.0.3
- Updated README: 2-column benchmark table, async rows, simplified layout

### Fixed

- Bench compilation with `--no-default-features` (stub `fn main`)
- Example compilation without `uuid` feature (cfg-gated main)
- Doc comments with `// INVARIANT` on all `.unwrap()` calls

## [0.0.2] - 2026-07-14

### Added

- UUID v7 interop: `to_uuid_v7()`, `from_uuid_v7()` with deterministic roundtrip
- `uuid` feature flag (optional, enabled by default)
- Integration tests, doc examples, `make ci`, benchmarks, README with goals/non-goals

### Changed

- Extracted tests from `src/lib.rs` into `tests/*.rs`
- Made `MONTH_CHARS` and `DAY_CHARS` public
- Bumped MSRV to 1.90.0
- Renamed package from `calid` to `kalid`

### Fixed

- Deprecated `criterion::black_box` → `std::hint::black_box`

## [0.0.1] - 2026-07-04

### Added

- Initial release: `Kalid::new()`, `from_epoch()`, `from_epoch_ms()`, `parse()`, `generate_kalid()`
- K-sortable 16-character ID format with encoded month/week/day
