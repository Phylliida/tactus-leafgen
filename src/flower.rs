//! Flowers: a radial whorl of petals (each petal is a little [`Blade`]) around a
//! colored center, reusing the leaf machinery. Color comes from the
//! [`Scene`]/[`Lamina`] model; the radial layout mirrors palmate/peltate.

use crate::blade::Blade;
use crate::svg::{Lamina, Rgb, Scene};
use crate::vec2::Vec2;
use crate::venation::VeinGraph;
use crate::Scalar;
use std::f64::consts::PI;

/// Map a petal-local point (midrib on +y, base at origin) to world space:
/// local +y → outward direction `d`, base → `base`.
fn place(base: Vec2, d: Vec2, p: Vec2) -> Vec2 {
    Vec2::new(base.x + p.x * d.y + p.y * d.x, base.y - p.x * d.x + p.y * d.y)
}

pub struct FlowerParams {
    pub n_petals: usize,
    /// Petal shape (a small blade).
    pub petal: Blade,
    pub petal_fill: Rgb,
    pub petal_stroke: Rgb,
    pub center_radius: Scalar,
    pub center_fill: Rgb,
    pub center_stroke: Rgb,
    pub stem_len: Scalar,
}

impl FlowerParams {
    /// Many narrow white ray petals + a yellow disk.
    pub fn daisy() -> Self {
        FlowerParams {
            n_petals: 16,
            petal: Blade::shape(5.0, 0.55, 1.7, 1.5),
            petal_fill: [253, 253, 250],
            petal_stroke: [176, 178, 168],
            center_radius: 1.7,
            center_fill: [255, 198, 40],
            center_stroke: [198, 148, 22],
            stem_len: 6.0,
        }
    }
    /// Five broad glossy-yellow petals.
    pub fn buttercup() -> Self {
        FlowerParams {
            n_petals: 5,
            petal: Blade::shape(3.4, 1.7, 2.0, 1.3),
            petal_fill: [255, 221, 50],
            petal_stroke: [220, 172, 20],
            center_radius: 1.0,
            center_fill: [150, 165, 55],
            center_stroke: [110, 125, 35],
            stem_len: 6.0,
        }
    }
    /// Five broad, rounded pink petals.
    pub fn rose() -> Self {
        FlowerParams {
            n_petals: 5,
            petal: Blade::shape(3.0, 2.0, 1.7, 1.3),
            petal_fill: [240, 150, 185],
            petal_stroke: [205, 110, 150],
            center_radius: 0.9,
            center_fill: [240, 200, 80],
            center_stroke: [200, 160, 40],
            stem_len: 6.0,
        }
    }
}

pub fn build_flower(p: &FlowerParams) -> Scene {
    let mut laminae: Vec<Lamina> = Vec::new();
    let mut veins = VeinGraph::new();
    let n = p.n_petals.max(3);
    let base_r = p.center_radius * 0.85; // petals tuck slightly under the center

    for i in 0..n {
        let th = PI / 2.0 + 2.0 * PI * i as Scalar / n as Scalar;
        let d = Vec2::new(th.cos(), th.sin());
        let base = d.scale(base_r);
        let ol: Vec<Vec2> = p.petal.outline(100).iter().map(|q| place(base, d, *q)).collect();
        laminae.push(Lamina::new(ol, p.petal_fill, p.petal_stroke));

        // petal midrib vein (stop short of the tip so it stays inside, even
        // for notched petals)
        let mut prev = veins.add_node(base, 1);
        let steps = 8;
        let vein_len = p.petal.length * 0.82;
        for k in 1..=steps {
            let f = k as Scalar / steps as Scalar;
            prev = veins.add_child(prev, base.add(d.scale(vein_len * f)), 2);
        }
    }

    // central disk drawn last (covers petal bases)
    let disk: Vec<Vec2> = (0..48)
        .map(|k| {
            let a = 2.0 * PI * k as Scalar / 48.0;
            Vec2::new(p.center_radius * a.cos(), p.center_radius * a.sin())
        })
        .collect();
    laminae.push(Lamina::new(disk, p.center_fill, p.center_stroke));

    Scene {
        laminae,
        veins,
        vein_base: p.petal_stroke,
        petiole_len: p.stem_len,
        petiole_color: [70, 110, 35],
    }
}
