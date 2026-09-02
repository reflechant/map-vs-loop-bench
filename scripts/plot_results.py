#!/usr/bin/env python3
"""
Generate publication-quality static benchmark plots from CSV data using matplotlib and seaborn.
Supports individual benchmark plots (ints, strings) for both Laptop (x86_64) and Raspberry Pi 4 (aarch64),
as well as cross-platform comparison charts.
"""

import argparse
from pathlib import Path
import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns


def setup_style():
    """Configure clean matplotlib and seaborn styling."""
    sns.set_theme(style="whitegrid")
    plt.rcParams.update({
        "font.sans-serif": ["DejaVu Sans", "Liberation Sans", "Helvetica", "Arial", "sans-serif"],
        "font.size": 11,
        "axes.titlesize": 13,
        "axes.titleweight": "bold",
        "axes.labelsize": 12,
        "axes.labelweight": "semibold",
        "xtick.labelsize": 10,
        "ytick.labelsize": 10,
        "legend.fontsize": 10,
        "figure.titlesize": 15,
        "figure.dpi": 300,
        "savefig.dpi": 300,
        "savefig.bbox": "tight",
    })


COLOR_MAP = {
    "linear_mid": "#2ca02c",   # Green
    "linear_max": "#d62728",   # Red
    "hashmap": "#1f77b4",      # Blue
    "btree": "#ff7f0e",        # Orange
    "btree_max": "#b35806",    # Dark Amber / Rust
    "trie": "#9467bd",         # Purple
}

LABEL_MAP = {
    "linear_mid": "Linear scan (hit @ n/2)",
    "linear_max": "Linear scan (miss / full scan)",
    "hashmap": "std::HashMap (SipHash)",
    "btree": "std::BTreeMap",
    "btree_max": "std::BTreeMap (miss / max)",
    "trie": "qp-trie",
}

MARKER_MAP = {
    "linear_mid": "o",
    "linear_max": "s",
    "hashmap": "^",
    "btree": "D",
    "btree_max": "d",
    "trie": "v",
}


import matplotlib.ticker as ticker


def format_n_tick(x, pos):
    if x < 1:
        return ""
    x_int = int(round(x))
    if x_int >= 1024 and x_int % 1024 == 0:
        return f"{x_int // 1024}k"
    return str(x_int)


def plot_single_bench(csv_path: Path, output_png: Path, title: str, subtitle: str = ""):
    """Plot single benchmark log-log chart from CSV."""
    df = pd.read_csv(csv_path)
    
    fig, ax = plt.subplots(figsize=(10, 6))

    columns = [col for col in df.columns if col not in ("N", "btree_mid")]
    
    for col in columns:
        color = COLOR_MAP.get(col, "#333333")
        label = LABEL_MAP.get(col, col)
        marker = MARKER_MAP.get(col, "o")
        ax.plot(
            df["N"],
            df[col],
            label=label,
            color=color,
            marker=marker,
            markersize=6,
            linewidth=2.2,
            alpha=0.95,
        )

    ax.set_xscale("log", base=2)
    ax.set_yscale("log")
    
    ax.set_xlabel("Collection Size N (elements)")
    ax.set_ylabel("Lookup Duration (ns / lookup)")
    
    full_title = title if not subtitle else f"{title}\n{subtitle}"
    ax.set_title(full_title, pad=15)
    
    ticks = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096]
    ax.set_xticks(ticks)
    ax.xaxis.set_major_formatter(ticker.FuncFormatter(format_n_tick))
    
    ax.grid(True, which="major", linestyle="--", linewidth=0.8, alpha=0.7)
    ax.grid(True, which="minor", linestyle=":", linewidth=0.5, alpha=0.3)
    
    ax.legend(frameon=True, facecolor="white", edgecolor="#cccccc", framealpha=0.9, loc="upper left")
    
    output_png.parent.mkdir(parents=True, exist_ok=True)
    plt.tight_layout()
    plt.savefig(output_png)
    plt.close()
    print(f"Generated chart: {output_png}")


