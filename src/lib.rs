use core::fmt;
use core::hash::{BuildHasher, BuildHasherDefault, Hash};
use core::ops::{Deref, DerefMut, Index};

pub use axhash_core::{AxBuildHasher, AxHasher};
pub use hashbrown::{HashMap as RawHashMap, HashSet as RawHashSet};

// ── Compatibility type aliases ────────────────────────────────────────────────
// These expose the raw `hashbrown` types with `AxHasher` baked in via the
// standard-library `BuildHasherDefault<H>` adaptor.  Because they are plain
// type aliases (no wrapper struct), third-party crates such as Serde can
// derive `Serialize` / `Deserialize` on structs that contain them without
// any extra configuration.

/// Drop-in `hashbrown::HashMap` with [`AxHasher`] as the default hasher.
/// Use this alias when maximum third-party compatibility matters (e.g. Serde
/// `#[derive]`).  Every method on [`hashbrown::HashMap`] is available directly.
pub type HashMap<K, V> = RawHashMap<K, V, BuildHasherDefault<AxHasher>>;

/// Drop-in `hashbrown::HashSet` with [`AxHasher`] as the default hasher.
/// Use this alias when maximum third-party compatibility matters (e.g. Serde
/// `#[derive]`).  Every method on [`hashbrown::HashSet`] is available directly.
pub type HashSet<T> = RawHashSet<T, BuildHasherDefault<AxHasher>>;

// ── AxHashMap ────────────────────────────────────────────────────────────────
/// `AxHashMap<K, V>` is a thin newtype wrapper around [`HashMap<K, V>`] that
/// adds the familiar `::new()` / `::with_capacity()` constructor syntax.
/// Every method on [`hashbrown::HashMap`] is accessible via `Deref`.
pub struct AxHashMap<K, V, S = BuildHasherDefault<AxHasher>>(RawHashMap<K, V, S>);

impl<K, V> AxHashMap<K, V, BuildHasherDefault<AxHasher>> {
    /// Creates an empty map with the default [`AxHasher`].
    ///
    /// The map is initially created with a capacity of 0 and will reallocate
    /// as elements are inserted.
    #[inline]
    pub fn new() -> Self {
        Self(RawHashMap::with_hasher(BuildHasherDefault::default()))
    }

    /// Creates an empty map with at least the given capacity and the default
    /// [`AxHasher`].
    ///
    /// The map will be able to hold at least `capacity` elements without
    /// reallocating.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(RawHashMap::with_capacity_and_hasher(
            capacity,
            BuildHasherDefault::default(),
        ))
    }
}

impl<K, V, S: BuildHasher> AxHashMap<K, V, S> {
    /// Creates an empty map that uses the supplied `hasher`.
    ///
    /// Use this when you need a custom seed or a completely different
    /// [`BuildHasher`] (e.g. [`AxBuildHasher::with_seed`]).
    #[inline]
    pub fn with_hasher(hasher: S) -> Self {
        Self(RawHashMap::with_hasher(hasher))
    }

    /// Creates an empty map with at least the given capacity that uses the
    /// supplied `hasher`.
    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        Self(RawHashMap::with_capacity_and_hasher(capacity, hasher))
    }

    /// Consumes the wrapper and returns the underlying [`RawHashMap`].
    #[inline]
    pub fn into_inner(self) -> RawHashMap<K, V, S> {
        self.0
    }
}

// ── Deref / DerefMut ─────────────────────────────────────────────────────────

