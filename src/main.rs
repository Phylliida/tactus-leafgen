//! Prototype driver: generate leaves and write `.svg` + `.png` pairs.
//!
//! Usage:
//!   cargo run --release                  # default ovate + serrate, closed venation
//!   cargo run --release -- shapes        # ovate/obovate/elliptic/lanceolate
//!   cargo run --release -- margins       # entire/serrate/dentate/crenate (ovate)
//!   cargo run --release -- arches        # cras/broch/eucamp (open venation)
//!   cargo run --release -- <seed> <arch> # one leaf; arch = cras|broch|eucamp
//!
//! Set LEAF_ZOOM=1 to also dump a native-res areole crop per leaf.

use tactus_leafgen::blade::{Blade, Margin};
use tactus_leafgen::major::{self, MajorParams, SecondaryArch};
use tactus_leafgen::raster;
use tactus_leafgen::rng::Rng;
use tactus_leafgen::svg::{self, RenderOpts};
use tactus_leafgen::venation::{self, AnastomoseParams, MinorParams};

fn generate(seed: u64, blade: &Blade, arch: SecondaryArch, closed: bool, stem: &str) {
    let major_params = MajorParams { arch, ..MajorParams::default() };
    let mut veins = major::build_major(blade, &major_params);

    let mut rng = Rng::new(seed);
    let sources = blade.sample_sources(5000, &mut rng);
    venation::grow_minor(&mut veins, sources, &MinorParams::default());
    let areoles = if closed {
        venation::anastomose(&mut veins, &AnastomoseParams::default())
    } else {
        0
    };

    let opts = RenderOpts::default();
    std::fs::write(format!("{stem}.svg"), svg::render(blade, &veins, &opts)).expect("svg");
    let canvas = raster::render(blade, &veins, &opts);
    if std::env::var("LEAF_ZOOM").is_ok() {
        let (cw, ch) = (canvas.w, canvas.h);
        canvas
            .crop(cw / 8, ch * 9 / 20, cw * 2 / 5, ch / 4)
            .write_png(&format!("{stem}_zoom.png"))
            .expect("zoom png");
    }
    canvas.write_png(&format!("{stem}.png")).expect("png");

    println!(
        "{stem}: {:?} {} -> {} nodes, {} edges, {} areoles",
        arch,
        if closed { "closed" } else { "open" },
        veins.nodes.len(),
        veins.edges.len(),
        areoles
    );
}

fn main() {
    let mut args = std::env::args().skip(1);
    let first = args.next();
    let seed = 42;

    match first.as_deref() {
        Some("shapes") => {
            for (name, blade) in [
                ("leaf_ovate", Blade::ovate()),
                ("leaf_obovate", Blade::obovate()),
                ("leaf_elliptic", Blade::elliptic()),
                ("leaf_lanceolate", Blade::lanceolate()),
            ] {
                generate(seed, &blade, SecondaryArch::Brochidodromous, true, name);
            }
        }
        Some("margins") => {
            for (name, m) in [
                ("leaf_entire", Margin::entire()),
                ("leaf_serrate", Margin::serrate()),
                ("leaf_dentate", Margin::dentate()),
                ("leaf_crenate", Margin::crenate()),
            ] {
                let arch = if m.kind == tactus_leafgen::blade::MarginType::Serrate
                    || m.kind == tactus_leafgen::blade::MarginType::Dentate
                {
                    SecondaryArch::Craspedodromous // toothed leaves usually run veins to teeth
                } else {
                    SecondaryArch::Brochidodromous
                };
                generate(seed, &Blade::ovate().with_margin(m), arch, true, name);
            }
        }
        Some("arches") => {
            generate(seed, &Blade::ovate(), SecondaryArch::Craspedodromous, false, "leaf_cras");
            generate(seed, &Blade::ovate(), SecondaryArch::Brochidodromous, false, "leaf_broch");
            generate(seed, &Blade::ovate(), SecondaryArch::Eucamptodromous, false, "leaf_eucamp");
        }
        other => {
            let seed: u64 = other.and_then(|s| s.parse().ok()).unwrap_or(0xC0FFEE);
            let arch = match args.next().as_deref() {
                Some("cras") => SecondaryArch::Craspedodromous,
                Some("eucamp") => SecondaryArch::Eucamptodromous,
                _ => SecondaryArch::Brochidodromous,
            };
            let blade = Blade::ovate().with_margin(Margin::serrate());
            generate(seed, &blade, arch, true, "leaf");
        }
    }
}
