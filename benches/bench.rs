#[cfg(not(feature = "uuid"))]
fn main() {}

#[cfg(feature = "uuid")]
use criterion::{Criterion, criterion_group, criterion_main};

#[cfg(feature = "uuid")]
fn bench_kalid_generate(c: &mut Criterion) {
    c.bench_function("kalid::generate_kalid", |b| b.iter(kalid::generate_kalid));
}

#[cfg(feature = "uuid")]
fn bench_kalid_from_epoch_ms(c: &mut Criterion) {
    c.bench_function("kalid::from_epoch_ms", |b| {
        b.iter(|| std::hint::black_box(kalid::Kalid::from_epoch_ms(1_784_060_036_000)))
    });
}

#[cfg(feature = "uuid")]
fn bench_kalid_as_string(c: &mut Criterion) {
    let kalid = kalid::Kalid::from_epoch_ms(1_784_060_036_000);
    c.bench_function("kalid::as_string", |b| b.iter(|| kalid.as_string()));
}

#[cfg(feature = "uuid")]
fn bench_kalid_parse(c: &mut Criterion) {
    let s = "019f62686310a34m";
    c.bench_function("kalid::parse", |b| b.iter(|| kalid::Kalid::parse(std::hint::black_box(s))));
}

#[cfg(feature = "uuid")]
fn bench_kalid_to_uuid_v7(c: &mut Criterion) {
    let kalid = kalid::Kalid::from_epoch_ms(1_784_060_036_000);
    c.bench_function("kalid::to_uuid_v7", |b| b.iter(|| kalid.to_uuid_v7()));
}

#[cfg(feature = "uuid")]
fn bench_kalid_from_uuid_v7(c: &mut Criterion) {
    let uuid = kalid::Kalid::from_epoch_ms(1_784_060_036_000).to_uuid_v7();
    c.bench_function("kalid::from_uuid_v7", |b| {
        b.iter(|| kalid::Kalid::from_uuid_v7(std::hint::black_box(&uuid)))
    });
}

#[cfg(feature = "uuid")]
fn bench_nanoid_generate(c: &mut Criterion) {
    c.bench_function("nanoid::nanoid!(16)", |b| b.iter(|| nanoid::nanoid!(16)));
}

#[cfg(feature = "uuid")]
fn bench_uuid_now_v7(c: &mut Criterion) {
    c.bench_function("uuid::Uuid::now_v7", |b| b.iter(uuid::Uuid::now_v7));
}

#[cfg(feature = "uuid")]
fn bench_ulid_generate(c: &mut Criterion) {
    c.bench_function("ulid::Ulid::r#gen().to_string()", |b| {
        b.iter(|| ulid::Ulid::r#gen().to_string())
    });
}

// -- Async (requires uuid + tokio) -----------------------------------

#[cfg(all(feature = "uuid", feature = "tokio"))]
fn bench_kalid_generate_async(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("kalid::generate_kalid_async", |b| {
        b.iter(|| rt.block_on(kalid::generate_kalid_async()))
    });
}

#[cfg(all(feature = "uuid", feature = "tokio"))]
fn bench_kalid_new_async(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("kalid::Kalid::new_async", |b| b.iter(|| rt.block_on(kalid::Kalid::new_async())));
}

// -- Criterion groups -------------------------------------------------

#[cfg(feature = "uuid")]
criterion_group!(
    name = kalid;
    config = Criterion::default().sample_size(100);
    targets =
        bench_kalid_generate, bench_kalid_from_epoch_ms,
        bench_kalid_as_string, bench_kalid_parse,
        bench_kalid_to_uuid_v7, bench_kalid_from_uuid_v7,
        bench_nanoid_generate, bench_uuid_now_v7, bench_ulid_generate,
);

#[cfg(all(feature = "uuid", feature = "tokio"))]
criterion_group!(
    name = kalid_async;
    config = Criterion::default().sample_size(100);
    targets =
        bench_kalid_generate_async, bench_kalid_new_async,
);

#[cfg(all(feature = "uuid", not(feature = "tokio")))]
criterion_main!(kalid);

#[cfg(all(feature = "uuid", feature = "tokio"))]
criterion_main!(kalid, kalid_async);
