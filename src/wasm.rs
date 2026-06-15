//! Raw `wasm32-unknown-unknown` interface (no wasm-bindgen).
//!
//! `generate(...)` takes a flat list of `f64` parameters, builds the leaf, and
//! stashes the SVG bytes in a leaked buffer. JS reads them from linear memory
//! via `result_ptr()` / `result_len()` (valid until the next `generate` call).

use crate::blade::{Blade, Lobing, Margin, MarginType};
use crate::compound;
use crate::ginkgo;
use crate::major::SecondaryArch;
use crate::monocot;
use crate::palmate::{self, PalmateBlade};
use crate::svg::{self, RenderOpts};
use crate::vec2::Vec2;
use crate::venation::VeinGraph;

static mut PTR: usize = 0;
static mut LEN: usize = 0;

fn store(s: String) {
    // Reclaim the previous buffer.
    let (p, l) = unsafe { (PTR, LEN) };
    if p != 0 {
        drop(unsafe { Vec::from_raw_parts(p as *mut u8, l, l) });
    }
    let mut v = s.into_bytes();
    v.shrink_to_fit();
    let len = v.len();
    let ptr = v.as_mut_ptr() as usize;
    core::mem::forget(v);
    unsafe {
        PTR = ptr;
        LEN = len;
    }
}

#[no_mangle]
pub extern "C" fn result_ptr() -> usize {
    unsafe { PTR }
}

#[no_mangle]
pub extern "C" fn result_len() -> usize {
    unsafe { LEN }
}

fn web_opts() -> RenderOpts {
    RenderOpts {
        target_height_px: 860.0,
        pad_px: 24.0,
        petiole_frac: 0.14,
        min_vein_px: 0.5,
        max_vein_px: 5.0,
    }
}

fn margin_of(kind: i32, n_teeth: usize, amp: f64) -> Margin {
    let k = match kind {
        1 => MarginType::Serrate,
        2 => MarginType::Dentate,
        3 => MarginType::Crenate,
        4 => MarginType::DoublySerrate,
        _ => MarginType::Entire,
    };
    Margin { kind: k, n_teeth, amp }
}

fn arch_of(a: i32) -> SecondaryArch {
    match a {
        0 => SecondaryArch::Craspedodromous,
        2 => SecondaryArch::Eucamptodromous,
        _ => SecondaryArch::Brochidodromous,
    }
}

/// kind: 0 pinnate-blade (simple/shaped/toothed/pinnate-lobed) · 1 maple
/// (palmate-lobed) · 2 ash (pinnate compound) · 3 horse-chestnut (palmate
/// compound) · 4 clover (trifoliate). `lobe_n` doubles as lobe/leaflet count.
#[no_mangle]
pub extern "C" fn generate(
    kind: f64,
    length: f64,
    half_width: f64,
    a: f64,
    b: f64,
    margin: f64,
    n_teeth: f64,
    amp: f64,
    lobe_n: f64,
    lobe_depth: f64,
    arch: f64,
    n_sec: f64,
    seed: f64,
) {
    let seed = seed as u64;
    let lobe_n = lobe_n as usize;
    let opts = web_opts();

    let (laminae, veins, petiole_len): (Vec<Vec<Vec2>>, VeinGraph, f64) = match kind as i32 {
        0 => {
            let mut blade = Blade::shape(length.max(1.0), half_width.max(0.3), a.max(0.3), b.max(0.3))
                .with_margin(margin_of(margin as i32, n_teeth as usize, amp));
            if lobe_n > 0 {
                blade = blade.with_lobing(Lobing::pinnate(lobe_n, lobe_depth));
            }
            let arch = if lobe_n > 0 { SecondaryArch::Craspedodromous } else { arch_of(arch as i32) };
            let (ol, v) = compound::assemble(&blade, arch, n_sec as usize, seed, 0.55, 600);
            (vec![ol], v, length * 0.14)
        }
        1 => {
            let pb = PalmateBlade::palmate(lobe_n.max(3), length.max(4.0));
            let (ol, v, pl) = palmate::assemble_palmate(&pb, seed, 0.40, 700);
            (vec![ol], v, pl)
        }
        2 => {
            let leaf = compound::pinnately_compound(seed, lobe_n.max(2), 0.45);
            (leaf.laminae, leaf.veins, leaf.petiole_len)
        }
        3 => {
            let leaf = compound::palmately_compound(seed, lobe_n.max(3), 115.0, 0.45);
            (leaf.laminae, leaf.veins, leaf.petiole_len)
        }
        4 => {
            let leaf = compound::palmately_compound(seed, 3, 38.0, 0.45);
            (leaf.laminae, leaf.veins, leaf.petiole_len)
        }
        5 => {
            let mb = monocot::MonocotBlade {
                length: length.max(6.0),
                half_width: half_width.max(0.4),
                base_rise: 0.05,
                plateau_end: 0.5,
                apex_exp: b.clamp(0.6, 2.6),
            };
            let (ol, v, pl) = monocot::build_monocot_venation(&mb, lobe_n.max(3), (n_sec as usize).max(2));
            (vec![ol], v, pl)
        }
        6 => {
            let r = length.max(5.0);
            let fb = ginkgo::FanBlade {
                radius: r,
                spread: (0.4 + a * 0.3).clamp(0.5, 1.3),
                notch: (lobe_depth * 0.5).clamp(0.0, 0.5),
                notch_width: 0.16,
            };
            let fs = r / (n_sec.max(3.0));
            let (ol, v, pl) = ginkgo::build_ginkgo_venation(&fb, lobe_n.max(2), fs, 0.13, 7);
            (vec![ol], v, pl)
        }
        7 => {
            let leaf = compound::bipinnately_compound(seed, lobe_n.max(2), (n_sec as usize).max(3));
            (leaf.laminae, leaf.veins, leaf.petiole_len)
        }
        _ => {
            let (ol, v) = compound::assemble(&Blade::ovate(), SecondaryArch::Brochidodromous, 7, seed, 0.55, 600);
            (vec![ol], v, 1.4)
        }
    };

    store(svg::render(&laminae, &veins, petiole_len, &opts));
}
