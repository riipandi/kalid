// -- Collision & dedup characterization --------------------------------

#[test]
fn collision_benchmark_vs_nanoid() {
    const N: usize = 100_000;

    // nanoid: generate 16-char IDs for fair comparison
    let nanoid_ids: Vec<String> = (0..N).map(|_| nanoid::nanoid!(16)).collect();
    let kalid_ids: Vec<String> = (0..N).map(|_| kalid::generate_kalid()).collect();

    let nano_collisions = N - {
        let mut set = std::collections::HashSet::new();
        for id in &nanoid_ids {
            set.insert(id.as_str());
        }
        set.len()
    };

    let kalid_collisions = N - {
        let mut set = std::collections::HashSet::new();
        for id in &kalid_ids {
            set.insert(id.as_str());
        }
        set.len()
    };
    println!("Collision benchmark: N={N}, nanoid collisions={nano_collisions}, kalid collisions={kalid_collisions}");
    println!("  nanoid: {} unique in {N} (0 collisions = expected)", N - nano_collisions);
    println!(
        "  kalid: {} unique in {N} (kalid 1ms resolution — collisions expected within same ms)",
        N - kalid_collisions
    );
    assert_eq!(nano_collisions, 0, "nanoid should have 0 collisions");
}

#[test]
fn performance_dry_run() {
    const N: usize = 10_000;
    use std::time::Instant;

    let start = Instant::now();
    for _ in 0..N {
        core::hint::black_box(kalid::generate_kalid());
    }
    let kalid_dur = start.elapsed();

    let start = Instant::now();
    for _ in 0..N {
        core::hint::black_box(nanoid::nanoid!(16));
    }
    let nanoid_dur = start.elapsed();

    let start = Instant::now();
    for _ in 0..N {
        core::hint::black_box(uuid::Uuid::now_v7());
    }
    let uuid_dur = start.elapsed();

    let start = Instant::now();
    for _ in 0..N {
        core::hint::black_box(ulid::Ulid::r#gen().to_string());
    }
    let ulid_dur = start.elapsed();

    let kalid_per = kalid_dur.as_nanos() / N as u128;
    let nanoid_per = nanoid_dur.as_nanos() / N as u128;
    let uuid_per = uuid_dur.as_nanos() / N as u128;
    let ulid_per = ulid_dur.as_nanos() / N as u128;

    println!("Performance: {N} iterations");
    println!("  Kalid : {:>8?} total, ~{:>6}ns/op", kalid_dur, kalid_per);
    println!("  UUIDv7: {:>8?} total, ~{:>6}ns/op", uuid_dur, uuid_per);
    println!("  ULID  : {:>8?} total, ~{:>6}ns/op", ulid_dur, ulid_per);
    println!("  Nanoid: {:>8?} total, ~{:>6}ns/op", nanoid_dur, nanoid_per);
}
