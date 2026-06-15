//! Blade (leaf lamina) geometry.
//!
//! Phase-1 model: a midrib laid on the +y axis from the base (0,0) to the apex
//! (0, length), with a half-width profile `w(t)` perpendicular to it, where
//! `t = y / length ∈ [0, 1]`. The outline is the midrib offset by `±w(t)`.
//!
//! The half-width profile is the classic beta-shaped bump
//!     w(t) = half_width · tᵃ (1−t)ᵇ / peak
//! whose widest point sits at `t* = a / (a+b)`:
//!   * `a < b`  → widest below the middle  → ovate
//!   * `a > b`  → widest above the middle  → obovate
//!   * `a = b`  → symmetric                → elliptic
//! Margin teeth/serrations and lobes are perturbations/modulations of `w(t)`
//! to be layered on later — this gives the entire-margin family.

use crate::rng::Rng;
use crate::vec2::Vec2;
use crate::Scalar;

pub struct Blade {
    pub length: Scalar,
    /// Maximum half-width (at the widest point).
    pub half_width: Scalar,
    pub a: Scalar,
    pub b: Scalar,
    /// Peak value of tᵃ(1−t)ᵇ, precomputed so `half_width_at` peaks at exactly
    /// `half_width`.
    peak: Scalar,
}

impl Blade {
    pub fn new(length: Scalar, half_width: Scalar, a: Scalar, b: Scalar) -> Self {
        let tstar = a / (a + b);
        let peak = tstar.powf(a) * (1.0 - tstar).powf(b);
        Blade {
            length,
            half_width,
            a,
            b,
            peak,
        }
    }

    /// Half-width at normalized position `t ∈ [0, 1]` along the midrib.
    pub fn half_width_at(&self, t: Scalar) -> Scalar {
        if t <= 0.0 || t >= 1.0 {
            return 0.0;
        }
        self.half_width * t.powf(self.a) * (1.0 - t).powf(self.b) / self.peak
    }

    /// Is point `p` inside the blade?
    pub fn contains(&self, p: Vec2) -> bool {
        if p.y < 0.0 || p.y > self.length {
            return false;
        }
        let t = p.y / self.length;
        p.x.abs() <= self.half_width_at(t)
    }

    /// Outline polygon: right chain base→apex, then left chain apex→base.
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

    /// Axis-aligned bounding box of the lamina (min, max).
    pub fn bbox(&self) -> (Vec2, Vec2) {
        (
            Vec2::new(-self.half_width, 0.0),
            Vec2::new(self.half_width, self.length),
        )
    }

    /// Rejection-sample `n` auxin sources uniformly inside the blade.
    /// (Poisson-disk sampling would give nicer spacing; this is the simple v1.)
    pub fn sample_sources(&self, n: usize, rng: &mut Rng) -> Vec<Vec2> {
        let (lo, hi) = self.bbox();
        let mut pts = Vec::with_capacity(n);
        let mut tries = 0usize;
        let max_tries = n.saturating_mul(200).max(10_000);
        while pts.len() < n && tries < max_tries {
            let p = Vec2::new(rng.range(lo.x, hi.x), rng.range(lo.y, hi.y));
            if self.contains(p) {
                pts.push(p);
            }
            tries += 1;
        }
        pts
    }
}
