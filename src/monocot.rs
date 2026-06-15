//! Monocot leaves: a linear/strap blade with **parallel (striate) venation** —
//! the whole region the midrib+reticulate model can't reach (grasses, lilies,
//! irises, bananas...).
//!
//! The blade is long and narrow with near-parallel sides (a plateau in the
//! half-width profile, unlike the single-hump dicot profile). Venation is a set
//! of longitudinal veins that all converge at the base and apex, optionally
//! tied together by sparse transverse cross-veins (the "ladder" of banana/Hosta).

use crate::vec2::Vec2;
use crate::venation::VeinGraph;
use crate::Scalar;

pub struct MonocotBlade {
    pub length: Scalar,
    pub half_width: Scalar,
    /// Fraction over which the width rises from the base.
    pub base_rise: Scalar,
    /// Fraction up to which the sides stay parallel (taper starts here).
    pub plateau_end: Scalar,
    /// Apex taper sharpness (>1 acuminate).
    pub apex_exp: Scalar,
}

impl MonocotBlade {
    /// Long, very narrow blade (grass).
    pub fn grass() -> Self {
        MonocotBlade { length: 20.0, half_width: 0.95, base_rise: 0.04, plateau_end: 0.5, apex_exp: 1.5 }
    }
    /// Broader strap (lily / Hosta).
    pub fn lily() -> Self {
        MonocotBlade { length: 15.0, half_width: 2.1, base_rise: 0.09, plateau_end: 0.4, apex_exp: 1.3 }
    }
    /// Stiff sword (iris), long parallel run.
    pub fn sword() -> Self {
        MonocotBlade { length: 19.0, half_width: 1.2, base_rise: 0.05, plateau_end: 0.72, apex_exp: 1.7 }
    }

    pub fn half_width_at(&self, t: Scalar) -> Scalar {
        if t <= 0.0 || t >= 1.0 {
            return 0.0;
        }
        if t <= self.base_rise {
            self.half_width * (t / self.base_rise)
        } else if t <= self.plateau_end {
            self.half_width
        } else {
            let u = (t - self.plateau_end) / (1.0 - self.plateau_end);
            self.half_width * (1.0 - u).powf(self.apex_exp)
        }
    }

    pub fn outline(&self, samples: usize) -> Vec<Vec2> {
        let mut pts = Vec::with_capacity(2 * samples + 2);
        for i in 0..=samples {
            let t = i as Scalar / samples as Scalar;
            pts.push(Vec2::new(self.half_width_at(t), t * self.length));
        }
        for i in (0..=samples).rev() {
            let t = i as Scalar / samples as Scalar;
            pts.push(Vec2::new(-self.half_width_at(t), t * self.length));
        }
        pts
    }
}

/// Build parallel venation: `n_veins` longitudinal veins (spread across the
/// width, all converging at base and apex), with a transverse cross-vein every
/// `cross_every` samples (0 = none). Returns (outline, veins, petiole_len).
pub fn build_monocot_venation(
    blade: &MonocotBlade,
    n_veins: usize,
    cross_every: usize,
) -> (Vec<Vec2>, VeinGraph, Scalar) {
    let mut g = VeinGraph::new();
    let l = blade.length;
    let n = n_veins.max(3);
    let samples = 64;
    let (t0, t1) = (0.025, 0.985);

    let base = g.add_node(Vec2::new(0.0, t0 * l), 0);
    let apex = g.add_node(Vec2::new(0.0, t1 * l), 0);

    let mut chains: Vec<Vec<usize>> = Vec::with_capacity(n);
    for vi in 0..n {
        // lateral position of this vein, in [-0.92, 0.92] of the local width
        let lat = if n == 1 { 0.0 } else { (-1.0 + 2.0 * vi as Scalar / (n as Scalar - 1.0)) * 0.92 };
        let order: u8 = if lat.abs() < 1e-3 { 1 } else { 2 }; // central vein a touch heavier

        let mut chain = vec![base];
        let mut prev = base;
        for k in 1..samples {
            let t = t0 + (t1 - t0) * k as Scalar / samples as Scalar;
            let w = blade.half_width_at(t);
            let idx = g.add_child(prev, Vec2::new(lat * w, t * l), order);
            chain.push(idx);
            prev = idx;
        }
        g.add_edge(prev, apex, order);
        chain.push(apex);
        chains.push(chain);
    }

    // Transverse commissural veins tying adjacent longitudinal veins together.
    if cross_every > 0 {
        for vi in 0..n - 1 {
            let (a, b) = (&chains[vi], &chains[vi + 1]);
            let m = a.len().min(b.len());
            let mut k = cross_every;
            while k < m - 1 {
                g.add_edge(a[k], b[k], 3);
                k += cross_every;
            }
        }
    }

    (blade.outline(400), g, l * 0.05)
}
