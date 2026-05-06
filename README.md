# axhash-map

High-performance `HashMap` and `HashSet` collections for Rust.

**Powered by [hashbrown]** (SwissTable layout) · **Fueled by [axhash]** (AES-NI accelerated hashing)

[![Crates.io](https://img.shields.io/crates/v/axhash-map.svg)](https://crates.io/crates/axhash-map)
[![Docs.rs](https://docs.rs/axhash-map/badge.svg)](https://docs.rs/axhash-map)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## Why axhash-map?

`std::collections::HashMap` uses **SipHash-1-3** by default — a secure but comparatively slow hash function. For most workloads you don't need cryptographic resistance; you need throughput.

`axhash-map` swaps the hasher for **axhash**, which exploits hardware AES instructions (AES-NI on x86-64, AES on ARMv8) to produce hashes at near-memory-bandwidth speed. The underlying table is **hashbrown** (SwissTable), the same implementation that backs `std::collections::HashMap` in Rust's standard library — so the table operations are identical; only the hashing step is faster.

```
┌─────────────────────────────────────────┐
│              axhash-map                 │
│                                         │
│   AxHashMap<K, V>    AxHashSet<T>       │
│         │                  │            │
│   hashbrown::HashMap  hashbrown::HashSet│
│   (SwissTable layout)                   │
│         │                  │            │
│       axhash::AxBuildHasher             │
│   (AES-NI accelerated hash engine)      │
└─────────────────────────────────────────┘
```

---

## Benchmark Results

Measured on Apple Silicon (`release` build, `N = 100,000` items).

Hasher comparison:

- **AxHashMap** (`hashbrown` + `axhash`)
- **std::collections::HashMap** (`SipHash-1-3`)

| Scenario                    | AxHashMap | std HashMap | Speedup  |
| --------------------------- | --------: | ----------: | :------: |
| Insert — `u64` keys         |    379 µs |    1,032 µs | **2.7×** |
| Insert — `String` keys      |    896 µs |    1,673 µs | **1.9×** |
| Lookup — all hits           |    200 µs |      748 µs | **3.7×** |
| Lookup — 50% hit / 50% miss |    767 µs |    1,994 µs | **2.6×** |
| Iteration (full scan)       |    130 µs |      124 µs |  ~equal  |

> Iteration performance is effectively identical because iteration does not invoke the hasher. Both maps use the same SwissTable layout provided by `hashbrown`.

Run the benchmarks yourself:

```bash
cargo bench --bench map_comparison
# HTML reports → target/criterion/
```

---

## Installation

```toml
# Cargo.toml
[dependencies]
axhash-map = "0.1"
```

No feature flags required. The AES acceleration is detected at runtime; a portable fallback is used automatically on CPUs without AES instructions.

---

## Quick start

### HashMap

```rust
use axhash_map::AxHashMap;

// Create an empty map.
let mut scores: AxHashMap<&str, u32> = AxHashMap::new();

scores.insert("alice", 42);
scores.insert("bob",   17);
scores.insert("carol", 99);

// Index operator
println!("{}", scores["alice"]); // 42

// Safe lookup
if let Some(&s) = scores.get("bob") {
    println!("bob scored {s}");
}

// Iteration
for (name, score) in &scores {
    println!("{name}: {score}");
}
```

### HashSet

```rust
use axhash_map::AxHashSet;

let mut seen: AxHashSet<u64> = AxHashSet::new();

seen.insert(1);
seen.insert(2);
seen.insert(2); // duplicate — ignored

assert_eq!(seen.len(), 2);
assert!(seen.contains(&1));

// Set operations (via Deref to hashbrown::HashSet)
let a: AxHashSet<u32> = [1, 2, 3].into_iter().collect();
let b: AxHashSet<u32> = [2, 3, 4].into_iter().collect();

let union:        AxHashSet<u32> = a.union(&b).copied().collect();
let intersection: AxHashSet<u32> = a.intersection(&b).copied().collect();
```

---

## Constructors

Both `AxHashMap` and `AxHashSet` expose the same constructor family:

```rust
use axhash_map::{AxHashMap, AxHashSet, AxBuildHasher};

// Default seed (fastest, deterministic within a process)
let map: AxHashMap<String, i32> = AxHashMap::new();
let set: AxHashSet<String>      = AxHashSet::new();

// Pre-allocate to avoid rehashing
let map = AxHashMap::<String, i32>::with_capacity(10_000);
let set = AxHashSet::<String>::with_capacity(10_000);

// Custom seed — use a random value for hash-flooding resistance
let hasher = AxBuildHasher::with_seed(0xdeadbeef_cafebabe);
let map: AxHashMap<String, i32> = AxHashMap::with_hasher(hasher);

// Custom seed + pre-allocated capacity
let hasher = AxBuildHasher::with_seed(0xdeadbeef_cafebabe);
let map: AxHashMap<String, i32> = AxHashMap::with_capacity_and_hasher(10_000, hasher);
```

---

## Collecting from iterators

```rust
use axhash_map::{AxHashMap, AxHashSet};

// FromIterator for AxHashMap
let map: AxHashMap<&str, usize> = vec![("a", 1), ("b", 2), ("c", 3)]
    .into_iter()
    .collect();

// FromIterator for AxHashSet
let set: AxHashSet<i32> = [1, 2, 3, 2, 1].into_iter().collect(); // len == 3

// Extend
let mut map: AxHashMap<u32, u32> = AxHashMap::new();
map.extend([(1, 10), (2, 20)]);
map.extend([(3, 30), (4, 40)]);
```

---

## All hashbrown methods are available

`AxHashMap` and `AxHashSet` implement `Deref` / `DerefMut` to the underlying
`hashbrown::HashMap` / `hashbrown::HashSet`, so every method on those types —
`entry`, `retain`, `drain`, `reserve`, `shrink_to_fit`, `raw_entry`, and more —
is directly accessible without any extra imports.

```rust
use axhash_map::AxHashMap;

let mut map: AxHashMap<&str, u32> = AxHashMap::new();

// entry API (from hashbrown, via Deref)
map.entry("hits").and_modify(|n| *n += 1).or_insert(1);
map.entry("hits").and_modify(|n| *n += 1).or_insert(1);
assert_eq!(map["hits"], 2);

// retain
map.insert("temp", 0);
map.retain(|_, v| *v > 0);
assert!(!map.contains_key("temp"));
```

---

## Interoperability with raw hashbrown types

The crate re-exports `RawHashMap` and `RawHashSet` (the bare hashbrown types)
and provides `From` conversions in both directions so you can cross the boundary
without a direct hashbrown dependency in your own `Cargo.toml`.

```rust
use axhash_map::{AxHashMap, RawHashMap, AxBuildHasher};

// Wrap a raw hashbrown map
let raw: RawHashMap<&str, u32, AxBuildHasher> =
    RawHashMap::with_hasher(AxBuildHasher::new());
let wrapped: AxHashMap<&str, u32> = raw.into();

// Unwrap back to hashbrown
let raw: RawHashMap<&str, u32, AxBuildHasher> = wrapped.into_inner();
```

---

## When to use a custom seed

The default `AxBuildHasher::new()` uses an internal constant as seed — the
output is **deterministic** across runs. This is fine for most workloads.

If your map accepts keys from **untrusted external input** (e.g. HTTP request
parameters) and you want to defend against **hash-flooding attacks**, supply a
random seed derived from OS entropy:

```rust
use axhash_map::{AxHashMap, AxBuildHasher};

// In real code, generate this from `rand`, `getrandom`, or similar.
let seed: u64 = /* os_random_u64() */ 0x1234_5678_9abc_def0;

let mut map: AxHashMap<String, String> =
    AxHashMap::with_hasher(AxBuildHasher::with_seed(seed));
```

---

## Feature flags

This crate has no feature flags of its own. The AES hardware path in axhash is
selected at **runtime** via CPUID — you always ship a single binary.

---

## Dependency footprint

```
axhash-map
├── axhash-core   (AxBuildHasher — AES hash engine)
└── hashbrown     (SwissTable, no default features — ahash excluded)
```

`axhash` itself is **not** a dependency; `axhash-core` is the crate that owns
`AxBuildHasher`. The wrapper crate adds no utility functions we don't use.

---

## License

MIT — see [LICENSE](LICENSE).

[hashbrown]: https://crates.io/crates/hashbrown
[axhash]: https://crates.io/crates/axhash
