//! Criterion benchmarks: `AxHashMap` vs `std::collections::HashMap`.
//!
//! Scenarios
//! ---------
//! 1. **Insert**   — fill an empty map with 100 000 items (u64 keys and String keys).
//! 2. **Lookup**   — 100 000 lookups against a pre-populated map:
//!                   - `hit`   : every key is present (best-case probing).
//!                   - `mixed` : ~50 % of keys are absent (realistic workload).
//! 3. **Iteration** — iterate all 100 000 entries and accumulate a checksum.
//!
//! Run:
//!   cargo bench --bench map_comparison
//!
//! Results land in `target/criterion/` as HTML reports.

use std::collections::HashMap as StdHashMap;
use std::hint::black_box;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

use axhash_map::AxHashMap;

// ── Configuration ─────────────────────────────────────────────────────────────

const N: usize = 100_000;
const SEED: u64 = 0xabcd_ef01_2345_6789;

fn configured_criterion() -> Criterion {
    Criterion::default()
        .measurement_time(Duration::from_secs(5))
        .warm_up_time(Duration::from_secs(2))
        .sample_size(100)
}

// ── SplitMix64 PRNG ───────────────────────────────────────────────────────────
// Deterministic, zero-dependency PRNG used to generate reproducible key sets.

struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

fn u64_keys(n: usize, seed: u64) -> Vec<u64> {
    let mut rng = SplitMix64(seed);
    (0..n).map(|_| rng.next()).collect()
}

fn string_keys(n: usize, seed: u64) -> Vec<String> {
    let mut rng = SplitMix64(seed);
    (0..n)
        .map(|i| format!("key-{:010}-{:016x}", i, rng.next()))
        .collect()
}

// ── 1. Insert ─────────────────────────────────────────────────────────────────

fn bench_insert(c: &mut Criterion) {
    // --- u64 keys ---
    {
        let keys = u64_keys(N, SEED);
        let mut group = c.benchmark_group("insert/u64");
        group.throughput(Throughput::Elements(N as u64));

        group.bench_function(BenchmarkId::new("AxHashMap", N), |b| {
            b.iter(|| {
                let mut map = AxHashMap::with_capacity(N);
                for &k in &keys {
                    map.insert(k, black_box(k.wrapping_mul(6364136223846793005)));
                }
                black_box(map)
            });
        });

        group.bench_function(BenchmarkId::new("StdHashMap", N), |b| {
            b.iter(|| {
                let mut map = StdHashMap::with_capacity(N);
                for &k in &keys {
                    map.insert(k, black_box(k.wrapping_mul(6364136223846793005)));
                }
                black_box(map)
            });
        });

        group.finish();
    }

    // --- String keys ---
    {
        let keys = string_keys(N, SEED);
        let mut group = c.benchmark_group("insert/string");
        group.throughput(Throughput::Elements(N as u64));

        group.bench_function(BenchmarkId::new("AxHashMap", N), |b| {
            b.iter(|| {
                let mut map = AxHashMap::with_capacity(N);
                for (i, k) in keys.iter().enumerate() {
                    map.insert(k.as_str(), black_box(i as u64));
                }
                black_box(map)
            });
        });

        group.bench_function(BenchmarkId::new("StdHashMap", N), |b| {
            b.iter(|| {
                let mut map = StdHashMap::with_capacity(N);
                for (i, k) in keys.iter().enumerate() {
                    map.insert(k.as_str(), black_box(i as u64));
                }
                black_box(map)
            });
        });

        group.finish();
    }
}

// ── 2. Lookup ─────────────────────────────────────────────────────────────────

fn bench_lookup(c: &mut Criterion) {
    let insert_keys = u64_keys(N, SEED);
    // Hit keys  : every key is in the map.
    let hit_keys = insert_keys.clone();
    // Mixed keys: interleave existing keys with keys that are likely absent.
    let absent_keys = u64_keys(N, SEED ^ 0xffff_ffff_ffff_ffff);
    let mixed_keys: Vec<u64> = hit_keys
        .iter()
        .zip(absent_keys.iter())
        .flat_map(|(&h, &m)| [h, m])
        .collect();

    // Pre-populate both maps once — lookup benches measure only query cost.
    let ax_map: AxHashMap<u64, u64> = insert_keys.iter().map(|&k| (k, k)).collect();
    let std_map: StdHashMap<u64, u64> = insert_keys.iter().map(|&k| (k, k)).collect();

    // --- hit ---
    {
        let mut group = c.benchmark_group("lookup/hit");
        group.throughput(Throughput::Elements(N as u64));

        group.bench_function(BenchmarkId::new("AxHashMap", N), |b| {
            b.iter(|| {
                let mut sum = 0u64;
                for k in &hit_keys {
                    if let Some(&v) = ax_map.get(k) {
                        sum = sum.wrapping_add(v);
                    }
                }
                black_box(sum)
            });
        });

        group.bench_function(BenchmarkId::new("StdHashMap", N), |b| {
            b.iter(|| {
                let mut sum = 0u64;
                for k in &hit_keys {
                    if let Some(&v) = std_map.get(k) {
                        sum = sum.wrapping_add(v);
                    }
                }
                black_box(sum)
            });
        });

        group.finish();
    }

    // --- mixed (50 % hit / 50 % miss) ---
    {
        let mut group = c.benchmark_group("lookup/mixed");
        group.throughput(Throughput::Elements(mixed_keys.len() as u64));

        group.bench_function(BenchmarkId::new("AxHashMap", N), |b| {
            b.iter(|| {
                let mut sum = 0u64;
                for k in &mixed_keys {
                    if let Some(&v) = ax_map.get(k) {
                        sum = sum.wrapping_add(v);
                    }
                }
                black_box(sum)
            });
        });

        group.bench_function(BenchmarkId::new("StdHashMap", N), |b| {
            b.iter(|| {
                let mut sum = 0u64;
                for k in &mixed_keys {
                    if let Some(&v) = std_map.get(k) {
                        sum = sum.wrapping_add(v);
                    }
                }
                black_box(sum)
            });
        });

        group.finish();
    }
}

// ── 3. Iteration ──────────────────────────────────────────────────────────────

fn bench_iter(c: &mut Criterion) {
    let insert_keys = u64_keys(N, SEED);

    let ax_map: AxHashMap<u64, u64> = insert_keys.iter().map(|&k| (k, k)).collect();
    let std_map: StdHashMap<u64, u64> = insert_keys.iter().map(|&k| (k, k)).collect();

    let mut group = c.benchmark_group("iteration");
    group.throughput(Throughput::Elements(N as u64));

    group.bench_function(BenchmarkId::new("AxHashMap", N), |b| {
        b.iter(|| {
            let mut sum = 0u64;
            for (_, &v) in &ax_map {
                sum = sum.wrapping_add(v);
            }
            black_box(sum)
        });
    });

    group.bench_function(BenchmarkId::new("StdHashMap", N), |b| {
        b.iter(|| {
            let mut sum = 0u64;
            for (_, &v) in &std_map {
                sum = sum.wrapping_add(v);
            }
            black_box(sum)
        });
    });

    group.finish();
}

// ── Entry point ───────────────────────────────────────────────────────────────

criterion_group!(
    name    = benches;
    config  = configured_criterion();
    targets = bench_insert, bench_lookup, bench_iter
);
criterion_main!(benches);
