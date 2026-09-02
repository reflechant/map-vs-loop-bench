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
  btree_max    BTreeMap, rotating absent key       (~O(log n), miss)

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
  btree_max    BTreeMap, rotating absent key       (miss)
  trie         qp-trie, rotating present key

log–log: X = N, Y = ns/lookup.",
        &series,
    );
}

fn print_report(title: &str, png_stem: &str, blurb: &str, series: &[Series]) {
    println!();
    println!("{title}");
    println!("{}", "=".repeat(title.len()));
    println!();
    println!("{blurb}");
    println!();

    print_timing_table(series);
    println!();
    print_crossover_summary(series);
    println!();
    print_ranking(series);
    println!();

    if let Err(e) = export_csv(png_stem, series) {
        eprintln!("  (csv export note: {e})");
    }

    let path = png_path(png_stem);
    match render_png(&path, title, series) {
        Ok(()) if term_image::try_show_png(&path) => {}
        Ok(()) => {
            println!("  (Chart generated at: {})", path.display());
        }
        Err(e) => {
            eprintln!("  (png failed: {e})");
        }
    }
    println!();
}

fn print_timing_table(series: &[Series]) {
    print!("| {:>6} ", "N");
    for s in series {
        print!("| {:>12} ", s.name);
    }
    println!("| {:>14} | {:>16} |", "Fastest", "Linear/HashMap");

    print!("|{:->8}", "");
    for _ in series {
        print!("|{:->14}", "");
    }
    println!("|{:->16}|{:->18}|", "", "");

    for (i, &n) in SIZES.iter().enumerate() {
        print!("| {:>6} ", compact_n(n));
        let mut min_val = f64::INFINITY;
        let mut min_name = "";
        let mut linear_mid_ns = 0.0;
        let mut hashmap_ns = 0.0;

        for s in series {
            let val = s.ns[i];
            if val < min_val {
                min_val = val;
                min_name = s.name;
            }
            if s.name == "linear_mid" {
                linear_mid_ns = val;
            } else if s.name == "hashmap" {
                hashmap_ns = val;
            }
            print!("| {:>10.2} ns ", val);
        }

        let ratio_str = if hashmap_ns > 0.0 && linear_mid_ns > 0.0 {
            if linear_mid_ns <= hashmap_ns {
                format!("{:.2}x faster", hashmap_ns / linear_mid_ns)
            } else {
                format!("{:.2}x slower", linear_mid_ns / hashmap_ns)
            }
        } else {
            "-".to_string()
        };

        println!("| {:>14} | {:>16} |", min_name, ratio_str);
    }
}

fn print_crossover_summary(series: &[Series]) {
    let mut linear_mid_max_win_n = None;
    let mut linear_max_max_win_n = None;

    for (i, &n) in SIZES.iter().enumerate() {
        let mid = series.iter().find(|s| s.name == "linear_mid").map(|s| s.ns[i]);
        let max = series.iter().find(|s| s.name == "linear_max").map(|s| s.ns[i]);
        let hash = series.iter().find(|s| s.name == "hashmap").map(|s| s.ns[i]);

        if let (Some(m), Some(h)) = (mid, hash) {
            if m < h {
                linear_mid_max_win_n = Some(n);
            }
        }
        if let (Some(m), Some(h)) = (max, hash) {
            if m < h {
                linear_max_max_win_n = Some(n);
            }
        }
    }

    println!("--- Crossover Summary vs HashMap ---");
    if let Some(n) = linear_mid_max_win_n {
        println!("  - Linear scan (hit @ n/2) is faster than HashMap up to N = {n}");
    } else {
        println!("  - HashMap is faster than linear hit even at N = 1");
    }
    if let Some(n) = linear_max_max_win_n {
        println!("  - Linear scan (miss / full scan) is faster than HashMap up to N = {n}");
    } else {
        println!("  - HashMap is faster than linear full scan at all measured sizes");
    }
}

fn export_csv(stem: &str, series: &[Series]) -> std::io::Result<()> {
    let target_dir = std::env::var("BENCH_CSV_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("results/data"));
    std::fs::create_dir_all(&target_dir)?;

    let path = target_dir.join(format!("{stem}.csv"));
    let mut w = String::new();
    w.push_str("N");
    for s in series {
        w.push(',');
        w.push_str(s.name);
    }
    w.push('\n');

    for (i, &n) in SIZES.iter().enumerate() {
        w.push_str(&n.to_string());
        for s in series {
            w.push(',');
            w.push_str(&format!("{:.4}", s.ns[i]));
        }
        w.push('\n');
    }

    std::fs::write(&path, w)?;
    println!("  (Raw data saved to: {})", path.display());
    Ok(())
}

/// Ranked times at each N: `N=4    linear_mid (2.1 ns)  >  btree (4.0 ns)  >  hashmap (13 ns)`
fn print_ranking(series: &[Series]) {
    println!("--- Ranked by Speed (Fastest > Slowest) ---");
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
    let mut btree_max = Vec::with_capacity(SIZES.len());

    for &n in SIZES {
        let data = IntData::generate(n);
        let map = data::int_hashmap(&data.keys);
        let tree = data::int_btreemap(&data.keys);
        let mid = data.mid_key();
        let missing = data.missing;
        let keys = data.keys;
        let missing_keys = data.missing_keys;

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
        btree_max.push(measure_ns(|| {
            let k = missing_keys[i % missing_keys.len()];
            i = i.wrapping_add(1);
            black_box_contains(tree.contains_key(&black_box(k)));
        }));
    }

    vec![
        // matplotlib tab10
        series("linear_mid", [44, 160, 44], linear_mid),
        series("linear_max", [214, 39, 40], linear_max),
        series("hashmap", [31, 119, 180], hashmap),
        series("btree_max", [179, 88, 6], btree_max),
    ]
}

fn measure_string_series() -> Vec<Series> {
    let mut linear_mid = Vec::with_capacity(SIZES.len());
    let mut linear_max = Vec::with_capacity(SIZES.len());
    let mut hashmap = Vec::with_capacity(SIZES.len());
    let mut btree_max = Vec::with_capacity(SIZES.len());
    let mut trie = Vec::with_capacity(SIZES.len());

    for &n in SIZES {
        let data = StringData::generate(n);
        let map = data::string_hashmap(&data.keys);
        let tree = data::string_btreemap(&data.keys);
        let qp = data::string_trie(&data.keys);
        let mid = data.mid_key().to_owned();
        let missing = data.missing.clone();
        let keys = data.keys;
        let missing_keys = data.missing_keys;

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
        btree_max.push(measure_ns(|| {
            let k = missing_keys[i % missing_keys.len()].as_str();
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
        series("btree_max", [179, 88, 6], btree_max),
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
