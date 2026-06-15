//! Canonical registry of every named leaf template — one source of truth the
//! studio mirrors and the tests enumerate. Each entry is built with the same
//! constructors the app uses and returned in a uniform form.

use crate::blade::{Blade, Margin};
use crate::major::SecondaryArch::{self, Brochidodromous as Broch, Craspedodromous as Cras};
use crate::vec2::Vec2;
use crate::venation::VeinGraph;
use crate::Scalar;
use crate::{compound, ginkgo, monocot, palmate, peltate};

pub struct NamedLeaf {
    pub name: &'static str,
    pub laminae: Vec<Vec<Vec2>>,
    pub veins: VeinGraph,
    pub petiole_len: Scalar,
    /// True for a single contiguous lamina (vein containment is well-defined);
    /// false for compound leaves, whose rachis legitimately lies between leaflets.
    pub single_lamina: bool,
}

const SEED: u64 = 42;

fn pinnate(name: &'static str, blade: Blade, arch: SecondaryArch, n_sec: usize) -> NamedLeaf {
    let (ol, veins) = compound::assemble(&blade, arch, n_sec, SEED, 0.6, 600);
    NamedLeaf { name, laminae: vec![ol], veins, petiole_len: blade.length * 0.14, single_lamina: true }
}

fn single(name: &'static str, ol: Vec<Vec2>, veins: VeinGraph, petiole_len: Scalar) -> NamedLeaf {
    NamedLeaf { name, laminae: vec![ol], veins, petiole_len, single_lamina: true }
}

fn compound(name: &'static str, leaf: compound::Leaf) -> NamedLeaf {
    NamedLeaf {
        name,
        laminae: leaf.laminae,
        veins: leaf.veins,
        petiole_len: leaf.petiole_len,
        single_lamina: false,
    }
}

/// Every template in the studio gallery.
pub fn all() -> Vec<NamedLeaf> {
    let mut v = Vec::new();

    // --- pinnate-blade family (single lamina) ---
    v.push(pinnate("ovate", Blade::ovate(), Broch, 7));
    v.push(pinnate("obovate", Blade::obovate(), Broch, 7));
    v.push(pinnate("elliptic", Blade::elliptic(), Broch, 7));
    v.push(pinnate("lanceolate", Blade::lanceolate(), Broch, 8));
    v.push(pinnate("serrate", Blade::ovate().with_margin(Margin::serrate()), Cras, 7));
    v.push(pinnate("dentate", Blade::ovate().with_margin(Margin::dentate()), Cras, 7));
    v.push(pinnate("crenate", Blade::ovate().with_margin(Margin::crenate()), Broch, 7));
    v.push(pinnate("doubly_serrate", Blade::ovate().with_margin(Margin::doubly_serrate()), Cras, 7));
    v.push(pinnate("spinose", Blade::shape(11.0, 3.0, 1.5, 1.8).with_margin(Margin::spinose()), Cras, 7));
    v.push(pinnate("oak", Blade::oak(), Cras, 7));
    v.push(pinnate("cordate", Blade::shape(10.0, 3.4, 1.0, 2.0).with_cordate(0.16), Broch, 7));
    v.push(pinnate("sagittate", Blade::shape(12.0, 2.4, 0.95, 2.6).with_cordate(0.2).with_base_dir(1.0), Broch, 6));
    v.push(pinnate("hastate", Blade::shape(11.0, 2.6, 1.0, 2.4).with_cordate(0.16).with_base_dir(0.0), Broch, 6));
    v.push(pinnate("oblique", Blade::ovate().with_margin(Margin::serrate()).with_asymmetry(0.45), Cras, 7));
    v.push(pinnate("notched", Blade::shape(9.0, 3.4, 1.9, 1.5).with_apex_notch(0.22), Broch, 6));

    // --- other single-lamina venation engines ---
    let (ol, vg, pl) = palmate::assemble_palmate(&palmate::PalmateBlade::maple(), SEED, 0.6, 600);
    v.push(single("maple", ol, vg, pl));
    let (ol, vg, pl) = monocot::build_monocot_venation(&monocot::MonocotBlade::grass(), 11, 6);
    v.push(single("grass", ol, vg, pl));
    let (ol, vg, pl) = monocot::build_monocot_venation(&monocot::MonocotBlade::lily(), 15, 4);
    v.push(single("lily", ol, vg, pl));
    let (ol, vg, pl) = ginkgo::build_ginkgo_venation(&ginkgo::FanBlade::ginkgo(), 4, 1.5, 0.13, 7);
    v.push(single("ginkgo", ol, vg, pl));
    let (ol, vg, pl) = peltate::assemble_peltate(&peltate::PeltateBlade::lotus(), 12, SEED, 0.6, 360);
    v.push(single("lotus", ol, vg, pl));
    let (ol, vg, pl) = peltate::assemble_peltate(&peltate::PeltateBlade::nasturtium(), 9, SEED, 0.6, 360);
    v.push(single("nasturtium", ol, vg, pl));

    // --- compound (multi-lamina) ---
    v.push(compound("ash", compound::pinnately_compound(SEED, 5, 0.6)));
    v.push(compound("horsechestnut", compound::palmately_compound(SEED, 7, 115.0, 0.6)));
    v.push(compound("clover", compound::palmately_compound(SEED, 3, 38.0, 0.6)));
    v.push(compound("mimosa", compound::bipinnately_compound(SEED, 5, 9)));

    v
}
