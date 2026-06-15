//! Anatomically-grounded flowers, following the floral-formula / floral-diagram
//! approach (Ijiri et al. 2005; see `docs/flower-research.md`).
//!
//! A flower is concentric **whorls** of organs around a receptacle, drawn from
//! outside in: calyx (sepals) → corolla (petals) → androecium (stamens) →
//! gynoecium (carpels/pistil). Organs are placed by **phyllotaxis** — whorled
//! (even spacing) or spiral (Vogel's golden-angle, for composite heads). Each
//! organ reuses the leaf `Blade` shape; the colored `Scene`/`Lamina` renderer
//! draws it. Output is a 2D top-view floral diagram with real geometry.

use crate::blade::Blade;
use crate::svg::{Lamina, Rgb, Scene};
use crate::vec2::Vec2;
use crate::venation::VeinGraph;
use crate::Scalar;
use std::f64::consts::PI;

/// Golden angle, 2π(1 − 1/φ) ≈ 137.508° — Vogel's divergence angle.
const GOLDEN_ANGLE: Scalar = 2.399_963_229_728_653;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OrganKind {
    Sepal,
    Petal,
    Stamen,     // filament (vein) + anther (dot)
    Carpel,     // central pistil: ovary dome + stigma lobes
    DiskFloret, // tiny floret of a capitulum (Asteraceae head)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phyllotaxis {
    Whorled, // `count` organs evenly around a ring
    Spiral,  // Vogel golden-angle, filling a disk (capitulum)
}

#[derive(Clone, Copy)]
pub struct Whorl {
    pub kind: OrganKind,
    pub count: usize,
    /// Attachment radius from the center (ring radius / disk extent for spiral).
    pub radius: Scalar,
    /// Organ shape. `length` = petal/sepal length or stamen filament length;
    /// `half_width` = petal half-width or anther/floret radius.
    pub organ: Blade,
    pub fill: Rgb,
    pub stroke: Rgb,
    pub phyllotaxis: Phyllotaxis,
    /// Angular offset (radians) — used to alternate adjacent whorls.
    pub angle_offset: Scalar,
}

pub struct FloralFormula {
    /// Outer→inner draw order (calyx, corolla, androecium, gynoecium).
    pub whorls: Vec<Whorl>,
    pub receptacle_radius: Scalar,
    pub receptacle_fill: Rgb,
    pub stem_len: Scalar,
}

fn place(base: Vec2, d: Vec2, p: Vec2) -> Vec2 {
    Vec2::new(base.x + p.x * d.y + p.y * d.x, base.y - p.x * d.x + p.y * d.y)
}

fn disk(center: Vec2, r: Scalar, n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|k| {
            let a = 2.0 * PI * k as Scalar / n as Scalar;
            Vec2::new(center.x + r * a.cos(), center.y + r * a.sin())
        })
        .collect()
}

fn darken(c: Rgb) -> Rgb {
    [(c[0] as Scalar * 0.78) as u8, (c[1] as Scalar * 0.78) as u8, (c[2] as Scalar * 0.78) as u8]
}

fn lerp_rgb(a: Rgb, b: Rgb, t: Scalar) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    [
        (a[0] as Scalar + (b[0] as Scalar - a[0] as Scalar) * t) as u8,
        (a[1] as Scalar + (b[1] as Scalar - a[1] as Scalar) * t) as u8,
        (a[2] as Scalar + (b[2] as Scalar - a[2] as Scalar) * t) as u8,
    ]
}

pub fn build(f: &FloralFormula) -> Scene {
    let mut laminae: Vec<Lamina> = Vec::new();
    let mut veins = VeinGraph::new();

    if f.receptacle_radius > 0.0 {
        laminae.push(Lamina::new(disk(Vec2::zero(), f.receptacle_radius, 40), f.receptacle_fill, darken(f.receptacle_fill)));
    }

    for w in &f.whorls {
        match w.phyllotaxis {
            Phyllotaxis::Spiral => spiral_florets(w, &mut laminae),
            Phyllotaxis::Whorled => whorled(w, &mut laminae, &mut veins),
        }
    }

    Scene {
        laminae,
        veins,
        vein_base: [140, 112, 55], // muted gold filaments/midribs (not near-white)
        petiole_len: f.stem_len,
        petiole_color: [70, 110, 35],
    }
}