impl<K, V, S> Deref for AxHashMap<K, V, S> {
    type Target = RawHashMap<K, V, S>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V, S> DerefMut for AxHashMap<K, V, S> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// ── Standard traits ───────────────────────────────────────────────────────────

impl<K, V> Default for AxHashMap<K, V, BuildHasherDefault<AxHasher>> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<K: fmt::Debug, V: fmt::Debug, S> fmt::Debug for AxHashMap<K, V, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<K: Clone, V: Clone, S: Clone> Clone for AxHashMap<K, V, S> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<K: Hash + Eq, V: PartialEq, S: BuildHasher> PartialEq for AxHashMap<K, V, S> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<K: Hash + Eq, V: Eq, S: BuildHasher> Eq for AxHashMap<K, V, S> {}

impl<K, Q, V, S> Index<&Q> for AxHashMap<K, V, S>
where
    K: Hash + Eq + core::borrow::Borrow<Q>,
    Q: Hash + Eq + ?Sized,
    S: BuildHasher,
{
    type Output = V;

    #[inline]
    fn index(&self, key: &Q) -> &Self::Output {
        self.0.index(key)
    }
}

// ── FromIterator / Extend ─────────────────────────────────────────────────────

impl<K: Hash + Eq, V> FromIterator<(K, V)> for AxHashMap<K, V, BuildHasherDefault<AxHasher>> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (lower, _) = iter.size_hint();
        let mut map = Self::with_capacity(lower);
        map.extend(iter);
        map
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> Extend<(K, V)> for AxHashMap<K, V, S> {
    #[inline]
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<'a, K: Hash + Eq + Copy, V: Copy, S: BuildHasher> Extend<(&'a K, &'a V)>
    for AxHashMap<K, V, S>
{
    #[inline]
    fn extend<I: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

// ── IntoIterator ──────────────────────────────────────────────────────────────

impl<K, V, S> IntoIterator for AxHashMap<K, V, S> {
    type Item = (K, V);
    type IntoIter = hashbrown::hash_map::IntoIter<K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a AxHashMap<K, V, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = hashbrown::hash_map::Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut AxHashMap<K, V, S> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = hashbrown::hash_map::IterMut<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

// ── From conversions ──────────────────────────────────────────────────────────

impl<K, V, S> From<RawHashMap<K, V, S>> for AxHashMap<K, V, S> {
    #[inline]
    fn from(inner: RawHashMap<K, V, S>) -> Self {
        Self(inner)
    }
}

impl<K, V, S> From<AxHashMap<K, V, S>> for RawHashMap<K, V, S> {
    #[inline]
    fn from(wrapper: AxHashMap<K, V, S>) -> Self {
        wrapper.0
    }
}

// ── AxHashSet ────────────────────────────────────────────────────────────────

/// High-performance hash set backed by [`hashbrown`] (SwissTable) with
/// [`AxHasher`] (AES-NI accelerated hashing) as the default hasher.
///
/// `AxHashSet<T>` is a thin newtype wrapper around [`HashSet<T>`] that adds
/// the familiar `::new()` / `::with_capacity()` constructor syntax.
/// Every method on [`hashbrown::HashSet`] is accessible via `Deref`.
pub struct AxHashSet<T, S = BuildHasherDefault<AxHasher>>(RawHashSet<T, S>);

impl<T> AxHashSet<T, BuildHasherDefault<AxHasher>> {
    /// Creates an empty set with the default [`AxHasher`].
    #[inline]
    pub fn new() -> Self {
        Self(RawHashSet::with_hasher(BuildHasherDefault::default()))
    }

    /// Creates an empty set with at least the given capacity and the default
    /// [`AxHasher`].
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(RawHashSet::with_capacity_and_hasher(
            capacity,
            BuildHasherDefault::default(),
        ))
    }
}

impl<T, S: BuildHasher> AxHashSet<T, S> {
    /// Creates an empty set that uses the supplied `hasher`.
    #[inline]
    pub fn with_hasher(hasher: S) -> Self {
        Self(RawHashSet::with_hasher(hasher))
    }

    /// Creates an empty set with at least the given capacity that uses the
    /// supplied `hasher`.
    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        Self(RawHashSet::with_capacity_and_hasher(capacity, hasher))
    }

    /// Consumes the wrapper and returns the underlying [`RawHashSet`].
    #[inline]
    pub fn into_inner(self) -> RawHashSet<T, S> {
        self.0
    }
}

// ── Deref / DerefMut ─────────────────────────────────────────────────────────

impl<T, S> Deref for AxHashSet<T, S> {
    type Target = RawHashSet<T, S>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, S> DerefMut for AxHashSet<T, S> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// ── Standard traits ───────────────────────────────────────────────────────────

impl<T> Default for AxHashSet<T, BuildHasherDefault<AxHasher>> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: fmt::Debug, S> fmt::Debug for AxHashSet<T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Clone, S: Clone> Clone for AxHashSet<T, S> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Hash + Eq, S: BuildHasher> PartialEq for AxHashSet<T, S> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Hash + Eq, S: BuildHasher> Eq for AxHashSet<T, S> {}

// ── FromIterator / Extend ─────────────────────────────────────────────────────

impl<T: Hash + Eq> FromIterator<T> for AxHashSet<T, BuildHasherDefault<AxHasher>> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (lower, _) = iter.size_hint();
        let mut set = Self::with_capacity(lower);
        set.extend(iter);
        set
    }
}

impl<T: Hash + Eq, S: BuildHasher> Extend<T> for AxHashSet<T, S> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<'a, T: Hash + Eq + Copy, S: BuildHasher> Extend<&'a T> for AxHashSet<T, S> {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

// ── IntoIterator ──────────────────────────────────────────────────────────────

impl<T, S> IntoIterator for AxHashSet<T, S> {
    type Item = T;
    type IntoIter = hashbrown::hash_set::IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T, S> IntoIterator for &'a AxHashSet<T, S> {
    type Item = &'a T;
    type IntoIter = hashbrown::hash_set::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

// ── From conversions ──────────────────────────────────────────────────────────

impl<T, S> From<RawHashSet<T, S>> for AxHashSet<T, S> {
    #[inline]
    fn from(inner: RawHashSet<T, S>) -> Self {
        Self(inner)
    }
}

impl<T, S> From<AxHashSet<T, S>> for RawHashSet<T, S> {
    #[inline]
    fn from(wrapper: AxHashSet<T, S>) -> Self {
        wrapper.0
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use core::hash::BuildHasherDefault;

    // ── AxHashMap ────────────────────────────────────────────────────────────

    #[test]
    fn map_basic_operations() {
        let mut map: AxHashMap<&str, u32> = AxHashMap::new();
        assert!(map.is_empty());

        map.insert("one", 1);
        map.insert("two", 2);
        map.insert("three", 3);

        assert_eq!(map.len(), 3);
        assert_eq!(map["one"], 1);
        assert_eq!(map.get("two"), Some(&2));
        assert_eq!(map.get("missing"), None);

        map.remove("two");
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn map_with_capacity() {
        let map: AxHashMap<u32, u32> = AxHashMap::with_capacity(128);
        assert!(map.capacity() >= 128);
    }

    #[test]
    fn map_default() {
        let map: AxHashMap<u64, u64> = AxHashMap::default();
        assert!(map.is_empty());
    }

    #[test]
    fn map_from_iterator() {
        let pairs = vec![("a", 1u32), ("b", 2), ("c", 3)];
        let map: AxHashMap<&str, u32> = pairs.into_iter().collect();
        assert_eq!(map.len(), 3);
        assert_eq!(map["b"], 2);
    }

    #[test]
    fn map_extend() {
        let mut map: AxHashMap<u32, u32> = AxHashMap::new();
        map.extend([(1, 10), (2, 20)]);
        map.extend([(3, 30)]);
        assert_eq!(map.len(), 3);
        assert_eq!(map[&2], 20);
    }

    #[test]
    fn map_iter() {
        let map: AxHashMap<u32, u32> = [(1, 10), (2, 20)].into_iter().collect();
        let mut sum = 0u32;
        for (_, v) in &map {
            sum += v;
        }
        assert_eq!(sum, 30);
    }

    #[test]
    fn map_into_inner_roundtrip() {
        let mut map: AxHashMap<&str, i32> = AxHashMap::new();
        map.insert("x", 99);
        let raw: RawHashMap<&str, i32, BuildHasherDefault<AxHasher>> = map.into_inner();
        assert_eq!(raw["x"], 99);
        let wrapped: AxHashMap<&str, i32> = raw.into();
        assert_eq!(wrapped["x"], 99);
    }

    #[test]
    fn map_seeded_hasher() {
        let hasher = AxBuildHasher::with_seed(0x1234_5678_9abc_def0);
        let mut map: AxHashMap<&str, u32, AxBuildHasher> = AxHashMap::with_hasher(hasher);
        map.insert("seeded", 7);
        assert_eq!(map["seeded"], 7);
    }

    // ── Type alias: HashMap ───────────────────────────────────────────────────

    #[test]
    fn alias_hashmap_basic() {
        let mut map: HashMap<&str, u32> = HashMap::with_hasher(BuildHasherDefault::default());
        map.insert("hello", 42);
        assert_eq!(map["hello"], 42);
    }

    #[test]
    fn alias_hashmap_collect() {
        let map: HashMap<&str, u32> = [("a", 1u32), ("b", 2), ("c", 3)]
            .into_iter()
            .collect::<RawHashMap<_, _, BuildHasherDefault<AxHasher>>>();
        assert_eq!(map.len(), 3);
    }

    // ── AxHashSet ────────────────────────────────────────────────────────────

    #[test]
    fn set_basic_operations() {
        let mut set: AxHashSet<u32> = AxHashSet::new();
        assert!(set.is_empty());

        set.insert(1);
        set.insert(2);
        set.insert(2); // duplicate
        set.insert(3);

        assert_eq!(set.len(), 3);
        assert!(set.contains(&1));
        assert!(!set.contains(&99));

        set.remove(&2);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn set_with_capacity() {
        let set: AxHashSet<u64> = AxHashSet::with_capacity(64);
        assert!(set.capacity() >= 64);
    }

    #[test]
    fn set_default() {
        let set: AxHashSet<u64> = AxHashSet::default();
        assert!(set.is_empty());
    }

    #[test]
    fn set_from_iterator() {
        let set: AxHashSet<u32> = [1u32, 2, 3, 2, 1].into_iter().collect();
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn set_extend() {
        let mut set: AxHashSet<u32> = AxHashSet::new();
        set.extend([1u32, 2, 3]);
        set.extend([3u32, 4, 5]);
        assert_eq!(set.len(), 5);
    }

    #[test]
    fn set_set_operations() {
        let a: AxHashSet<u32> = [1, 2, 3].into_iter().collect();
        let b: AxHashSet<u32> = [2, 3, 4].into_iter().collect();

        let union: AxHashSet<u32> = a.union(&b).copied().collect();
        assert_eq!(union.len(), 4);

        let inter: AxHashSet<u32> = a.intersection(&b).copied().collect();
        assert_eq!(inter.len(), 2);
    }

    #[test]
    fn set_into_inner_roundtrip() {
        let mut set: AxHashSet<i32> = AxHashSet::new();
        set.insert(42);
        let raw: RawHashSet<i32, BuildHasherDefault<AxHasher>> = set.into_inner();
        assert!(raw.contains(&42));
        let wrapped: AxHashSet<i32> = raw.into();
        assert!(wrapped.contains(&42));
    }

    // ── Type alias: HashSet ───────────────────────────────────────────────────

    #[test]
    fn alias_hashset_basic() {
        let mut set: HashSet<u32> = HashSet::with_hasher(BuildHasherDefault::default());
        set.insert(1);
        set.insert(2);
        set.insert(2);
        assert_eq!(set.len(), 2);
    }
}
