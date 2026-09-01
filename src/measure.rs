use std::hint::black_box;
use std::time::{Duration, Instant};

/// Median nanoseconds per call of `f`.
///
/// Warms up, calibrates a batch size so each sample is ~1ms, then takes the
/// median of repeated batches. Construction of inputs belongs *outside* `f`.
pub fn measure_ns(mut f: impl FnMut()) -> f64 {
    let warmup_until = Instant::now() + Duration::from_millis(30);
    while Instant::now() < warmup_until {
        f();
    }

    let mut batch = 1usize;
    loop {
        let t0 = Instant::now();
        for _ in 0..batch {
            f();
        }
        let elapsed = t0.elapsed();
        if elapsed >= Duration::from_millis(1) || batch >= 1_000_000 {
            break;
        }
        batch = batch.saturating_mul(2);
    }

    let mut samples = Vec::with_capacity(40);
    let deadline = Instant::now() + Duration::from_millis(120);
    while Instant::now() < deadline && samples.len() < 40 {
        let t0 = Instant::now();
        for _ in 0..batch {
            f();
        }
        let ns = t0.elapsed().as_secs_f64() * 1e9 / batch as f64;
        samples.push(ns);
    }

    samples.sort_by(|a, b| a.partial_cmp(b).expect("finite sample"));
    samples[samples.len() / 2]
}

/// `black_box` both the needle and the boolean so LLVM cannot constant-fold
/// a lookup whose arguments look invariant.
#[inline]
pub fn black_box_contains(found: bool) {
    black_box(found);
}
