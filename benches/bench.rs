#![cfg(feature = "uuid")]

use criterion::{Criterion, criterion_group, criterion_main};

fn bench_kalid_generate(c: &mut Criterion) {
    c.bench_function("kalid::generate_kalid", |b| b.iter(kalid::generate_kalid));
}

fn bench_kalid_from_epoch_ms(c: &mut Criterion) {
    c.bench_function("kalid::from_epoch_ms", |b| {
        b.iter(|| std::hint::black_box(kalid::Kalid::from_epoch_ms(1_784_060_036_000)))
    });
}

fn bench_kalid_as_string(c: &mut Criterion) {
    let kalid = kalid::Kalid::from_epoch_ms(1_784_060_036_000);
    c.bench_function("kalid::as_string", |b| b.iter(|| kalid.as_string()));
}

fn bench_kalid_parse(c: &mut Criterion) {
    let s = "019f62686310a34m";
    c.bench_function("kalid::parse", |b| b.iter(|| kalid::Kalid::parse(std::hint::black_box(s))));
}

fn bench_kalid_to_uuid_v7(c: &mut Criterion) {
    let kalid = kalid::Kalid::from_epoch_ms(1_784_060_036_000);
    c.bench_function("kalid::to_uuid_v7", |b| b.iter(|| kalid.to_uuid_v7()));
}

fn bench_kalid_from_uuid_v7(c: &mut Criterion) {
    let uuid = kalid::Kalid::from_epoch_ms(1_784_060_036_000).to_uuid_v7();
    c.bench_function("kalid::from_uuid_v7", |b| {
        b.iter(|| kalid::Kalid::from_uuid_v7(std::hint::black_box(&uuid)))
    });
}

// -- Competitor comparisons -----------------------------------------

fn bench_nanoid_generate(c: &mut Criterion) {
    c.bench_function("nanoid::nanoid!(16)", |b| b.iter(|| nanoid::nanoid!(16)));
}

fn bench_uuid_now_v7(c: &mut Criterion) {
    c.bench_function("uuid::Uuid::now_v7", |b| b.iter(uuid::Uuid::now_v7));
}

fn bench_ulid_generate(c: &mut Criterion) {
    c.bench_function("ulid::Ulid::r#gen().to_string()", |b| {
        b.iter(|| ulid::Ulid::r#gen().to_string())
    });
}

criterion_group!(
    name = kalid;
    config = Criterion::default().sample_size(100);
    targets =
        bench_kalid_generate,
        bench_kalid_from_epoch_ms,
        bench_kalid_as_string,
        bench_kalid_parse,
        bench_kalid_to_uuid_v7,
        bench_kalid_from_uuid_v7,
        bench_nanoid_generate,
        bench_uuid_now_v7,
        bench_ulid_generate,
);
criterion_main!(kalid);
