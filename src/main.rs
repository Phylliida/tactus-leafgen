//! Prototype driver: generate leaves and write `.svg` + `.png` pairs.
//!
//! Usage:
//!   cargo run --release                  # default ovate + serrate, closed venation
//!   cargo run --release -- shapes        # ovate/obovate/elliptic/lanceolate
//!   cargo run --release -- margins       # entire/serrate/dentate/crenate (ovate)
//!   cargo run --release -- oak           # pinnately lobed
//!   cargo run --release -- maple         # palmately lobed
//!   cargo run --release -- arches        # cras/broch/eucamp (open venation)
//!   cargo run --release -- <seed> <arch> # one leaf; arch = cras|broch|eucamp
//!
//! Set LEAF_ZOOM=1 to also dump a native-res areole crop per leaf.

use tactus_leafgen::blade::{Blade, Margin};
use tactus_leafgen::compound;
use tactus_leafgen::flower::{self, FlowerParams};
use tactus_leafgen::major::{self, MajorParams, SecondaryArch};
use tactus_leafgen::ginkgo;
use tactus_leafgen::monocot;
use tactus_leafgen::peltate;
use tactus_leafgen::palmate::{self, PalmateBlade};
use tactus_leafgen::raster;
use tactus_leafgen::rng::Rng;
use tactus_leafgen::svg::{self, RenderOpts};
use tactus_leafgen::vec2::Vec2;
use tactus_leafgen::venation::{self, AnastomoseParams, MinorParams, VeinGraph};

fn render_scene(stem: &str, scene: &svg::Scene) {
    let opts = RenderOpts::default();
    std::fs::write(format!("{stem}.svg"), svg::render(scene, &opts)).expect("svg");
    let canvas = raster::render(scene, &opts);
    if std::env::var("LEAF_ZOOM").is_ok() {
        let (cw, ch) = (canvas.w, canvas.h);
        canvas
            .crop(cw / 8, ch * 9 / 20, cw * 2 / 5, ch / 4)
            .write_png(&format!("{stem}_zoom.png"))
            .expect("zoom png");
    }
    canvas.write_png(&format!("{stem}.png")).expect("png");
    println!("{stem}: {} nodes, {} edges", scene.veins.nodes.len(), scene.veins.edges.len());
}

/// Render a green leaf from its outline polygons + veins.
fn finish(stem: &str, laminae: Vec<Vec<Vec2>>, veins: VeinGraph, petiole_len: f64) {
    render_scene(stem, &svg::Scene::leaf(laminae, veins, petiole_len));
}

fn generate(seed: u64, blade: &Blade, arch: SecondaryArch, closed: bool, stem: &str) {
    let major_params = MajorParams { arch, ..MajorParams::default() };
    let mut veins = major::build_major(blade, &major_params);

    let mut rng = Rng::new(seed);
    let sources = blade.sample_sources(5000, &mut rng);
    venation::grow_minor(&mut veins, sources, &MinorParams::default());
    if closed {
        venation::anastomose(&mut veins, &AnastomoseParams::default());
    }

    let petiole_len = blade.length * RenderOpts::default().petiole_frac;
    finish(stem, vec![blade.outline(900)], veins, petiole_len);
}

