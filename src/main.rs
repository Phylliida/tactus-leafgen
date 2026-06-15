//! Prototype driver: generate one leaf and write `leaf.svg`.
//!
//! Usage:
//!   cargo run --release                 # default seed
//!   cargo run --release -- <seed>       # pick a seed
//!   cargo run --release -- <seed> <out.svg>

use tactus_leafgen::blade::Blade;
use tactus_leafgen::raster;
use tactus_leafgen::rng::Rng;
use tactus_leafgen::svg::{self, RenderOpts};
use tactus_leafgen::venation::{self, VenationParams};

fn main() {
    let mut args = std::env::args().skip(1);
    let seed: u64 = args
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0xC0FFEE);
    let out = args.next().unwrap_or_else(|| "leaf.svg".to_string());

    // Ovate blade: widest below the middle (a < b).
    let blade = Blade::new(10.0, 3.2, 1.6, 2.6);

    let mut rng = Rng::new(seed);
    let n_sources = 900;
    let sources = blade.sample_sources(n_sources, &mut rng);

    let params = VenationParams::default();
    let v = venation::grow_open(&blade, sources, &params);

    let svg = svg::render(&blade, &v.nodes, &RenderOpts::default());
    std::fs::write(&out, svg).expect("write svg");

    let png = out.strip_suffix(".svg").unwrap_or(&out).to_string() + ".png";
    raster::render(&blade, &v.nodes, &RenderOpts::default())
        .write_png(&png)
        .expect("write png");

    println!(
        "seed {:#x}: {} sources -> {} vein nodes, {} iters, {} sources left over\n  wrote {} and {}",
        seed,
        n_sources,
        v.nodes.len(),
        v.iters_used,
        v.leftover_sources,
        out,
        png
    );
}