def plot_comparison(
    laptop_csv: Path,
    rpi_csv: Path,
    output_png: Path,
    title: str,
):
    """Plot side-by-side comparison between Laptop and RPi 4."""
    df_laptop = pd.read_csv(laptop_csv)
    df_rpi = pd.read_csv(rpi_csv)
    
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 6), sharey=True)
    
    columns = [col for col in df_laptop.columns if col not in ("N", "btree_mid")]
    ticks = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096]
    
    # Laptop plot
    for col in columns:
        color = COLOR_MAP.get(col, "#333333")
        label = LABEL_MAP.get(col, col)
        marker = MARKER_MAP.get(col, "o")
        ax1.plot(
            df_laptop["N"],
            df_laptop[col],
            label=label,
            color=color,
            marker=marker,
            markersize=6,
            linewidth=2.2,
        )
    ax1.set_xscale("log", base=2)
    ax1.set_yscale("log")
    ax1.set_xticks(ticks)
    ax1.xaxis.set_major_formatter(ticker.FuncFormatter(format_n_tick))
    ax1.set_xlabel("Collection Size N (elements)")
    ax1.set_ylabel("Lookup Duration (ns / lookup)")
    ax1.set_title("Laptop (Intel Core i5-1135G7, 4C/8T @ 2.4–4.2 GHz, x86_64)", fontsize=11, fontweight="bold")
    ax1.grid(True, which="major", linestyle="--", alpha=0.7)
    ax1.grid(True, which="minor", linestyle=":", alpha=0.3)
    ax1.legend(frameon=True, loc="upper left")

    # RPi plot
    rpi_columns = [col for col in df_rpi.columns if col not in ("N", "btree_mid")]
    for col in rpi_columns:
        color = COLOR_MAP.get(col, "#333333")
        label = LABEL_MAP.get(col, col)
        marker = MARKER_MAP.get(col, "o")
        ax2.plot(
            df_rpi["N"],
            df_rpi[col],
            label=label,
            color=color,
            marker=marker,
            markersize=6,
            linewidth=2.2,
        )
    ax2.set_xscale("log", base=2)
    ax2.set_yscale("log")
    ax2.set_xticks(ticks)
    ax2.xaxis.set_major_formatter(ticker.FuncFormatter(format_n_tick))
    ax2.set_xlabel("Collection Size N (elements)")
    ax2.set_title("Raspberry Pi 4 Model B (Broadcom BCM2711, 4× Cortex-A72 @ 1.8 GHz, aarch64)", fontsize=11, fontweight="bold")
    ax2.grid(True, which="major", linestyle="--", alpha=0.7)
    ax2.grid(True, which="minor", linestyle=":", alpha=0.3)
    ax2.legend(frameon=True, loc="upper left")
    
    fig.suptitle(f"{title} — Cross-Platform Comparison", fontsize=15, fontweight="bold", y=1.02)
    plt.tight_layout()
    output_png.parent.mkdir(parents=True, exist_ok=True)
    plt.savefig(output_png)
    plt.close()
    print(f"Generated comparison chart: {output_png}")


def main():
    parser = argparse.ArgumentParser(description="Generate benchmark plots from CSV data.")
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parent.parent)
    args = parser.parse_args()
    
    setup_style()
    root = args.repo_root
    
    # Laptop plots
    laptop_ints = root / "results" / "laptop" / "ints-lookup.csv"
    laptop_strings = root / "results" / "laptop" / "strings-lookup.csv"
    if laptop_ints.exists():
        plot_single_bench(
            laptop_ints,
            root / "results" / "laptop" / "ints-lookup.png",
            "u64 Key Membership Lookup",
            "Laptop (Intel Core i5-1135G7, 4C/8T @ 2.4–4.2 GHz, x86_64)",
        )
    if laptop_strings.exists():
        plot_single_bench(
            laptop_strings,
            root / "results" / "laptop" / "strings-lookup.png",
            "16-Char String Key Membership Lookup",
            "Laptop (Intel Core i5-1135G7, 4C/8T @ 2.4–4.2 GHz, x86_64)",
        )

    # RPi 4 plots
    rpi_ints = root / "results" / "rpi4" / "ints-lookup.csv"
    rpi_strings = root / "results" / "rpi4" / "strings-lookup.csv"
    if rpi_ints.exists():
        plot_single_bench(
            rpi_ints,
            root / "results" / "rpi4" / "ints-lookup.png",
            "u64 Key Membership Lookup",
            "Raspberry Pi 4 Model B (Broadcom BCM2711, 4× ARM Cortex-A72 @ 1.8 GHz, aarch64)",
        )
    if rpi_strings.exists():
        plot_single_bench(
            rpi_strings,
            root / "results" / "rpi4" / "strings-lookup.png",
            "16-Char String Key Membership Lookup",
            "Raspberry Pi 4 Model B (Broadcom BCM2711, 4× ARM Cortex-A72 @ 1.8 GHz, aarch64)",
        )

    # Cross-platform comparison plots
    if laptop_ints.exists() and rpi_ints.exists():
        plot_comparison(
            laptop_ints,
            rpi_ints,
            root / "results" / "comparison-ints.png",
            "u64 Key Membership Lookup",
        )
    if laptop_strings.exists() and rpi_strings.exists():
        plot_comparison(
            laptop_strings,
            rpi_strings,
            root / "results" / "comparison-strings.png",
            "16-Char String Key Membership Lookup",
        )


if __name__ == "__main__":
    main()
