//! String (16-char) membership: linear scan vs HashMap vs BTreeMap vs qp-trie.

fn main() {
    map_vs_loop_bench::plot::print_strings();
}
