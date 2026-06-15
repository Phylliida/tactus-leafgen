//! Minimal SVG renderer (no dependencies). Renders a `Scene`: a set of colored
//! filled regions (`Lamina`s — leaf blades or flower petals/centers) plus a
//! vein graph and a petiole/stem.

use crate::vec2::Vec2;
use crate::venation::VeinGraph;
use crate::Scalar;
use std::fmt::Write as _;

pub type Rgb = [u8; 3];

/// A filled region: a leaf lamina, a flower petal, a floral disk, etc.
pub struct Lamina {
    pub points: Vec<Vec2>,
    pub fill: Rgb,
    pub stroke: Rgb,
}

impl Lamina {
    /// A leaf-green lamina (the default for leaves).
    pub fn leaf(points: Vec<Vec2>) -> Self {
        Lamina { points, fill: [231, 243, 212], stroke: [90, 125, 42] }
    }
    pub fn new(points: Vec<Vec2>, fill: Rgb, stroke: Rgb) -> Self {
        Lamina { points, fill, stroke }
    }
}

/// A complete drawable: colored regions + veins + a petiole/stem from (0,0) down.
pub struct Scene {
    pub laminae: Vec<Lamina>,
    pub veins: VeinGraph,
    pub vein_base: Rgb,
    pub petiole_len: Scalar,
    pub petiole_color: Rgb,
}

impl Scene {
    /// A green leaf scene from outline polygons + veins.
    pub fn leaf(outlines: Vec<Vec<Vec2>>, veins: VeinGraph, petiole_len: Scalar) -> Self {
        Scene {
            laminae: outlines.into_iter().map(Lamina::leaf).collect(),
            veins,
            vein_base: [58, 92, 22],
            petiole_len,
            petiole_color: [58, 92, 22],
        }
    }
}

pub struct RenderOpts {
    /// Target on-screen height, in px.
    pub target_height_px: Scalar,
    pub pad_px: Scalar,
    pub petiole_frac: Scalar,
    pub min_vein_px: Scalar,
    pub max_vein_px: Scalar,
}

impl Default for RenderOpts {
    fn default() -> Self {
        RenderOpts {
            target_height_px: 1500.0,
            pad_px: 30.0,
            petiole_frac: 0.14,
            min_vein_px: 0.5,
            max_vein_px: 5.0,
        }
    }
}

pub fn vein_width(order: u8, o: &RenderOpts) -> Scalar {
    let f = match order {
        0 => 1.0,
        1 => 0.6,
        2 => 0.32,
        3 => 0.2,
        _ => 0.13,
    };
    (o.max_vein_px * f).max(o.min_vein_px)
}

fn tint(c: Rgb, f: Scalar) -> Rgb {
    let f = f.clamp(0.0, 1.0);
    [
        (c[0] as Scalar + (255.0 - c[0] as Scalar) * f) as u8,
        (c[1] as Scalar + (255.0 - c[1] as Scalar) * f) as u8,
        (c[2] as Scalar + (255.0 - c[2] as Scalar) * f) as u8,
    ]
}

/// Vein colour for a given order: the scene's base, lightened for finer orders.
pub fn vein_color(base: Rgb, order: u8) -> Rgb {
    let f = match order {
        0 | 1 => 0.0,
        2 => 0.3,
        3 => 0.5,
        _ => 0.62,
    };
    tint(base, f)
}

/// Bounding box (minx, miny, maxx, maxy) of the scene.
pub fn scene_bounds(scene: &Scene) -> (Scalar, Scalar, Scalar, Scalar) {
    let mut mnx = Scalar::INFINITY;
    let mut mny = Scalar::INFINITY;
    let mut mxx = -Scalar::INFINITY;
    let mut mxy = -Scalar::INFINITY;
    let mut acc = |p: Vec2| {
        mnx = mnx.min(p.x);
        mny = mny.min(p.y);
        mxx = mxx.max(p.x);
        mxy = mxy.max(p.y);
    };
    for lam in &scene.laminae {
        for p in &lam.points {
            acc(*p);
        }
    }
    for p in &scene.veins.nodes {
        acc(*p);
    }
    acc(Vec2::new(0.0, 0.0));
    acc(Vec2::new(0.0, -scene.petiole_len));
    (mnx, mny, mxx, mxy)
}

pub fn render(scene: &Scene, opts: &RenderOpts) -> String {
    let (minx, miny, maxx, maxy) = scene_bounds(scene);
    let world_h = (maxy - miny).max(1e-6);
    let world_w = (maxx - minx).max(1e-6);
    let scale = opts.target_height_px / world_h;
    let pad = opts.pad_px;
    let svg_w = world_w * scale + 2.0 * pad;
    let svg_h = world_h * scale + 2.0 * pad;

    let tx = |p: Vec2| -> (Scalar, Scalar) { ((p.x - minx) * scale + pad, (maxy - p.y) * scale + pad) };
    let rgb = |c: Rgb| format!("rgb({},{},{})", c[0], c[1], c[2]);

    let mut s = String::new();
    s.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{:.0}\" height=\"{:.0}\" viewBox=\"0 0 {:.0} {:.0}\">\n",
        svg_w, svg_h, svg_w, svg_h
    ));
    s.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#fbfdf6\"/>\n");

    // Petiole/stem first (behind everything).
    let (bx, by) = tx(Vec2::new(0.0, 0.0));
    let (pxp, pyp) = tx(Vec2::new(0.0, -scene.petiole_len));
    let _ = write!(
        s,
        "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" stroke-width=\"{:.2}\" stroke-linecap=\"round\"/>\n",
        bx, by, pxp, pyp, rgb(scene.petiole_color), opts.max_vein_px
    );

    for lam in &scene.laminae {
        s.push_str("<path d=\"");
        for (i, p) in lam.points.iter().enumerate() {
            let (x, y) = tx(*p);
            let _ = write!(s, "{}{:.2} {:.2} ", if i == 0 { "M" } else { "L" }, x, y);
        }
        let _ = write!(s, "Z\" fill=\"{}\" stroke=\"{}\" stroke-width=\"2\"/>\n", rgb(lam.fill), rgb(lam.stroke));
    }

    // Veins as merged polylines, finest first so majors render on top.
    let mut polys = scene.veins.polylines();
    polys.sort_by(|x, y| y.0.cmp(&x.0));
    s.push_str("<g fill=\"none\" stroke-linecap=\"round\" stroke-linejoin=\"round\">\n");
    for (ord, chain) in &polys {
        let w = vein_width(*ord, opts);
        let _ = write!(s, "<polyline stroke=\"{}\" stroke-width=\"{:.2}\" points=\"", rgb(vein_color(scene.vein_base, *ord)), w);
        for &ni in chain {
            let (x, y) = tx(scene.veins.nodes[ni]);
            let _ = write!(s, "{:.1},{:.1} ", x, y);
        }
        s.push_str("\"/>\n");
    }
    s.push_str("</g>\n</svg>\n");
    s
}
