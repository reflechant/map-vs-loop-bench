//! Matplotlib-style log–log chart via plotters.
//!
//! Text uses Noto Sans from the `ttf-noto-sans` crate (ab_glyph), so we do not
//! link fontconfig/freetype.

use std::error::Error;
use std::path::Path;
use std::sync::Once;

use plotters::prelude::*;
use plotters::style::Color;

use crate::data::SIZES;
use crate::plot::{Series, compact_n, compact_ns};

const W: u32 = 1400;
const H: u32 = 820;

fn ensure_font() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        if plotters::style::register_font("sans-serif", FontStyle::Normal, ttf_noto_sans::REGULAR)
            .is_err()
        {
            panic!("Noto Sans Regular should parse");
        }
    });
}

pub fn render_png(path: &Path, title: &str, series: &[Series]) -> Result<(), Box<dyn Error>> {
    ensure_font();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let xmin = *SIZES.first().unwrap() as f64;
    let xmax = *SIZES.last().unwrap() as f64;
    let (ymin, ymax) = ns_range(series);

    let root = BitMapBackend::new(path, (W, H)).into_drawing_area();
    root.fill(&RGBColor(255, 255, 255))?;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 28))
        .margin(18)
        .x_label_area_size(52)
        .y_label_area_size(78)
        .build_cartesian_2d((xmin..xmax).log_scale(), (ymin..ymax).log_scale())?;

    chart
        .configure_mesh()
        .x_desc("N (elements)")
        .y_desc("time per lookup")
        .axis_desc_style(("sans-serif", 18))
        .label_style(("sans-serif", 14))
        .x_labels(10)
        .y_labels(8)
        .x_label_formatter(&|&n| compact_n(n.round() as usize))
        .y_label_formatter(&|&ns| compact_ns(ns))
        .light_line_style(RGBColor(230, 230, 230).filled())
        .bold_line_style(RGBColor(200, 200, 200).filled())
        .axis_style(RGBColor(80, 80, 80))
        .draw()?;

    for s in series {
        let color = RGBColor(s.color[0], s.color[1], s.color[2]);
        let points: Vec<(f64, f64)> = SIZES
            .iter()
            .zip(s.ns.iter())
            .map(|(&n, &ns)| (n as f64, ns))
            .collect();

        chart
            .draw_series(LineSeries::new(
                points.iter().copied(),
                color.stroke_width(3),
            ))?
            .label(s.name)
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 22, y)], color.stroke_width(3))
            });

        chart.draw_series(
            points
                .iter()
                .copied()
                .map(|(x, y)| Circle::new((x, y), 5, color.filled())),
        )?;
    }

    chart
        .configure_series_labels()
        .background_style(RGBColor(255, 255, 255).mix(0.92).filled())
        .border_style(RGBColor(180, 180, 180))
        .label_font(("sans-serif", 15))
        .position(SeriesLabelPosition::UpperLeft)
        .margin(8)
        .draw()?;

    root.present()?;
    Ok(())
}

pub fn png_path(stem: &str) -> std::path::PathBuf {
    Path::new("target")
        .join("map-vs-loop-bench")
        .join(format!("{stem}.png"))
}

fn ns_range(series: &[Series]) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = 0.0f64;
    for s in series {
        for &ns in &s.ns {
            lo = lo.min(ns);
            hi = hi.max(ns);
        }
    }
    ((lo * 0.65).max(0.4), hi * 1.35)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::SIZES;
    use crate::plot::Series;

    #[test]
    fn writes_png() {
        let ns = SIZES.iter().map(|&n| n as f64).collect();
        let series = [Series {
            name: "demo",
            color: [31, 119, 180],
            ns,
        }];
        let path = std::env::temp_dir().join("map-vs-loop-bench-test.png");
        render_png(&path, "demo", &series).unwrap();
        assert!(path.metadata().unwrap().len() > 1_000);
    }
}