fn whorled(w: &Whorl, laminae: &mut Vec<Lamina>, veins: &mut VeinGraph) {
    let n = w.count.max(1);
    for i in 0..n {
        let th = w.angle_offset + 2.0 * PI * i as Scalar / n as Scalar;
        let d = Vec2::new(th.cos(), th.sin());
        match w.kind {
            OrganKind::Sepal | OrganKind::Petal => {
                let base = d.scale(w.radius);
                let ol: Vec<Vec2> = w.organ.outline(80).iter().map(|q| place(base, d, *q)).collect();
                laminae.push(Lamina::new(ol, w.fill, w.stroke));
                if w.kind == OrganKind::Petal {
                    let vl = w.organ.length * 0.8;
                    let mut prev = veins.add_node(base, 1);
                    for k in 1..=6 {
                        prev = veins.add_child(prev, base.add(d.scale(vl * k as Scalar / 6.0)), 2);
                    }
                }
            }
            OrganKind::Stamen => {
                let inner = d.scale(w.radius);
                let outer = d.scale(w.radius + w.organ.length);
                let mut prev = veins.add_node(inner, 2);
                for k in 1..=4 {
                    prev = veins.add_child(prev, inner.add(outer.sub(inner).scale(k as Scalar / 4.0)), 3);
                }
                laminae.push(Lamina::new(disk(outer, w.organ.half_width, 8), w.fill, w.stroke));
            }
            OrganKind::Carpel => {
                if i == 0 {
                    // central ovary/stigma dome
                    laminae.push(Lamina::new(disk(Vec2::zero(), w.radius * 1.4, 20), w.fill, w.stroke));
                }
                // stigma lobe
                laminae.push(Lamina::new(disk(d.scale(w.radius), w.organ.half_width, 6), darken(w.fill), w.stroke));
            }
            OrganKind::DiskFloret => {}
        }
    }
}

fn spiral_florets(w: &Whorl, laminae: &mut Vec<Lamina>) {
    let n = w.count.max(1);
    for k in 0..n {
        let a = k as Scalar * GOLDEN_ANGLE + w.angle_offset;
        let t = ((k as Scalar + 0.5) / n as Scalar).sqrt(); // 0 center → 1 rim
        let r = w.radius * t;
        let pos = Vec2::new(r * a.cos(), r * a.sin());
        let col = lerp_rgb(w.stroke, w.fill, t); // center darker (immature) → rim open
        laminae.push(Lamina::new(disk(pos, w.organ.half_width, 6), col, darken(col)));
    }
}

// ---- presets (anatomically-styled floral formulas) ----

impl FloralFormula {
    /// 5 yellow petals + 5 green sepals + a ring of stamens + central carpels.
    pub fn buttercup() -> Self {
        FloralFormula {
            receptacle_radius: 0.72,
            receptacle_fill: [120, 150, 60],
            stem_len: 6.0,
            whorls: vec![
                Whorl { kind: OrganKind::Sepal, count: 5, radius: 0.55, organ: Blade::shape(2.4, 1.0, 1.7, 1.4), fill: [150, 175, 80], stroke: [110, 135, 55], phyllotaxis: Phyllotaxis::Whorled, angle_offset: PI / 5.0 },
                Whorl { kind: OrganKind::Petal, count: 5, radius: 0.6, organ: Blade::shape(3.4, 1.7, 2.0, 1.3), fill: [255, 221, 50], stroke: [220, 172, 20], phyllotaxis: Phyllotaxis::Whorled, angle_offset: 0.0 },
                Whorl { kind: OrganKind::Stamen, count: 28, radius: 0.42, organ: Blade::shape(0.5, 0.14, 1.0, 1.0), fill: [245, 205, 60], stroke: [180, 140, 30], phyllotaxis: Phyllotaxis::Whorled, angle_offset: 0.0 },
                Whorl { kind: OrganKind::Carpel, count: 12, radius: 0.26, organ: Blade::shape(0.3, 0.1, 1.0, 1.0), fill: [130, 160, 65], stroke: [95, 120, 45], phyllotaxis: Phyllotaxis::Whorled, angle_offset: 0.0 },
            ],
        }
    }

