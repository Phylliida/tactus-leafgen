//! Palmately-lobed blade (maple / sycamore / grape).
//!
//! Unlike the midrib model, a palmate leaf is built *radially* around the
//! petiole point at the origin. Each lobe is a bump in the polar radius
//! function `R(α)` (α measured from the +y axis), peaking at the lobe's axis
//! angle and dipping to `base_radius` between lobes (the sinuses). Venation is
//! **actinodromous**: one primary vein runs from the origin out to each lobe
//! tip, with reticulate minor venation filling the lamina.

use crate::rng::Rng;
use crate::vec2::Vec2;
use crate::venation::VeinGraph;
use crate::Scalar;

#[derive(Clone, Debug)]
pub struct PalmateBlade {
    /// Lobe axis angles (radians, signed, from +y toward +x).
    pub angles: Vec<Scalar>,
    /// Lobe lengths (origin → tip).
    pub lengths: Vec<Scalar>,
    /// Angular half-width of each lobe bump (radians).
    pub widths: Vec<Scalar>,
    /// Radius floor between lobes (sinus depth: smaller = deeper sinuses).
    pub base_radius: Scalar,
    /// Outline sweep extent: |α| ≤ span (radians).
    pub span: Scalar,
}

impl PalmateBlade {
    /// Five-lobed maple.
    pub fn maple() -> Self {
        let l = 7.0;
        PalmateBlade {
            angles: vec![0.0, 0.84, -0.84, 1.62, -1.62], // 0, ±48°, ±93°
            lengths: vec![l, 0.92 * l, 0.92 * l, 0.74 * l, 0.74 * l],
            widths: vec![0.40, 0.38, 0.38, 0.36, 0.36],
            base_radius: 0.11 * l,
            span: 2.05, // ~117°
        }
    }

    /// Polar radius of the lamina margin at angle `alpha`. A cusped (|d|^1.5)
    /// falloff gives pointed lobe tips rather than rounded blobs.
    pub fn radius_at(&self, alpha: Scalar) -> Scalar {
        let mut r = self.base_radius;
        for i in 0..self.angles.len() {
            let d = ((alpha - self.angles[i]) / self.widths[i]).abs();
            r += (self.lengths[i] - self.base_radius).max(0.0) * (-d.powf(1.5)).exp();
        }
        r
    }

    pub fn lobe_tip(&self, i: usize) -> Vec2 {
        let a = self.angles[i];
        Vec2::new(a.sin() * self.lengths[i], a.cos() * self.lengths[i])
    }

    pub fn contains(&self, p: Vec2) -> bool {
        let r = (p.x * p.x + p.y * p.y).sqrt();
        if r < 1e-9 {
            return true;
        }
        let alpha = p.x.atan2(p.y); // angle from +y axis
        if alpha.abs() > self.span {
            return false;
        }
        r <= self.radius_at(alpha)
    }

    pub fn outline(&self, samples: usize) -> Vec<Vec2> {
        let mut pts = Vec::with_capacity(samples + 2);
        for i in 0..=samples {
            let alpha = -self.span + 2.0 * self.span * (i as Scalar / samples as Scalar);
            let r = self.radius_at(alpha);
            pts.push(Vec2::new(alpha.sin() * r, alpha.cos() * r));
        }
        // close through the petiole origin (basal sinus)
        pts.push(Vec2::new(0.0, 0.0));
        pts
    }

    pub fn sample_sources(&self, n: usize, rng: &mut Rng) -> Vec<Vec2> {
        let max_r = self.lengths.iter().cloned().fold(0.0_f64, Scalar::max);
        let mut pts = Vec::with_capacity(n);
        let mut tries = 0usize;
        let max_tries = n.saturating_mul(200).max(10_000);
        while pts.len() < n && tries < max_tries {
            let p = Vec2::new(rng.range(-max_r, max_r), rng.range(-0.4 * max_r, max_r));
            if self.contains(p) {
                pts.push(p);
            }
            tries += 1;
        }
        pts
    }
}

/// Build the major (primary) venation: a vein from the origin to each lobe tip,
/// plus a few secondaries angling off each primary toward the lobe margins.
pub fn build_palmate_major(blade: &PalmateBlade) -> VeinGraph {
    let mut g = VeinGraph::new();
    let origin = g.add_node(Vec2::new(0.0, 0.0), 0);

    for i in 0..blade.angles.len() {
        let tip = blade.lobe_tip(i);
        // Primary: straight chain origin → tip (order 0, like a midrib).
        let steps = 18;
        let mut prev = origin;
        let mut prim_nodes = vec![origin];
        for k in 1..=steps {
            let t = k as Scalar / steps as Scalar;
            prev = g.add_child(prev, Vec2::new(tip.x * t, tip.y * t), 0);
            prim_nodes.push(prev);
        }
        // Secondaries off the primary, angling toward the lobe edges.
        let axis = blade.angles[i];
        for &frac in &[0.45, 0.65, 0.82] {
            let base_idx = prim_nodes[(frac * steps as Scalar) as usize];
            let base = g.nodes[base_idx];
            for &s in &[1.0 as Scalar, -1.0] {
                let off = axis + s * blade.widths[i] * 1.1;
                let reach = blade.lengths[i] * (1.0 - frac) * 0.7;
                let end = base.add(Vec2::new(off.sin() * reach, off.cos() * reach));
                let mid = base.add(end.sub(base).scale(0.5));
                // gentle two-segment secondary
                let m = g.add_child(base_idx, mid, 1);
                g.add_child(m, end, 1);
            }
        }
    }
    g
}
