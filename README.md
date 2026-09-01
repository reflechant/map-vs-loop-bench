# When is a loop faster than a HashMap?

Looking up a key in a `HashMap` is *O(1)*. Scanning a `Vec` is *O(n)*.
For a large collection the map wins. For a **small** one, the loop often wins
anyway — hashing and pointer-chasing cost more than walking a few contiguous
words.

This crate measures that crossover.

## Run

```bash
cargo bench --bench ints      # u64 keys
cargo bench --bench strings   # 16-character strings (adds a trie)
cargo bench                   # both ints and strings
```

You get a log–log plot (inlined in Ghostty / Kitty / iTerm; otherwise a PNG
under `target/map-vs-loop-bench/` plus a ranked text line per size).

## What is timed

A collection of **N** random keys is built **once**, then we time a single
membership test (`contains` / `contains_key`). Insert/build cost is not in
the number.

| series | structure | probe |
|---|---|---|
| `linear_mid` | `Vec` | key sitting at index `n/2` (typical hit) |
| `linear_max` | `Vec` | a key that is **not** there (full scan) |
| `hashmap` | `std::HashMap` | a present key, rotating so one hot key cannot win on prediction |
| `btree` | `std::BTreeMap` | same as hashmap |
| `trie` | `qp-trie` | strings only |

`HashMap` uses the default hasher (SipHash). A faster hasher would move its
win to smaller N. A binary heap is not a search tree, so it is not in the
plot — finding an arbitrary key in a heap is *O(n)* with worse locality than
`Vec`.

## How to read the plot

- **X** is N (log). **Y** is nanoseconds per lookup (log).
- The linear series climb; HashMap stays roughly flat.
- Where they cross, the map starts to pay for itself.
- `linear_max` crosses first (a miss already scanned the whole array).
- `linear_mid` crosses later (a hit only walks half the array on average).

Expect HashMap to look *worse* than `Vec` and often `BTreeMap` at tiny N:
SipHash on a `u64` is real work, and the table is not as cache-friendly as a
short array.

## Why the loop is fast when N is small

A `Vec` is one contiguous block. The CPU fetches it in cache-line-sized
chunks and can compare several keys before RAM is involved again. A HashMap
must hash the key, then jump to a slot that may not be in cache. That
overhead is roughly constant; the scan’s cost grows with N. At some N the
lines meet.

Strings add a wrinkle: comparing two strings can stop at the first mismatch,
while HashMap still hashes the whole key. The trie walks byte-by-byte and
can beat HashMap in a middle band of N.

## AI usage disclosure

this project was generated with Grok 4.6 (High)
