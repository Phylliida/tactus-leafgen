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

/// Midrib span over which lobes are distributed (leaves the base/apex tips smooth).
const LOBE_LO: Scalar = 0.05;
const LOBE_HI: Scalar = 0.95;

/// Quadratic Bézier point.
fn bez(p0: Vec2, c: Vec2, p1: Vec2, s: Scalar) -> Vec2 {
    let u = 1.0 - s;
    p0.scale(u * u).add(c.scale(2.0 * u * s)).add(p1.scale(s * s))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarginType {
    Entire,
    Serrate,       // forward-pointing sawtooth teeth
    Dentate,       // symmetric outward-pointing teeth
    Crenate,       // rounded scallops
    DoublySerrate, // big teeth that are themselves serrated (birch, elm)
    Spinose,       // sharp, widely-spaced spines (holly)
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
    pub fn doubly_serrate() -> Self {
        Margin { kind: MarginType::DoublySerrate, n_teeth: 9, amp: 0.34 }
    }
    pub fn spinose() -> Self {
        Margin { kind: MarginType::Spinose, n_teeth: 7, amp: 0.5 }
    }
}

/// Pinnate lobing: a low-frequency modulation of the half-width whose sinuses
/// cut toward the midrib. `n` lobes per side, `depth ∈ [0,1]` (0 unlobed, →1
/// sinus reaches the midrib), `sharp` shapes the sinus (>1 narrower sinuses /
/// broader lobes).
#[derive(Clone, Copy, Debug)]
pub struct Lobing {
    pub n: usize,
    pub depth: Scalar,
    pub sharp: Scalar,
}

impl Lobing {
    pub fn none() -> Self {
        Lobing { n: 0, depth: 0.0, sharp: 1.0 }
    }
    pub fn pinnate(n: usize, depth: Scalar) -> Self {
        Lobing { n, depth, sharp: 1.1 }
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
    pub lobing: Lobing,
    /// Basal lobes below the petiole, depth as a fraction of length (0 = none,
    /// cordate / heart-shaped base when > 0).
    pub cordate: Scalar,
    /// Base asymmetry ∈ [-1, 1]: one half-base wider than the other (oblique
    /// base, e.g. elm). 0 = bilaterally symmetric.
    pub asymmetry: Scalar,
    /// Basal lobe direction (only with cordate > 0): 0 = hastate (out),
    /// 0.5 = cordate (down-out), 1 = sagittate (down, arrowhead).
    pub base_dir: Scalar,
    /// Apical notch depth as a fraction of length (0 = pointed; > 0 =
    /// emarginate / obcordate notched tip).
    pub apex_notch: Scalar,
}

impl Blade {
    pub fn shape(length: Scalar, half_width: Scalar, a: Scalar, b: Scalar) -> Self {
        let tstar = a / (a + b);
        let peak = tstar.powf(a) * (1.0 - tstar).powf(b);
        Blade {
            length,
            half_width,
            a,
            b,
            peak,
            margin: Margin::entire(),
            lobing: Lobing::none(),
            cordate: 0.0,
            asymmetry: 0.0,
            base_dir: 0.5,
            apex_notch: 0.0,
        }
    }

    pub fn with_margin(mut self, margin: Margin) -> Self {
        self.margin = margin;
        self
    }

    pub fn with_lobing(mut self, lobing: Lobing) -> Self {
        self.lobing = lobing;
        self
    }

    pub fn with_cordate(mut self, depth: Scalar) -> Self {
        self.cordate = depth;
        self
    }

    pub fn with_asymmetry(mut self, asym: Scalar) -> Self {
        self.asymmetry = asym;
        self
    }

    pub fn with_base_dir(mut self, dir: Scalar) -> Self {
        self.base_dir = dir;
        self
    }

    pub fn with_apex_notch(mut self, notch: Scalar) -> Self {
        self.apex_notch = notch;
        self
    }

    /// Side-aware half-width: applies base asymmetry (`side` = +1 right / −1 left).
    fn half_width_side(&self, t: Scalar, side: Scalar) -> Scalar {
        let w = self.half_width_at(t);
        if self.asymmetry == 0.0 {
            return w;
        }
        let base_emphasis = (1.0 - t / 0.35).max(0.0); // strongest at the base
        w * (1.0 + side * self.asymmetry * base_emphasis)
    }

    /// Pinnately-lobed, oak-like (rounded lobes).
    pub fn oak() -> Self {
        Blade::shape(11.0, 3.0, 1.5, 1.8).with_lobing(Lobing::pinnate(6, 0.55))
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

    /// Half-width at normalized midrib position `t ∈ [0, 1]`, including lobing.
    pub fn half_width_at(&self, t: Scalar) -> Scalar {
        if t <= 0.0 {
            return 0.0;
        }
        // A notched tip needs nonzero width at the apex so it can split. The
        // floor ramps up from the apical sinus (1 − apex_notch) to the tip.
        let apex_floor = if self.apex_notch > 0.0 {
            let sinus_t = 1.0 - self.apex_notch;
            0.16 * self.half_width * ((t - sinus_t) / self.apex_notch).clamp(0.0, 1.0)
        } else {
            0.0
        };
        if t >= 1.0 {
            return apex_floor;
        }
        let smooth = self.half_width * t.powf(self.a) * (1.0 - t).powf(self.b) / self.peak;
        smooth.max(apex_floor) * self.lobe_factor(t)
    }

    /// Lobing multiplier ∈ [1−depth, 1]: 1 at lobe tips, 1−depth at sinuses.
    fn lobe_factor(&self, t: Scalar) -> Scalar {
        if self.lobing.n == 0 {
            return 1.0;
        }
        let (lo, hi) = (LOBE_LO, LOBE_HI);
        if t <= lo || t >= hi {
            return 1.0;
        }
        let phase = (t - lo) / (hi - lo) * self.lobing.n as Scalar;
        let frac = phase.fract();
        // sinus = 1 at lobe boundaries (frac 0/1), 0 at lobe centre (frac 0.5)
        let sinus = (0.5 * (1.0 + (2.0 * PI * frac).cos())).powf(self.lobing.sharp);
        1.0 - self.lobing.depth * sinus
    }

    /// Midrib positions of the lobe tips (empty if unlobed). One secondary vein
    /// is routed to each.
    pub fn lobe_centers(&self) -> Vec<Scalar> {
        if self.lobing.n == 0 {
            return Vec::new();
        }
        (0..self.lobing.n)
            .map(|k| LOBE_LO + (k as Scalar + 0.5) / self.lobing.n as Scalar * (LOBE_HI - LOBE_LO))
            .collect()
    }

    /// Is point `p` inside the smooth blade body? (Teeth project outward and
    /// basal cordate lobes hang below — both ignored here, so sources sampled
    /// in the body are always inside the real leaf.)
    pub fn contains(&self, p: Vec2) -> bool {
        if p.y < 0.0 || p.y > self.length {
            return false;
        }
        let t = p.y / self.length;
        let side = if p.x >= 0.0 { 1.0 } else { -1.0 };
        if p.x.abs() > self.half_width_side(t, side) {
            return false;
        }
        // Carve out the apical notch (the V cut into the top centre).
        if self.apex_notch > 0.0 {
            let sinus_t = 1.0 - self.apex_notch;
            if t > sinus_t {
                let x_notch = 0.16 * self.half_width * (t - sinus_t) / self.apex_notch;
                if p.x.abs() < x_notch {
                    return false;
                }
            }
        }
        true
    }

    fn smooth_pt(&self, t: Scalar, side: Scalar) -> Vec2 {
        Vec2::new(side * self.half_width_side(t, side), t * self.length)
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
            // big sawtooth with three smaller sawteeth riding on it
            MarginType::DoublySerrate => 0.6 * ph + 0.4 * (ph * 3.0).fract(),
            // sharp narrow spine in the middle of each (otherwise flat) period
            MarginType::Spinose => (1.0 - (2.0 * ph - 1.0).abs() * 2.6).max(0.0),
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

    /// Outline polygon (with teeth + optional cordate base): right chain
    /// base→apex, then left apex→base, with basal lobes spliced in when cordate.
    pub fn outline(&self, samples: usize) -> Vec<Vec2> {
        let mut pts = Vec::with_capacity(2 * samples + 28);
        let t0 = if self.cordate > 0.0 { 0.12 } else { 0.0 };
        if self.cordate > 0.0 {
            pts.extend(self.base_lobe(1.0, t0, true)); // sinus → right blade base
        }
        for i in 0..=samples {
            let t = t0 + (1.0 - t0) * i as Scalar / samples as Scalar;
            pts.push(self.margin_pt(t, 1.0));
        }
        if self.apex_notch > 0.0 {
            pts.push(Vec2::new(0.0, self.length * (1.0 - self.apex_notch))); // apical sinus
        }
        for i in (0..=samples).rev() {
            let t = t0 + (1.0 - t0) * i as Scalar / samples as Scalar;
            pts.push(self.margin_pt(t, -1.0));
        }
        if self.cordate > 0.0 {
            pts.extend(self.base_lobe(-1.0, t0, false)); // left blade base → sinus
        }
        pts
    }

    /// Bézier control point for a basal lobe (direction set by `base_dir`).
    fn base_lobe_ctrl(&self, side: Scalar, t0: Scalar) -> Vec2 {
        let bw = self.half_width_side(t0, side);
        // base_dir: 0 hastate (out, shallow) → 1 sagittate (down, deep)
        let cx = 1.9 + (0.45 - 1.9) * self.base_dir;
        let cy = 0.55 + (1.7 - 0.55) * self.base_dir;
        Vec2::new(side * bw * cx, -self.cordate * self.length * cy)
    }

    /// Tip of a basal lobe (for routing a vein into it), if cordate.
    pub fn basal_lobe_tip(&self, side: Scalar) -> Option<Vec2> {
        if self.cordate <= 0.0 {
            return None;
        }
        let t0 = 0.12;
        let bw = self.half_width_side(t0, side);
        let blade_base = Vec2::new(side * bw, t0 * self.length);
        Some(bez(Vec2::new(0.0, 0.0), self.base_lobe_ctrl(side, t0), blade_base, 0.5))
    }

    /// One basal lobe curve (quadratic Bézier bulging below the petiole).
    fn base_lobe(&self, side: Scalar, t0: Scalar, entering: bool) -> Vec<Vec2> {
        let sinus = Vec2::new(0.0, 0.0);
        let bw = self.half_width_side(t0, side);
        let blade_base = Vec2::new(side * bw, t0 * self.length);
        let ctrl = self.base_lobe_ctrl(side, t0);
        let n = 12;
        let mut v = Vec::with_capacity(n);
        if entering {
            for k in 0..n {
                v.push(bez(sinus, ctrl, blade_base, k as Scalar / n as Scalar));
            }
        } else {
            for k in 1..=n {
                v.push(bez(blade_base, ctrl, sinus, k as Scalar / n as Scalar));
            }
        }
        v
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
