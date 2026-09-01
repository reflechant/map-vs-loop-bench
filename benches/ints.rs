//! u64 membership: linear scan vs HashMap vs BTreeMap.
//!
//! HashMap uses std's default SipHash hasher. BTreeMap is the tree
//! (a heap is not a search structure).

fn main() {
    map_vs_loop_bench::plot::print_ints();
}
