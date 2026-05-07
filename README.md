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

## Ecosystem

| Crate | Description |
|---|---|
| `axhash` | High-performance hashing engine |
| `axhash-map` | Fast HashMap/HashSet powered by `hashbrown` |
| `axhash-indexmap` | Ordered maps with AxHash |
| `axhash-dashmap` | Concurrent DashMap powered by AxHash |

```text
┌──────────────────────────────────────────────────────────┐
│                       axhash-map                         │
│                                                          │
│   Type aliases (Serde-compatible)                        │
│   HashMap<K, V>              HashSet<T>                  │
│                                                          │
│   Branded newtypes (ergonomic constructors)              │
│   AxHashMap<K, V>            AxHashSet<T>                │
│         │                          │                     │
│   hashbrown::HashMap    hashbrown::HashSet               │
│   (SwissTable layout)                                    │
│         │                          │                     │
│        BuildHasherDefault<AxHasher>                      │
│         (AES-NI accelerated hash engine)                 │
└──────────────────────────────────────────────────────────┘
```

---

## Two usage modes

This crate provides **two ways** to use the same fast hasher. Pick the one that fits your situation:

### Mode 1 — Type alias (`HashMap` / `HashSet`)

Plain type aliases over `hashbrown` with `BuildHasherDefault<AxHasher>` baked in.
Because there is no wrapper struct, **Serde and other `#[derive]`-based crates work out of the box**.

```rust
use axhash_map::HashMap; // or HashSet

// Works with serde::Serialize / Deserialize without any extra config.
let mut map: HashMap<&str, u32> = HashMap::default();
map.insert("fast", 1);
```

### Mode 2 — Branded newtype (`AxHashMap` / `AxHashSet`)

A thin newtype wrapper that adds the familiar `::new()` / `::with_capacity()` constructors.
Every `hashbrown` method is accessible transparently via `Deref`.

```rust
use axhash_map::AxHashMap;

let mut map: AxHashMap<&str, u32> = AxHashMap::new();
map.insert("fast", 1);
```

| Need | Use |
|---|---|
| `::new()` / `::with_capacity()` | `AxHashMap` / `AxHashSet` |
| Serde `#[derive(Serialize, Deserialize)]` | `HashMap` / `HashSet` |
| Custom / seeded hasher | `AxHashMap::with_hasher(AxBuildHasher::with_seed(s))` |
| Raw `hashbrown` access | `RawHashMap` / `RawHashSet` |

---

## Benchmark Results

Measured on Apple Silicon (`release` build, `N = 100,000` items).

| Scenario                    | AxHashMap | std HashMap | Speedup  |
| --------------------------- | --------: | ----------: | :------: |
| Insert — `u64` keys         |    379 µs |    1,032 µs | **2.7×** |
| Insert — `String` keys      |    896 µs |    1,673 µs | **1.9×** |
| Lookup — all hits           |    200 µs |      748 µs | **3.7×** |
| Lookup — 50% hit / 50% miss |    767 µs |    1,994 µs | **2.6×** |
| Iteration (full scan)       |    130 µs |      124 µs |  ~equal  |

> Iteration performance is effectively identical because iteration does not invoke the hasher.

Run the benchmarks yourself:

```bash
cargo bench --bench map_comparison
# HTML reports → target/criterion/
```

---

## Installation

```toml
[dependencies]
axhash-map = "0.1"
```

No feature flags required. AES acceleration is detected at runtime; a portable
fallback is used automatically on CPUs without AES instructions.

---

## Quick start

### Using the type alias (`HashMap`)

```rust
use axhash_map::HashMap;
use core::hash::BuildHasherDefault;
use axhash_map::AxHasher;

// Construct via Default (zero-cost).
let mut map: HashMap<&str, u32> = HashMap::default();
map.insert("alice", 42);
map.insert("bob",   17);

assert_eq!(map["alice"], 42);
assert_eq!(map.len(), 2);
```

### Using the branded newtype (`AxHashMap`)

```rust
use axhash_map::AxHashMap;

let mut scores: AxHashMap<&str, u32> = AxHashMap::new();

scores.insert("alice", 42);
scores.insert("bob",   17);
scores.insert("carol", 99);

// Index operator
assert_eq!(scores["alice"], 42);

// Safe lookup
assert_eq!(scores.get("bob"), Some(&17));
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

let a: AxHashSet<u32> = [1, 2, 3].into_iter().collect();
let b: AxHashSet<u32> = [2, 3, 4].into_iter().collect();

let union:        AxHashSet<u32> = a.union(&b).copied().collect();
let intersection: AxHashSet<u32> = a.intersection(&b).copied().collect();
assert_eq!(union.len(), 4);
assert_eq!(intersection.len(), 2);
```

