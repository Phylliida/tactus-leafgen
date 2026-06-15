//! Blade (leaf lamina) geometry: parametric outline + margin.
//!
//! The midrib lies on the +y axis from base (0,0) to apex (0, length). The
//! smooth half-width profile (perpendicular to the midrib, `t = y/length`) is
//! the beta-shaped bump
//!
//!     w(t) = half_width · tᵃ (1−t)ᵇ / peak
//!
//! which has a *rounded* maximum (no corner) at `t* = a/(a+b)`:
//!
//!   * widest position `t*` : `a < b` ovate, `a = b` elliptic, `a > b` obovate
//!   * `a` shapes the base, `b` the apex:
//!       <1 → rounded/obtuse end, =1 → straight (cuneate/acute),
//!       >1 → attenuate/acuminate (drawn-out)
//!   * aspect (length : half_width) gives lanceolate ↔ orbicular
//!
//! The margin decorates the smooth outline with teeth: periodic displacements
//! along the outward normal (serrate / dentate / crenate).

use crate::rng::Rng;
use crate::vec2::Vec2;
use crate::Scalar;

use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarginType {
    Entire,
    Serrate, // forward-pointing sawtooth teeth
    Dentate, // symmetric outward-pointing teeth
    Crenate, // rounded scallops
}

#[derive(Clone, Copy, Debug)]
pub struct Margin {
    pub kind: MarginType,
    pub n_teeth: usize,
    /// Tooth height in world units (along the outward normal).
    pub amp: Scalar,
}

impl Margin {
    pub fn entire() -> Self {
        Margin { kind: MarginType::Entire, n_teeth: 0, amp: 0.0 }
    }
    pub fn serrate() -> Self {
        Margin { kind: MarginType::Serrate, n_teeth: 24, amp: 0.24 }
    }
    pub fn dentate() -> Self {
        Margin { kind: MarginType::Dentate, n_teeth: 16, amp: 0.30 }
    }
    pub fn crenate() -> Self {
        Margin { kind: MarginType::Crenate, n_teeth: 13, amp: 0.32 }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Blade {
    pub length: Scalar,
    pub half_width: Scalar,
    /// Base-shape exponent (small a → widest point low → ovate).
    pub a: Scalar,
    /// Apex-shape exponent.
    pub b: Scalar,
    /// Peak of tᵃ(1−t)ᵇ, so the profile peaks at exactly `half_width`.
    peak: Scalar,
    pub margin: Margin,
}

impl Blade {
    pub fn shape(length: Scalar, half_width: Scalar, a: Scalar, b: Scalar) -> Self {
        let tstar = a / (a + b);
        let peak = tstar.powf(a) * (1.0 - tstar).powf(b);
        Blade { length, half_width, a, b, peak, margin: Margin::entire() }
    }

    pub fn with_margin(mut self, margin: Margin) -> Self {
        self.margin = margin;
        self
    }

    // ---- named shape presets (entire margin; add one with `.with_margin`) ----

    pub fn ovate() -> Self {
        Blade::shape(10.0, 3.2, 1.3, 2.2)
    }
    pub fn obovate() -> Self {
        Blade::shape(10.0, 3.2, 2.2, 1.3)
    }
    pub fn elliptic() -> Self {
        Blade::shape(10.0, 3.0, 1.7, 1.7)
    }
    pub fn lanceolate() -> Self {
        Blade::shape(12.0, 1.9, 1.5, 2.7)
    }

    /// Smooth half-width at normalized midrib position `t ∈ [0, 1]`.
    pub fn half_width_at(&self, t: Scalar) -> Scalar {
        if t <= 0.0 || t >= 1.0 {
            return 0.0;
        }
        self.half_width * t.powf(self.a) * (1.0 - t).powf(self.b) / self.peak
    }

    /// Is point `p` inside the smooth blade? (Teeth project outward from the
    /// smooth curve, so the smooth region is contained in the real leaf —
    /// sources sampled here are always inside.)
    pub fn contains(&self, p: Vec2) -> bool {
        if p.y < 0.0 || p.y > self.length {
            return false;
        }
        p.x.abs() <= self.half_width_at(p.y / self.length)
    }

    fn smooth_pt(&self, t: Scalar, side: Scalar) -> Vec2 {
        Vec2::new(side * self.half_width_at(t), t * self.length)
    }

    /// Outward unit normal to the smooth margin at `t` on the given side.
    fn normal(&self, t: Scalar, side: Scalar) -> Vec2 {
        let dt = 1e-3;
        let a = self.smooth_pt((t - dt).max(0.0), side);
        let b = self.smooth_pt((t + dt).min(1.0), side);
        let tan = b.sub(a);
        let mut n = Vec2::new(tan.y, -tan.x).normalized();
        if n.x * side < 0.0 {
            n = n.scale(-1.0); // ensure it points away from the midrib
        }
        n
    }

    /// Tooth displacement (≥ 0) at `t`, along the outward normal.
    fn tooth_disp(&self, t: Scalar) -> Scalar {
        if self.margin.kind == MarginType::Entire || self.margin.n_teeth == 0 {
            return 0.0;
        }
        let (lo, hi) = (0.06, 0.94); // keep teeth off the base/apex tips
        if t < lo || t > hi {
            return 0.0;
        }
        let u = (t - lo) / (hi - lo);
        let ph = (u * self.margin.n_teeth as Scalar).fract();
        let prof = match self.margin.kind {
            MarginType::Entire => 0.0,
            MarginType::Crenate => 0.5 * (1.0 - (2.0 * PI * ph).cos()),
            MarginType::Dentate => 1.0 - (2.0 * ph - 1.0).abs(),
            MarginType::Serrate => ph,
        };
        self.margin.amp * prof
    }

    fn margin_pt(&self, t: Scalar, side: Scalar) -> Vec2 {
        let base = self.smooth_pt(t, side);
        let d = self.tooth_disp(t);
        if d == 0.0 {
            base
        } else {
            base.add(self.normal(t, side).scale(d))
        }
    }

    /// Outline polygon (with teeth): right chain base→apex, then left apex→base.
    pub fn outline(&self, samples: usize) -> Vec<Vec2> {
        let mut pts = Vec::with_capacity(2 * samples + 2);
        for i in 0..=samples {
            pts.push(self.margin_pt(i as Scalar / samples as Scalar, 1.0));
        }
        for i in (0..=samples).rev() {
            pts.push(self.margin_pt(i as Scalar / samples as Scalar, -1.0));
        }
        pts
    }

    /// Max horizontal half-extent including teeth (for sizing renders).
    pub fn half_extent(&self) -> Scalar {
        self.half_width + if self.margin.kind == MarginType::Entire { 0.0 } else { self.margin.amp }
    }

    pub fn sample_sources(&self, n: usize, rng: &mut Rng) -> Vec<Vec2> {
        let mut pts = Vec::with_capacity(n);
        let mut tries = 0usize;
        let max_tries = n.saturating_mul(200).max(10_000);
        while pts.len() < n && tries < max_tries {
            let p = Vec2::new(
                rng.range(-self.half_width, self.half_width),
                rng.range(0.0, self.length),
            );
            if self.contains(p) {
                pts.push(p);
            }
            tries += 1;
        }
        pts
    }
}
