//! Ginkgo: a fan-shaped (flabellate) blade with **open dichotomous venation** —
//! veins enter from the petiole and fork repeatedly (Y-splits), with no midrib,
//! no reticulation, and no cross-veins. The third distinct venation engine
//! (alongside reticulate dicots and parallel monocots).

use crate::vec2::Vec2;
use crate::venation::VeinGraph;
use crate::Scalar;

pub struct FanBlade {
    /// Distance from petiole to the distal margin.
    pub radius: Scalar,
    /// Half-angle of the fan (radians).
    pub spread: Scalar,
    /// Central cleft depth as a fraction of radius (0 = entire, >0 = bilobed).
    pub notch: Scalar,
    /// Angular width of the cleft (radians).
    pub notch_width: Scalar,
}

impl FanBlade {
    pub fn ginkgo() -> Self {
        FanBlade { radius: 9.0, spread: 1.05, notch: 0.22, notch_width: 0.16 }
    }

    /// Distal-margin radius at fan angle `alpha` (measured from +y).
    pub fn radius_at(&self, alpha: Scalar) -> Scalar {
        let d = alpha / self.notch_width;
        self.radius * (1.0 - self.notch * (-d * d).exp())
    }

    pub fn contains(&self, p: Vec2) -> bool {
        let r = (p.x * p.x + p.y * p.y).sqrt();
        let alpha = p.x.atan2(p.y);
        alpha.abs() <= self.spread && r <= self.radius_at(alpha)
    }

    pub fn outline(&self, samples: usize) -> Vec<Vec2> {
        let mut pts = Vec::with_capacity(samples + 2);
        pts.push(Vec2::new(0.0, 0.0)); // petiole point
        for i in 0..=samples {
            let alpha = -self.spread + 2.0 * self.spread * (i as Scalar / samples as Scalar);
            let r = self.radius_at(alpha);
            pts.push(Vec2::new(alpha.sin() * r, alpha.cos() * r));
        }
        pts
    }
}

/// Grow dichotomous venation: `n_base` veins leave the petiole and each forks
/// every ~`fork_spacing` of length into two veins diverging by `divergence`
/// (radians), up to `max_gen` generations, stopping at the margin.
pub fn build_ginkgo_venation(
    blade: &FanBlade,
    n_base: usize,
    fork_spacing: Scalar,
    divergence: Scalar,
    max_gen: u8,
) -> (Vec<Vec2>, VeinGraph, Scalar) {
    let mut g = VeinGraph::new();
    let base = g.add_node(Vec2::new(0.0, 0.0), 0);
    let step = blade.radius * 0.03;
    let n = n_base.max(2);

    // (node, heading angle, length since last fork, generation)
    let mut stack: Vec<(usize, Scalar, Scalar, u8)> = Vec::new();
    for i in 0..n {
        let a = (-1.0 + 2.0 * i as Scalar / (n as Scalar - 1.0)) * blade.spread * 0.5;
        stack.push((base, a, 0.0, 1));
    }

    while let Some((mut node, angle, mut len, gen)) = stack.pop() {
        loop {
            let p = g.nodes[node];
            let np = Vec2::new(p.x + step * angle.sin(), p.y + step * angle.cos());
            let r = (np.x * np.x + np.y * np.y).sqrt();
            let alpha = np.x.atan2(np.y);
            if alpha.abs() >= blade.spread * 0.99 || r >= blade.radius_at(alpha) {
                break; // reached a margin
            }
            let order = gen.min(4);
            node = g.add_child(node, np, order);
            len += step;
            if len >= fork_spacing && gen < max_gen {
                stack.push((node, angle + divergence, 0.0, gen + 1));
                stack.push((node, angle - divergence, 0.0, gen + 1));
                break; // this tip ends; its two children carry on
            }
        }
    }

    (blade.outline(300), g, blade.radius * 0.32)
}