---

## Constructors

### Branded newtype constructors

```rust
use axhash_map::{AxHashMap, AxHashSet, AxBuildHasher};

// Default (zero seed)
let map: AxHashMap<String, i32> = AxHashMap::new();
let set: AxHashSet<String>      = AxHashSet::new();

// Pre-allocate to avoid rehashing
let map = AxHashMap::<String, i32>::with_capacity(10_000);
let set = AxHashSet::<String>::with_capacity(10_000);

// Custom seed — use OS entropy for hash-flooding resistance
let seed: u64 = 0xdeadbeef_cafebabe;
let map: AxHashMap<String, i32, AxBuildHasher> =
    AxHashMap::with_hasher(AxBuildHasher::with_seed(seed));

// Custom seed + pre-allocated capacity
let map: AxHashMap<String, i32, AxBuildHasher> =
    AxHashMap::with_capacity_and_hasher(10_000, AxBuildHasher::with_seed(seed));
```

### Type alias constructors

```rust
use axhash_map::HashMap;

// Use hashbrown's built-in constructors directly on the alias.
let mut map: HashMap<String, i32> = HashMap::default();
let mut map = HashMap::<String, i32>::with_capacity(10_000);
```

---

## Collecting from iterators

```rust
use axhash_map::{AxHashMap, AxHashSet};

// FromIterator for AxHashMap
let map: AxHashMap<&str, usize> = [("a", 1), ("b", 2), ("c", 3)]
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
`hashbrown::HashMap` / `hashbrown::HashSet`, so every method — `entry`,
`retain`, `drain`, `reserve`, `shrink_to_fit`, and more — is directly
accessible without any extra imports.

```rust
use axhash_map::AxHashMap;

let mut map: AxHashMap<&str, u32> = AxHashMap::new();

map.entry("hits").and_modify(|n| *n += 1).or_insert(1);
map.entry("hits").and_modify(|n| *n += 1).or_insert(1);
assert_eq!(map["hits"], 2);

map.insert("temp", 0);
map.retain(|_, v| *v > 0);
assert!(!map.contains_key("temp"));
```

---

## Interoperability with raw hashbrown types

The crate re-exports `RawHashMap` and `RawHashSet` (the bare hashbrown types)
and provides `From` conversions in both directions so you can cross the boundary
without a direct `hashbrown` dependency in your own `Cargo.toml`.

```rust
use core::hash::BuildHasherDefault;
use axhash_map::{AxHashMap, RawHashMap, AxHasher};

// Wrap a raw hashbrown map.
let raw: RawHashMap<&str, u32, BuildHasherDefault<AxHasher>> =
    RawHashMap::with_hasher(BuildHasherDefault::default());
let wrapped: AxHashMap<&str, u32> = raw.into();

// Unwrap back to hashbrown.
let raw: RawHashMap<&str, u32, BuildHasherDefault<AxHasher>> = wrapped.into_inner();
```

---

## When to use a custom seed

The default hasher uses a constant seed — the output is **deterministic** across
runs. This is fine for most workloads.

If your map accepts keys from **untrusted external input** (e.g. HTTP request
parameters) and you want to defend against **hash-flooding attacks**, supply a
random seed derived from OS entropy:

```rust
use axhash_map::{AxHashMap, AxBuildHasher};

// In production, generate from `rand`, `getrandom`, or similar.
let seed: u64 = 0x1234_5678_9abc_def0;

let mut map: AxHashMap<String, String, AxBuildHasher> =
    AxHashMap::with_hasher(AxBuildHasher::with_seed(seed));
```

---

## Feature flags

This crate has no feature flags. The AES hardware path is selected at
**runtime** via CPUID — you always ship a single binary.

---

## Dependency footprint

```text
axhash-map
├── axhash-core   (AxHasher + AxBuildHasher — AES hash engine)
└── hashbrown     (SwissTable, no default features — ahash excluded)
```

---

## License

MIT — see [LICENSE](LICENSE).

[hashbrown]: https://crates.io/crates/hashbrown
[axhash]: https://crates.io/crates/axhash
