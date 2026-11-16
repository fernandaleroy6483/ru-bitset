//! A compact set of non-negative integers backed by a dynamically sized bit vector.
//!
//! `BitSet` stores membership in a `Vec<u64>`. Bits are indexed starting at 0.
//! The backing vector grows on demand; it never shrinks automatically.
//!
//! # Examples
//!
//! ```
//! use ru_bitset::BitSet;
//!
//! let mut set = BitSet::new();
//! set.insert(3);
//! set.insert(70);
//! assert!(set.contains(3));
//! assert!(set.contains(70));
//! assert!(!set.contains(4));
//! ```

/// A set of non-negative integers stored as a bit vector.
///
/// The word type is `u64` so that the common case of a few hundred members
/// still fits in a handful of machine words. `Vec<u64>` is used instead of
/// `Vec<bool>` because `Vec<bool>` packs bits and hides the word layout,
/// making bulk operations slower and less obvious.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BitSet {
    words: Vec<u64>,
}

impl BitSet {
    /// Creates an empty set.
    pub fn new() -> Self {
        BitSet { words: Vec::new() }
    }

    /// Returns the number of bits currently addressable by this set.
    ///
    /// The length is a multiple of 64 except when the set is empty.
    pub fn len(&self) -> usize {
        self.words.len() * 64
    }

    /// Returns `true` if the set contains no members.
    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|&w| w == 0)
    }

    /// Inserts `value` into the set.
    ///
    /// The backing vector grows only as far as needed to address `value`.
    pub fn insert(&mut self, value: usize) {
        let word = value / 64;
        let bit = value % 64;
        if word >= self.words.len() {
            self.words.resize(word + 1, 0);
        }
        self.words[word] |= 1u64 << bit;
    }

    /// Removes `value` from the set.
    ///
    /// Returns `true` if the value was present.
    pub fn remove(&mut self, value: usize) -> bool {
        let word = value / 64;
        let bit = value % 64;
        if word >= self.words.len() {
            return false;
        }
        let mask = 1u64 << bit;
        let was_set = self.words[word] & mask != 0;
        self.words[word] &= !mask;
        was_set
    }

    /// Returns `true` if the set contains `value`.
    pub fn contains(&self, value: usize) -> bool {
        let word = value / 64;
        let bit = value % 64;
        word < self.words.len() && (self.words[word] & (1u64 << bit)) != 0
    }

    /// Removes all members, keeping the allocated capacity.
    pub fn clear(&mut self) {
        self.words.clear();
    }

    /// Returns the number of members in the set.
    ///
    /// This counts set bits, not the length of the backing vector.
    pub fn count(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }

    /// Returns a new set containing members present in both sets.
    pub fn intersection(&self, other: &BitSet) -> BitSet {
        let n = self.words.len().min(other.words.len());
        let mut words = Vec::with_capacity(n);
        for i in 0..n {
            words.push(self.words[i] & other.words[i]);
        }
        BitSet { words }
    }

    /// Returns a new set containing members present in either set.
    pub fn union(&self, other: &BitSet) -> BitSet {
        let n = self.words.len().max(other.words.len());
        let mut words = Vec::with_capacity(n);
        for i in 0..n {
            let a = self.words.get(i).copied().unwrap_or(0);
            let b = other.words.get(i).copied().unwrap_or(0);
            words.push(a | b);
        }
        BitSet { words }
    }

    /// Returns a new set containing members present in `self` but not `other`.
    pub fn difference(&self, other: &BitSet) -> BitSet {
        let mut words = Vec::with_capacity(self.words.len());
        for (i, &a) in self.words.iter().enumerate() {
            let b = other.words.get(i).copied().unwrap_or(0);
            words.push(a & !b);
        }
        BitSet { words }
    }
}

#[cfg(test)]
mod tests {
    use super::BitSet;

    #[test]
    fn new_set_is_empty() {
        let set = BitSet::new();
        assert!(set.is_empty());
        assert_eq!(set.count(), 0);
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn insert_and_contains() {
        let mut set = BitSet::new();
        set.insert(0);
        set.insert(63);
        set.insert(64);
        set.insert(1000);
        assert!(set.contains(0));
        assert!(set.contains(63));
        assert!(set.contains(64));
        assert!(set.contains(1000));
        assert!(!set.contains(1));
        assert!(!set.contains(62));
        assert!(!set.contains(999));
        assert_eq!(set.count(), 4);
    }

    #[test]
    fn insert_is_idempotent() {
        let mut set = BitSet::new();
        set.insert(7);
        set.insert(7);
        assert_eq!(set.count(), 1);
    }

    #[test]
    fn remove_reports_presence() {
        let mut set = BitSet::new();
        set.insert(5);
        assert!(set.remove(5));
        assert!(!set.remove(5));
        assert!(!set.remove(6));
        assert!(set.is_empty());
    }

    #[test]
    fn removing_out_of_range_is_false() {
        let mut set = BitSet::new();
        assert!(!set.remove(usize::MAX));
    }

    #[test]
    fn contains_out_of_range_is_false() {
        let set = BitSet::new();
        assert!(!set.contains(usize::MAX));
    }

    #[test]
    fn clear_removes_all_members() {
        let mut set = BitSet::new();
        set.insert(1);
        set.insert(2);
        set.clear();
        assert!(set.is_empty());
        assert_eq!(set.count(), 0);
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn intersection_is_commutative() {
        let mut a = BitSet::new();
        a.insert(1);
        a.insert(2);
        a.insert(3);
        let mut b = BitSet::new();
        b.insert(2);
        b.insert(3);
        b.insert(4);
        let ab = a.intersection(&b);
        let ba = b.intersection(&a);
        assert_eq!(ab, ba);
        assert!(ab.contains(2));
        assert!(ab.contains(3));
        assert!(!ab.contains(1));
        assert!(!ab.contains(4));
        assert_eq!(ab.count(), 2);
    }

    #[test]
    fn union_handles_different_lengths() {
        let mut a = BitSet::new();
        a.insert(1);
        let mut b = BitSet::new();
        b.insert(1000);
        let u = a.union(&b);
        assert!(u.contains(1));
        assert!(u.contains(1000));
        assert_eq!(u.count(), 2);
    }

    #[test]
    fn difference_removes_shared_members() {
        let mut a = BitSet::new();
        a.insert(1);
        a.insert(2);
        let mut b = BitSet::new();
        b.insert(2);
        b.insert(3);
        let d = a.difference(&b);
        assert!(d.contains(1));
        assert!(!d.contains(2));
        assert!(!d.contains(3));
        assert_eq!(d.count(), 1);
    }

    #[test]
    fn operations_with_empty_sets() {
        let mut a = BitSet::new();
        a.insert(10);
        let empty = BitSet::new();
        assert_eq!(a.intersection(&empty).count(), 0);
        assert_eq!(a.union(&empty).count(), 1);
        assert_eq!(a.difference(&empty).count(), 1);
        assert_eq!(empty.difference(&a).count(), 0);
    }

    #[test]
    fn len_reflects_backing_words() {
        let mut set = BitSet::new();
        set.insert(0);
        assert_eq!(set.len(), 64);
        set.insert(64);
        assert_eq!(set.len(), 128);
    }

    #[test]
    fn large_index_does_not_panic() {
        let mut set = BitSet::new();
        let value = 10_000;
        set.insert(value);
        assert!(set.contains(value));
        assert_eq!(set.count(), 1);
    }
}