    /// 5 pink petals + green sepals + a gold ring of stamens + center.
    pub fn rose() -> Self {
        FloralFormula {
            receptacle_radius: 0.66,
            receptacle_fill: [150, 170, 70],
            stem_len: 6.0,
            whorls: vec![
                Whorl { kind: OrganKind::Sepal, count: 5, radius: 0.6, organ: Blade::shape(2.6, 0.9, 1.6, 1.6), fill: [120, 155, 70], stroke: [90, 120, 50], phyllotaxis: Phyllotaxis::Whorled, angle_offset: PI / 5.0 },
                Whorl { kind: OrganKind::Petal, count: 5, radius: 0.55, organ: Blade::shape(3.0, 2.0, 1.7, 1.3), fill: [240, 150, 185], stroke: [205, 110, 150], phyllotaxis: Phyllotaxis::Whorled, angle_offset: 0.0 },
                Whorl { kind: OrganKind::Stamen, count: 26, radius: 0.45, organ: Blade::shape(0.8, 0.14, 1.0, 1.0), fill: [240, 200, 80], stroke: [190, 150, 40], phyllotaxis: Phyllotaxis::Whorled, angle_offset: 0.0 },
                Whorl { kind: OrganKind::Carpel, count: 1, radius: 0.22, organ: Blade::shape(0.3, 0.18, 1.0, 1.0), fill: [220, 200, 90], stroke: [180, 150, 50], phyllotaxis: Phyllotaxis::Whorled, angle_offset: 0.0 },
            ],
        }
    }

    /// Lily: 6 tepals + 6 prominent stamens (brown anthers) + central pistil.
    pub fn lily() -> Self {
        FloralFormula {
            receptacle_radius: 0.45,
            receptacle_fill: [150, 180, 90],
            stem_len: 6.0,
            whorls: vec![
                Whorl { kind: OrganKind::Petal, count: 6, radius: 0.4, organ: Blade::shape(4.6, 1.15, 1.7, 1.5), fill: [248, 232, 240], stroke: [210, 160, 185], phyllotaxis: Phyllotaxis::Whorled, angle_offset: 0.0 },
                Whorl { kind: OrganKind::Stamen, count: 6, radius: 0.3, organ: Blade::shape(2.6, 0.28, 1.0, 1.0), fill: [150, 80, 45], stroke: [110, 55, 30], phyllotaxis: Phyllotaxis::Whorled, angle_offset: PI / 6.0 },
                Whorl { kind: OrganKind::Carpel, count: 1, radius: 0.3, organ: Blade::shape(0.4, 0.2, 1.0, 1.0), fill: [170, 200, 110], stroke: [120, 150, 70], phyllotaxis: Phyllotaxis::Whorled, angle_offset: 0.0 },
            ],
        }
    }

    /// Composite head (Asteraceae): ray florets (petals) + spiral disk florets.
    fn capitulum(disk_r: Scalar, n_rays: usize, ray: Blade, ray_fill: Rgb, ray_stroke: Rgb, n_florets: usize, floret_rim: Rgb, floret_center: Rgb, floret_size: Scalar) -> Self {
        FloralFormula {
            receptacle_radius: disk_r,
            receptacle_fill: floret_center,
            stem_len: 7.0,
            whorls: vec![
                Whorl { kind: OrganKind::Petal, count: n_rays, radius: disk_r * 0.95, organ: ray, fill: ray_fill, stroke: ray_stroke, phyllotaxis: Phyllotaxis::Whorled, angle_offset: 0.0 },
                Whorl { kind: OrganKind::DiskFloret, count: n_florets, radius: disk_r, organ: Blade::shape(0.2, floret_size, 1.0, 1.0), fill: floret_rim, stroke: floret_center, phyllotaxis: Phyllotaxis::Spiral, angle_offset: 0.0 },
            ],
        }
    }

    /// Daisy: white ray florets + yellow disk.
    pub fn daisy() -> Self {
        Self::capitulum(1.7, 21, Blade::shape(4.2, 0.5, 1.7, 1.5), [253, 253, 250], [176, 178, 168], 230, [255, 205, 55], [200, 150, 40], 0.12)
    }

    /// Sunflower: yellow ray florets + a large brown spiral disk.
    pub fn sunflower() -> Self {
        Self::capitulum(2.8, 28, Blade::shape(5.0, 0.7, 1.8, 1.4), [255, 200, 40], [225, 165, 25], 600, [205, 150, 55], [95, 65, 30], 0.13)
    }

    /// Override the corolla (petal) whorl's organ count (studio slider).
    pub fn with_petal_count(mut self, n: usize) -> Self {
        if let Some(w) = self.whorls.iter_mut().find(|w| w.kind == OrganKind::Petal) {
            w.count = n.max(3);
        }
        self
    }
}
