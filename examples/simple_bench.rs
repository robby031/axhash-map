//! A quick end-to-end demonstration and informal benchmark of `axhash-map`.
//!
//! Run with:
//!   cargo run --example simple_bench --release
//!
//! The example exercises three scenarios:
//!   1. Sequential integer keys (cache-friendly, low collision pressure).
//!   2. String keys (real-world-ish workload).
//!   3. A deduplication task using `AxHashSet`.

use std::hint::black_box;
use std::time::Instant;

use axhash_map::{AxHashMap, AxHashSet};

const N: usize = 1_000_000;

fn main() {
    println!("axhash-map — simple benchmark (N = {N})\n");

    bench_integer_map();
    bench_string_map();
    bench_set_dedup();

    println!("\nAll checks passed.");
}

// ── 1. Integer key map ────────────────────────────────────────────────────────

fn bench_integer_map() {
    // --- insert ---
    let t = Instant::now();
    let mut map: AxHashMap<u64, u64> = AxHashMap::with_capacity(N);
    for i in 0..N as u64 {
        map.insert(i, i.wrapping_mul(6364136223846793005));
    }
    let insert_ms = t.elapsed().as_millis();

    // --- lookup ---
    let t = Instant::now();
    let mut sum = 0u64;
    for i in 0..N as u64 {
        sum = sum.wrapping_add(*black_box(map.get(&i).unwrap()));
    }
    let lookup_ms = t.elapsed().as_millis();

    println!(
        "[integer map]  insert {N} entries: {insert_ms} ms  |  lookup {N} entries: {lookup_ms} ms  |  checksum: {sum:016x}"
    );

    assert_eq!(map.len(), N);
}

// ── 2. String key map ─────────────────────────────────────────────────────────

fn bench_string_map() {
    // --- build keys up front (not included in timing) ---
    let keys: Vec<String> = (0..N).map(|i| format!("key_{i:08}")).collect();

    // --- insert ---
    let t = Instant::now();
    let mut map: AxHashMap<&str, usize> = AxHashMap::with_capacity(N);
    for (i, k) in keys.iter().enumerate() {
        map.insert(k.as_str(), i);
    }
    let insert_ms = t.elapsed().as_millis();

    // --- lookup ---
    let t = Instant::now();
    let mut found = 0usize;
    for k in &keys {
        if map.contains_key(k.as_str()) {
            found += 1;
        }
    }
    let lookup_ms = t.elapsed().as_millis();

    println!(
        "[string  map]  insert {N} entries: {insert_ms} ms  |  lookup {N} entries: {lookup_ms} ms  |  found: {found}"
    );

    assert_eq!(found, N);
}

// ── 3. Set deduplication ──────────────────────────────────────────────────────

fn bench_set_dedup() {
    // Build a stream with ~50 % duplicates.
    let stream: Vec<u32> = (0..N as u32)
        .flat_map(|i| [i, i / 2]) // each value appears at least twice
        .collect();
    let expected_unique = N; // 0..N covers exactly N distinct values

    let t = Instant::now();
    let set: AxHashSet<u32> = stream.into_iter().collect();
    let dedup_ms = t.elapsed().as_millis();

    println!(
        "[set dedup  ]  deduplicate {} entries → {} unique: {} ms",
        N * 2,
        set.len(),
        dedup_ms
    );

    assert_eq!(set.len(), expected_unique);
}
