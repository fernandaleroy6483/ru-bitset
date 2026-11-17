# ru-bitset

A compact set of non-negative integers backed by a dynamically sized bit vector.

```rust
use ru_bitset::BitSet;

let mut set = BitSet::new();
set.insert(3);
set.insert(70);
assert!(set.contains(3));
assert!(!set.contains(4));
assert_eq!(set.count(), 2);

let mut other = BitSet::new();
other.insert(70);
other.insert(99);
let both = set.intersection(&other);
assert!(both.contains(70));
assert_eq!(both.count(), 1);
```

## Why this exists

Storing a set of small non-negative integers in a `HashSet<usize>` pays hashing and
allocation costs per element. When the values are dense or bounded, a bit vector is
both smaller and faster: membership is a shift and a mask, and union/intersection are
word-at-a-time operations. The trade-off is that a single large value forces the
backing vector to grow to cover it, so this is not a sparse set for huge indices.

## Edge cases

The backing vector grows to cover the highest inserted index and is never shrunk
automatically. `remove` and `contains` accept any `usize`; querying a value beyond
the current backing length simply returns `false` rather than panicking. `clear`
drops all members and resets the backing length to zero.
