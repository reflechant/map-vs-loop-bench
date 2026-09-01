use std::hint::black_box;

use crate::chart::{png_path, render_png};
use crate::data::{self, IntData, SIZES, StringData, linear_contains_str, linear_contains_u64};
use crate::measure::{black_box_contains, measure_ns};
use crate::term_image;

#[derive(Clone)]
pub struct Series {
    pub name: &'static str,
    pub color: [u8; 3],
    /// Median ns per lookup, aligned with `SIZES`.
    pub ns: Vec<f64>,
}

pub fn print_ints() {
    eprintln!("Measuring lookup times for the plot...");
    let series = measure_int_series();
    print_report(
        "u64 membership lookup",
        "ints-lookup",
        "\
Prebuilt collections of N random u64 keys. Timed: one contains / contains_key.
Construction is not timed. HashMap uses std SipHash.

  linear_mid   Vec scan, hit at index n/2          (~n/2 compares)
  linear_max   Vec scan, key absent                (full scan)
  hashmap      HashMap, rotating present key       (~O(1))
  btree        BTreeMap, rotating present key      (~O(log n))

log–log: X = N, Y = ns/lookup. Linear climbs; HashMap is the flat line.",
        &series,
    );
}

pub fn print_strings() {
    eprintln!("Measuring lookup times for the plot...");
    let series = measure_string_series();
    print_report(
        "string membership lookup (len=16)",
        "strings-lookup",
        "\
Prebuilt collections of N random 16-char keys. Timed: one contains / contains_key.
Construction is not timed. HashMap uses std SipHash. Trie is qp-trie.

  linear_mid   Vec scan, hit at index n/2
  linear_max   Vec scan, key absent
  hashmap      HashMap, rotating present key
  btree        BTreeMap, rotating present key
  trie         qp-trie, rotating present key

log–log: X = N, Y = ns/lookup.",
        &series,
    );
}

fn print_report(title: &str, png_stem: &str, blurb: &str, series: &[Series]) {
    println!();
    println!("{title}");
    println!();
    println!("{blurb}");
    println!();

    let path = png_path(png_stem);
    match render_png(&path, title, series) {
        Ok(()) if term_image::try_show_png(&path) => {}
        Ok(()) => {
            print_ranking(series);
            println!("  ({})", path.display());
        }
        Err(e) => {
            print_ranking(series);
            eprintln!("  (png failed: {e})");
        }
    }
    println!();
}

/// Ranked times at each N: `N=4    linear_mid (2.1 ns)  >  btree (4.0 ns)  >  hashmap (13 ns)`
fn print_ranking(series: &[Series]) {
    for (i, &n) in SIZES.iter().enumerate() {
        let mut rows: Vec<(&str, f64)> = series.iter().map(|s| (s.name, s.ns[i])).collect();
        rows.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let chain = rows
            .iter()
            .map(|(name, ns)| format!("{name} ({})", compact_ns(*ns)))
            .collect::<Vec<_>>()
            .join("  >  ");
        println!("  N={:<5}  {chain}", compact_n(n));
    }
}

fn measure_int_series() -> Vec<Series> {
    let mut linear_mid = Vec::with_capacity(SIZES.len());
    let mut linear_max = Vec::with_capacity(SIZES.len());
    let mut hashmap = Vec::with_capacity(SIZES.len());
    let mut btree = Vec::with_capacity(SIZES.len());

    for &n in SIZES {
        let data = IntData::generate(n);
        let map = data::int_hashmap(&data.keys);
        let tree = data::int_btreemap(&data.keys);
        let mid = data.mid_key();
        let missing = data.missing;
        let keys = data.keys;

        linear_mid.push(measure_ns(|| {
            black_box_contains(linear_contains_u64(&keys, black_box(mid)));
        }));
        linear_max.push(measure_ns(|| {
            black_box_contains(linear_contains_u64(&keys, black_box(missing)));
        }));

        let mut i = 0usize;
        hashmap.push(measure_ns(|| {
            let k = keys[i % keys.len()];
            i = i.wrapping_add(1);
            black_box_contains(map.contains_key(&black_box(k)));
        }));

        let mut i = 0usize;
        btree.push(measure_ns(|| {
            let k = keys[i % keys.len()];
            i = i.wrapping_add(1);
            black_box_contains(tree.contains_key(&black_box(k)));
        }));
    }

    vec![
        // matplotlib tab10
        series("linear_mid", [44, 160, 44], linear_mid),
        series("linear_max", [214, 39, 40], linear_max),
        series("hashmap", [31, 119, 180], hashmap),
        series("btree", [255, 127, 14], btree),
    ]
}

fn measure_string_series() -> Vec<Series> {
    let mut linear_mid = Vec::with_capacity(SIZES.len());
    let mut linear_max = Vec::with_capacity(SIZES.len());
    let mut hashmap = Vec::with_capacity(SIZES.len());
    let mut btree = Vec::with_capacity(SIZES.len());
    let mut trie = Vec::with_capacity(SIZES.len());

    for &n in SIZES {
        let data = StringData::generate(n);
        let map = data::string_hashmap(&data.keys);
        let tree = data::string_btreemap(&data.keys);
        let qp = data::string_trie(&data.keys);
        let mid = data.mid_key().to_owned();
        let missing = data.missing.clone();
        let keys = data.keys;

        linear_mid.push(measure_ns(|| {
            black_box_contains(linear_contains_str(&keys, black_box(mid.as_str())));
        }));
        linear_max.push(measure_ns(|| {
            black_box_contains(linear_contains_str(&keys, black_box(missing.as_str())));
        }));

        let mut i = 0usize;
        hashmap.push(measure_ns(|| {
            let k = keys[i % keys.len()].as_str();
            i = i.wrapping_add(1);
            black_box_contains(map.contains_key(black_box(k)));
        }));

        let mut i = 0usize;
        btree.push(measure_ns(|| {
            let k = keys[i % keys.len()].as_str();
            i = i.wrapping_add(1);
            black_box_contains(tree.contains_key(black_box(k)));
        }));

        let mut i = 0usize;
        trie.push(measure_ns(|| {
            let k = keys[i % keys.len()].as_str();
            i = i.wrapping_add(1);
            black_box_contains(qp.contains_key_str(black_box(k)));
        }));
    }

    vec![
        series("linear_mid", [44, 160, 44], linear_mid),
        series("linear_max", [214, 39, 40], linear_max),
        series("hashmap", [31, 119, 180], hashmap),
        series("btree", [255, 127, 14], btree),
        series("trie", [148, 103, 189], trie),
    ]
}

fn series(name: &'static str, color: [u8; 3], ns: Vec<f64>) -> Series {
    Series { name, color, ns }
}

pub fn compact_n(n: usize) -> String {
    if n >= 1024 && n.is_multiple_of(1024) {
        format!("{}k", n / 1024)
    } else {
        format!("{n}")
    }
}

pub fn compact_ns(ns: f64) -> String {
    if ns < 10.0 {
        format!("{ns:.2} ns")
    } else if ns < 100.0 {
        format!("{ns:.1} ns")
    } else if ns < 1000.0 {
        format!("{ns:.0} ns")
    } else if ns < 10_000.0 {
        format!("{:.2} µs", ns / 1e3)
    } else if ns < 1_000_000.0 {
        format!("{:.1} µs", ns / 1e3)
    } else {
        format!("{:.2} ms", ns / 1e6)
    }
}