fn generate_palmate(seed: u64, blade: &PalmateBlade, stem: &str) {
    let (outline, veins, petiole_len) = palmate::assemble_palmate(blade, seed, 1.0, 800);
    finish(stem, vec![outline], veins, petiole_len);
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
            use tactus_leafgen::blade::MarginType;
            for (name, m) in [
                ("leaf_entire", Margin::entire()),
                ("leaf_serrate", Margin::serrate()),
                ("leaf_dentate", Margin::dentate()),
                ("leaf_crenate", Margin::crenate()),
            ] {
                let arch = if m.kind == MarginType::Serrate || m.kind == MarginType::Dentate {
                    SecondaryArch::Craspedodromous
                } else {
                    SecondaryArch::Brochidodromous
                };
                generate(seed, &Blade::ovate().with_margin(m), arch, true, name);
            }
        }
        Some("oak") => {
            generate(seed, &Blade::oak(), SecondaryArch::Craspedodromous, true, "leaf_oak");
        }
        Some("maple") => {
            generate_palmate(seed, &PalmateBlade::maple(), "leaf_maple");
        }
        Some("ash") => {
            let leaf = compound::pinnately_compound(seed, 5, 1.0);
            finish("leaf_ash", leaf.laminae, leaf.veins, leaf.petiole_len);
        }
        Some("horsechestnut") => {
            let leaf = compound::palmately_compound(seed, 7, 115.0, 1.0);
            finish("leaf_horsechestnut", leaf.laminae, leaf.veins, leaf.petiole_len);
        }
        Some("clover") => {
            let leaf = compound::palmately_compound(seed, 3, 38.0, 1.0);
            finish("leaf_clover", leaf.laminae, leaf.veins, leaf.petiole_len);
        }
        Some("grass") => {
            let (ol, v, pl) = monocot::build_monocot_venation(&monocot::MonocotBlade::grass(), 11, 6);
            finish("leaf_grass", vec![ol], v, pl);
        }
        Some("lily") => {
            let (ol, v, pl) = monocot::build_monocot_venation(&monocot::MonocotBlade::lily(), 15, 4);
            finish("leaf_lily", vec![ol], v, pl);
        }
        Some("ginkgo") => {
            let (ol, v, pl) = ginkgo::build_ginkgo_venation(&ginkgo::FanBlade::ginkgo(), 4, 1.5, 0.13, 7);
            finish("leaf_ginkgo", vec![ol], v, pl);
        }
        Some("mimosa") => {
            let leaf = compound::bipinnately_compound(seed, 5, 9);
            finish("leaf_mimosa", leaf.laminae, leaf.veins, leaf.petiole_len);
        }
        Some("birch") => {
            let blade = Blade::ovate().with_margin(Margin::doubly_serrate());
            generate(seed, &blade, SecondaryArch::Craspedodromous, true, "leaf_birch");
        }
        Some("lilac") => {
            let blade = Blade::shape(10.0, 3.4, 1.0, 2.0).with_cordate(0.16);
            generate(seed, &blade, SecondaryArch::Brochidodromous, true, "leaf_lilac");
        }
        Some("holly") => {
            let blade = Blade::shape(11.0, 3.0, 1.5, 1.8).with_margin(Margin::spinose());
            generate(seed, &blade, SecondaryArch::Craspedodromous, true, "leaf_holly");
        }
        Some("elm") => {
            let blade = Blade::ovate().with_margin(Margin::serrate()).with_asymmetry(0.45);
            generate(seed, &blade, SecondaryArch::Craspedodromous, true, "leaf_elm");
        }
        Some("sagittate") => {
            let blade = Blade::shape(12.0, 2.4, 0.95, 2.6).with_cordate(0.2).with_base_dir(1.0);
            generate(seed, &blade, SecondaryArch::Brochidodromous, true, "leaf_sagittate");
        }
        Some("hastate") => {
            let blade = Blade::shape(11.0, 2.6, 1.0, 2.4).with_cordate(0.16).with_base_dir(0.0);
            generate(seed, &blade, SecondaryArch::Brochidodromous, true, "leaf_hastate");
        }
        Some("tuliptree") => {
            let blade = Blade::shape(9.0, 3.4, 1.9, 1.5).with_apex_notch(0.22);
            generate(seed, &blade, SecondaryArch::Brochidodromous, true, "leaf_tuliptree");
        }
        Some("lotus") => {
            let (ol, v, pl) = peltate::assemble_peltate(&peltate::PeltateBlade::lotus(), 12, seed, 1.0, 360);
            finish("leaf_lotus", vec![ol], v, pl);
        }
        Some("nasturtium") => {
            let (ol, v, pl) = peltate::assemble_peltate(&peltate::PeltateBlade::nasturtium(), 9, seed, 1.0, 360);
            finish("leaf_nasturtium", vec![ol], v, pl);
        }
        Some("daisy") => render_scene("flower_daisy", &flower::build_flower(&FlowerParams::daisy())),
        Some("buttercup") => render_scene("flower_buttercup", &flower::build_flower(&FlowerParams::buttercup())),
        Some("rose") => render_scene("flower_rose", &flower::build_flower(&FlowerParams::rose())),
        Some("arches") => {
            generate(seed, &Blade::ovate(), SecondaryArch::Craspedodromous, false, "leaf_cras");
            generate(seed, &Blade::ovate(), SecondaryArch::Brochidodromous, false, "leaf_broch");
            generate(seed, &Blade::ovate(), SecondaryArch::Eucamptodromous, false, "leaf_eucamp");
        }
        Some("bench") => {
            use std::time::Instant;
            let opts = RenderOpts::default();

            // kind-0 leaf: time each phase separately.
            let blade = Blade::ovate().with_margin(Margin::serrate());
            let mp = MajorParams { arch: SecondaryArch::Craspedodromous, ..MajorParams::default() };
            let t = Instant::now();
            let mut v = major::build_major(&blade, &mp);
            let t_major = t.elapsed();
            let mut rng = Rng::new(42);
            let src = blade.sample_sources(5000, &mut rng);
            let t = Instant::now();
            venation::grow_minor(&mut v, src, &MinorParams::default());
            let t_minor = t.elapsed();
            let t = Instant::now();
            venation::anastomose(&mut v, &AnastomoseParams::default());
            let t_ana = t.elapsed();
            let n_edges = v.edges.len();
            let t = Instant::now();
            let svg = svg::render(&svg::Scene::leaf(vec![blade.outline(900)], v, 1.4), &opts);
            let t_svg = t.elapsed();
            println!(
                "ovate ({} edges): major {:?}  grow_minor {:?}  anastomose {:?}  svg {:?} ({} KB)",
                n_edges, t_major, t_minor, t_ana, t_svg, svg.len() / 1024
            );

            // ash (compound) assembly vs svg.
            let t = Instant::now();
            let leaf = compound::pinnately_compound(42, 5, 1.0);
            let t_ash = t.elapsed();
            let n_edges2 = leaf.veins.edges.len();
            let t = Instant::now();
            let svg = svg::render(&svg::Scene::leaf(leaf.laminae, leaf.veins, leaf.petiole_len), &opts);
            let t_svg2 = t.elapsed();
            println!(
                "ash   ({} edges): assemble {:?}  svg {:?} ({} KB)",
                n_edges2, t_ash, t_svg2, svg.len() / 1024
            );
        }
        other => {
            let seed: u64 = other.and_then(|s| s.parse().ok()).unwrap_or(0xC0FFEE);
            let arch = match args.next().as_deref() {
                Some("cras") => SecondaryArch::Craspedodromous,
                Some("eucamp") => SecondaryArch::Eucamptodromous,
                _ => SecondaryArch::Brochidodromous,
            };
            generate(seed, &Blade::ovate().with_margin(Margin::serrate()), arch, true, "leaf");
        }
    }
}
