//! Peltate leaves (lotus, nasturtium): the petiole attaches to the *centre* of
//! a roughly circular blade, and veins radiate outward in all directions from
//! that point. Reticulate minor venation fills between the radiating primaries.

use crate::rng::Rng;
use crate::vec2::Vec2;
use crate::venation::{self, AnastomoseParams, MinorParams, VeinGraph};
use crate::Scalar;

use std::f64::consts::PI;

pub struct PeltateBlade {
    pub radius: Scalar,
    /// Number of marginal lobes/scallops (0 = smooth circle).
    pub lobes: usize,
    /// Scallop amplitude as a fraction of radius.
    pub lobe_amp: Scalar,
}

impl PeltateBlade {
    pub fn lotus() -> Self {
        PeltateBlade { radius: 7.0, lobes: 0, lobe_amp: 0.0 }
    }
    pub fn nasturtium() -> Self {
        PeltateBlade { radius: 6.5, lobes: 9, lobe_amp: 0.06 }
    }

    /// Margin radius at angle `theta` (from +x).
    pub fn radius_at(&self, theta: Scalar) -> Scalar {
        if self.lobes == 0 {
            self.radius
        } else {
            self.radius * (1.0 + self.lobe_amp * (self.lobes as Scalar * theta).cos())
        }
    }

    pub fn contains(&self, p: Vec2) -> bool {
        let r = (p.x * p.x + p.y * p.y).sqrt();
        r <= self.radius_at(p.y.atan2(p.x))
    }

    pub fn outline(&self, samples: usize) -> Vec<Vec2> {
        (0..samples)
            .map(|i| {
                let th = 2.0 * PI * i as Scalar / samples as Scalar;
                let r = self.radius_at(th);
                Vec2::new(r * th.cos(), r * th.sin())
            })
            .collect()
    }

    pub fn sample_sources(&self, n: usize, rng: &mut Rng) -> Vec<Vec2> {
        let mut pts = Vec::with_capacity(n);
        let mut tries = 0usize;
        let max_tries = n.saturating_mul(200).max(10_000);
        while pts.len() < n && tries < max_tries {
            let p = Vec2::new(rng.range(-self.radius, self.radius), rng.range(-self.radius, self.radius));
            if self.contains(p) {
                pts.push(p);
            }
            tries += 1;
        }
        pts
    }
}

/// Radiating primary veins from the centre to the margin.
pub fn build_peltate_major(blade: &PeltateBlade, n_veins: usize) -> VeinGraph {
    let mut g = VeinGraph::new();
    let center = g.add_node(Vec2::new(0.0, 0.0), 0);
    let n = n_veins.max(4);
    let steps = 18;
    for i in 0..n {
        let th = 2.0 * PI * i as Scalar / n as Scalar;
        let dir = Vec2::new(th.cos(), th.sin());
        let rmax = blade.radius_at(th) * 0.97;
        let mut prev = center;
        for k in 1..=steps {
            let f = k as Scalar / steps as Scalar;
            let order = if k <= steps * 2 / 3 { 1 } else { 2 };
            prev = g.add_child(prev, dir.scale(rmax * f), order);
        }
    }
    g
}

/// Assemble a full peltate leaf: radiating primaries + reticulate minor venation.
/// Returns (outline, veins, petiole_len).
pub fn assemble_peltate(
    blade: &PeltateBlade,
    n_veins: usize,
    seed: u64,
    density: Scalar,
    outline_n: usize,
) -> (Vec<Vec2>, VeinGraph, Scalar) {
    let mut veins = build_peltate_major(blade, n_veins);
    let f = 2.0 * blade.radius / 10.0; // diameter-relative scale (defaults tuned for ~10)
    let minor = MinorParams {
        influence_radius: 0.6 * f,
        kill_radius: 0.10 * f,
        step: 0.09 * f,
        seed_clearance: 0.16 * f,
        max_iters: 400,
        max_order: 4,
    };
    let mut rng = Rng::new(seed);
    let n_sources = (100.0 * blade.radius * blade.radius * density).clamp(500.0, 6000.0) as usize;
    venation::grow_minor(&mut veins, blade.sample_sources(n_sources, &mut rng), &minor);
    venation::anastomose(&mut veins, &AnastomoseParams { radius: 0.17 * f, ancestor_depth: 4 });
    // petiole runs from the centre down past the lower margin
    (blade.outline(outline_n), veins, blade.radius * 1.4)
}
