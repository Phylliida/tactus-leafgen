//! Prototype driver: generate leaves and write `.svg` + `.png` pairs.
//!
//! Usage:
//!   cargo run --release                       # default seed, brochidodromous
//!   cargo run --release -- <seed>             # pick a seed
//!   cargo run --release -- <seed> <arch>      # arch = cras | broch | eucamp
//!   cargo run --release -- <seed> <arch> <out.svg>
//!   cargo run --release -- all                # all three arches → leaf_*.svg/png

use tactus_leafgen::blade::Blade;
use tactus_leafgen::major::{self, MajorParams, SecondaryArch};
use tactus_leafgen::raster;
use tactus_leafgen::rng::Rng;
use tactus_leafgen::svg::{self, RenderOpts};
use tactus_leafgen::venation::{self, MinorParams};

fn generate(seed: u64, arch: SecondaryArch, stem: &str) {
    let blade = Blade::new(10.0, 3.2, 1.6, 2.6);
    let major_params = MajorParams {
        arch,
        ..MajorParams::default()
    };
    let mut veins = major::build_major(&blade, &major_params);

    let mut rng = Rng::new(seed);
    let sources = blade.sample_sources(2200, &mut rng);
    let (iters, leftover) = venation::grow_minor(&mut veins, sources, &MinorParams::default());

    let opts = RenderOpts::default();
    std::fs::write(format!("{stem}.svg"), svg::render(&blade, &veins, &opts)).expect("svg");
    raster::render(&blade, &veins, &opts)
        .write_png(&format!("{stem}.png"))
        .expect("png");

    println!(
        "{stem}: {:?} seed {:#x} -> {} nodes, {} edges ({} minor iters, {} left)",
        arch,
        seed,
        veins.nodes.len(),
        veins.edges.len(),
        iters,
        leftover
    );
}

fn main() {
    let mut args = std::env::args().skip(1);
    let first = args.next();

    if first.as_deref() == Some("all") {
        let seed = 42;
        generate(seed, SecondaryArch::Craspedodromous, "leaf_cras");
        generate(seed, SecondaryArch::Brochidodromous, "leaf_broch");
        generate(seed, SecondaryArch::Eucamptodromous, "leaf_eucamp");
        return;
    }

    let seed: u64 = first.and_then(|s| s.parse().ok()).unwrap_or(0xC0FFEE);
    let arch = match args.next().as_deref() {
        Some("cras") | Some("craspedodromous") => SecondaryArch::Craspedodromous,
        Some("eucamp") | Some("eucamptodromous") => SecondaryArch::Eucamptodromous,
        _ => SecondaryArch::Brochidodromous,
    };
    let stem = args
        .next()
        .map(|o| o.strip_suffix(".svg").unwrap_or(&o).to_string())
        .unwrap_or_else(|| "leaf".to_string());
    generate(seed, arch, &stem);
}
