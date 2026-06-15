//! Major venation: midrib + secondaries, built parametrically so that
//! secondary architecture is an explicit choice rather than an emergent
//! accident. Covers the named pinnate types:
//!
//!   * Craspedodromous — secondaries run straight to the margin (→ teeth).
//!   * Brochidodromous — secondaries arch and join the next one up, forming a
//!     series of marginal loops (the start of areole structure).
//!   * Eucamptodromous — secondaries curve apically and fade without joining.
//!
//! Each secondary is traced as a quadratic Bézier from its midrib origin to an
//! anchor near/at the margin, leaving the midrib at a set divergence angle.

use crate::blade::Blade;
use crate::vec2::Vec2;
use crate::venation::VeinGraph;
use crate::Scalar;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecondaryArch {
    Craspedodromous,
    Brochidodromous,
    Eucamptodromous,
}

#[derive(Clone, Copy, Debug)]
pub struct MajorParams {
    pub n_secondaries: usize,
    /// Midrib fraction of the lowest / highest secondary origin.
    pub first_t: Scalar,
    pub last_t: Scalar,
    /// Angle (degrees) each secondary leaves the midrib, measured from the
    /// midrib axis toward the apex.
    pub divergence_deg: Scalar,
    /// How far up the remaining midrib a secondary reaches before its anchor
    /// (fraction of `1 - t_i`). Larger → longer, shallower secondaries.
    pub reach_frac: Scalar,
    pub arch: SecondaryArch,
    /// For brochido/eucampto: how far inside the margin the anchor sits, as a
    /// fraction of the local half-width.
    pub margin_gap: Scalar,
    pub midrib_samples: usize,
    pub sec_samples: usize,
    pub loop_samples: usize,
}

impl Default for MajorParams {
    fn default() -> Self {
        MajorParams {
            n_secondaries: 7,
            first_t: 0.12,
            last_t: 0.70,
            divergence_deg: 50.0,
            reach_frac: 0.34,
            arch: SecondaryArch::Brochidodromous,
            margin_gap: 0.12,
            midrib_samples: 44,
            sec_samples: 16,
            loop_samples: 12,
        }
    }
}

fn bezier(o: Vec2, c: Vec2, a: Vec2, n: usize) -> Vec<Vec2> {
    (0..=n)
        .map(|i| {
            let t = i as Scalar / n as Scalar;
            let u = 1.0 - t;
            o.scale(u * u).add(c.scale(2.0 * u * t)).add(a.scale(t * t))
        })
        .collect()
}

/// Add a polyline of `pts` (whose first element is the existing node `start`)
/// to the graph as a chain of `order` edges; return the last node's index.
fn add_chain(g: &mut VeinGraph, start: usize, pts: &[Vec2], order: u8) -> usize {
    let mut prev = start;
    for p in pts.iter().skip(1) {
        prev = g.add_child(prev, *p, order);
    }
    prev
}

pub fn build_major(blade: &Blade, p: &MajorParams) -> VeinGraph {
    let mut g = VeinGraph::new();
    let l = blade.length;

    // Secondary origins: one per lobe tip if lobed, else evenly spaced.
    let lobe_ts = blade.lobe_centers();
    let lobed = !lobe_ts.is_empty();
    let sec_ts: Vec<Scalar> = if lobed {
        lobe_ts
    } else {
        (0..p.n_secondaries)
            .map(|i| {
                let f = if p.n_secondaries == 1 {
                    0.5
                } else {
                    i as Scalar / (p.n_secondaries as Scalar - 1.0)
                };
                p.first_t + (p.last_t - p.first_t) * f
            })
            .collect()
    };
    // Lobed leaves run a vein straight to each lobe tip (craspedodromous).
    let arch = if lobed { SecondaryArch::Craspedodromous } else { p.arch };

    // Midrib node parameters: uniform samples ∪ secondary origins, sorted.
    let mut ts: Vec<Scalar> = (0..=p.midrib_samples)
        .map(|i| i as Scalar / p.midrib_samples as Scalar)
        .collect();
    ts.extend(sec_ts.iter().copied());
    ts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ts.dedup_by(|a, b| (*a - *b).abs() < 1e-9);

    let mid_idx: Vec<usize> = ts.iter().map(|t| g.add_node(Vec2::new(0.0, t * l), 0)).collect();
    for w in mid_idx.windows(2) {
        g.add_edge(w[0], w[1], 0);
    }
    let find_mid = |t: Scalar| -> usize {
        let mut best = 0;
        let mut bd = Scalar::INFINITY;
        for (k, tt) in ts.iter().enumerate() {
            let d = (tt - t).abs();
            if d < bd {
                bd = d;
                best = k;
            }
        }
        mid_idx[best]
    };

    let div = p.divergence_deg.to_radians();
    // anchors[side] in order of ascending t, for brochido loop connectors.
    let mut anchors: [Vec<usize>; 2] = [Vec::new(), Vec::new()];

    for &t_i in &sec_ts {
        let origin = find_mid(t_i);
        let o = g.nodes[origin];
        for (si, &s) in [1.0 as Scalar, -1.0].iter().enumerate() {
            // Lobed: anchor at the lobe tip (same height). Else cap below the
            // thin apex so secondaries don't curl back into a teardrop.
            let t_a = if lobed {
                t_i
            } else {
                (t_i + (1.0 - t_i) * p.reach_frac).min(0.84)
            };
            let margin_w = blade.half_width_at(t_a);
            let inset = match arch {
                SecondaryArch::Craspedodromous => 0.0,
                SecondaryArch::Brochidodromous => p.margin_gap,
                SecondaryArch::Eucamptodromous => p.margin_gap * 2.2,
            };
            let a = Vec2::new(s * margin_w * (1.0 - inset), t_a * l);
            let dist = o.dist_sq(a).sqrt();
            let d0 = Vec2::new(s * div.sin(), div.cos());
            let c = o.add(d0.scale(dist * 0.55));
            let pts = bezier(o, c, a, p.sec_samples);
            let last = add_chain(&mut g, origin, &pts, 1);
            anchors[si].push(last);
        }
    }

    // Brochidodromous: join adjacent anchors with outward-bowed arches.
    if arch == SecondaryArch::Brochidodromous {
        for side in &anchors {
            for w in side.windows(2) {
                let a0 = g.nodes[w[0]];
                let a1 = g.nodes[w[1]];
                let mid = a0.add(a1).scale(0.5);
                let sgn = if mid.x >= 0.0 { 1.0 } else { -1.0 };
                let bow = a0.dist_sq(a1).sqrt() * 0.22;
                let c = Vec2::new(mid.x + sgn * bow, mid.y);
                let pts = bezier(a0, c, a1, p.loop_samples);
                // chain interior points, then close onto the existing anchor w[1]
                let mut prev = w[0];
                for q in pts.iter().take(pts.len() - 1).skip(1) {
                    let idx = g.add_node(*q, 1);
                    g.add_edge(prev, idx, 1);
                    prev = idx;
                }
                g.add_edge(prev, w[1], 1);
            }
        }
    }

    g
}
