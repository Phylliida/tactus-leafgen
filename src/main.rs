//! Prototype driver: generate leaves and write `.svg` + `.png` pairs.
//!
//! Usage:
//!   cargo run --release                       # default: brochido, closed venation
//!   cargo run --release -- <seed>             # pick a seed
//!   cargo run --release -- <seed> <arch>      # arch = cras | broch | eucamp
//!   cargo run --release -- <seed> <arch> open # open (no reticulation)
//!   cargo run --release -- all                # 3 arches, open, leaf_*.svg/png

use tactus_leafgen::blade::Blade;
use tactus_leafgen::major::{self, MajorParams, SecondaryArch};
use tactus_leafgen::raster;
use tactus_leafgen::rng::Rng;
use tactus_leafgen::svg::{self, RenderOpts};
use tactus_leafgen::venation::{self, AnastomoseParams, MinorParams};

fn generate(seed: u64, arch: SecondaryArch, closed: bool, stem: &str) {
    let blade = Blade::new(10.0, 3.2, 1.6, 2.6);
    let major_params = MajorParams {
        arch,
        ..MajorParams::default()
    };
    let mut veins = major::build_major(&blade, &major_params);

    let mut rng = Rng::new(seed);
    let sources = blade.sample_sources(5000, &mut rng);
    let (_iters, _left) = venation::grow_minor(&mut veins, sources, &MinorParams::default());

    let areoles = if closed {
        venation::anastomose(&mut veins, &AnastomoseParams::default())
    } else {
        0
    };

    let opts = RenderOpts::default();
    std::fs::write(format!("{stem}.svg"), svg::render(&blade, &veins, &opts)).expect("svg");
    let canvas = raster::render(&blade, &veins, &opts);
    if std::env::var("LEAF_ZOOM").is_ok() {
        let (cw, ch) = (canvas.w, canvas.h);
        canvas
            .crop(cw / 8, ch * 9 / 20, cw * 2 / 5, ch / 4)
            .write_png(&format!("{stem}_zoom.png"))
            .expect("zoom png");
    }
    canvas.write_png(&format!("{stem}.png")).expect("png");

    println!(
        "{stem}: {:?} {} seed {:#x} -> {} nodes, {} edges, {} anastomoses",
        arch,
        if closed { "closed" } else { "open" },
        seed,
        veins.nodes.len(),
        veins.edges.len(),
        areoles
    );
}

fn main() {
    let mut args = std::env::args().skip(1);
    let first = args.next();

    if first.as_deref() == Some("all") {
        let seed = 42;
        generate(seed, SecondaryArch::Craspedodromous, false, "leaf_cras");
        generate(seed, SecondaryArch::Brochidodromous, false, "leaf_broch");
        generate(seed, SecondaryArch::Eucamptodromous, false, "leaf_eucamp");
        return;
    }

    let seed: u64 = first.and_then(|s| s.parse().ok()).unwrap_or(0xC0FFEE);
    let arch = match args.next().as_deref() {
        Some("cras") | Some("craspedodromous") => SecondaryArch::Craspedodromous,
        Some("eucamp") | Some("eucamptodromous") => SecondaryArch::Eucamptodromous,
        _ => SecondaryArch::Brochidodromous,
    };
    let closed = args.next().as_deref() != Some("open");
    generate(seed, arch, closed, "leaf");
}
