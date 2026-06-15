//! Minimal SVG renderer for a generated leaf (no dependencies).

use crate::vec2::Vec2;
use crate::venation::VeinGraph;
use crate::Scalar;

pub struct RenderOpts {
    /// Target on-screen height of the lamina, in px.
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

/// Stroke width for a vein of the given order.
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

/// Vein colour for the given order (majors darker, minors lighter).
pub fn vein_color(order: u8) -> (u8, u8, u8) {
    match order {
        0 | 1 => (58, 92, 22),
        2 => (84, 116, 40),
        _ => (110, 142, 66),
    }
}

/// Bounding box (minx, miny, maxx, maxy) of the laminae + veins + petiole tip.
pub fn scene_bounds(laminae: &[Vec<Vec2>], veins: &VeinGraph, petiole_len: Scalar) -> (Scalar, Scalar, Scalar, Scalar) {
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
    for poly in laminae {
        for p in poly {
            acc(*p);
        }
    }
    for p in &veins.nodes {
        acc(*p);
    }
    acc(Vec2::new(0.0, 0.0));
    acc(Vec2::new(0.0, -petiole_len));
    (mnx, mny, mxx, mxy)
}

/// Render a leaf from its lamina polygons + vein graph (shape-agnostic: simple,
/// lobed, palmate, or compound — compound leaves pass one polygon per leaflet).
/// The petiole runs from (0,0) down by `petiole_len`.
pub fn render(laminae: &[Vec<Vec2>], veins: &VeinGraph, petiole_len: Scalar, opts: &RenderOpts) -> String {
    let (minx, miny, maxx, maxy) = scene_bounds(laminae, veins, petiole_len);
    let world_h = (maxy - miny).max(1e-6);
    let world_w = (maxx - minx).max(1e-6);
    let scale = opts.target_height_px / world_h;
    let pad = opts.pad_px;
    let svg_w = world_w * scale + 2.0 * pad;
    let svg_h = world_h * scale + 2.0 * pad;

    let tx = |p: Vec2| -> (Scalar, Scalar) { ((p.x - minx) * scale + pad, (maxy - p.y) * scale + pad) };

    let mut s = String::new();
    s.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{:.0}\" height=\"{:.0}\" viewBox=\"0 0 {:.0} {:.0}\">\n",
        svg_w, svg_h, svg_w, svg_h
    ));
    s.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#fbfdf6\"/>\n");

    for poly in laminae {
        s.push_str("<path d=\"");
        for (i, p) in poly.iter().enumerate() {
            let (x, y) = tx(*p);
            s.push_str(&format!("{}{:.2} {:.2} ", if i == 0 { "M" } else { "L" }, x, y));
        }
        s.push_str("Z\" fill=\"#e7f3d4\" stroke=\"#5a7d2a\" stroke-width=\"2\"/>\n");
    }

    let (bx, by) = tx(Vec2::new(0.0, 0.0));
    let (px, py) = tx(Vec2::new(0.0, -petiole_len));
    s.push_str(&format!(
        "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"#3a5c16\" stroke-width=\"{:.2}\" stroke-linecap=\"round\"/>\n",
        bx, by, px, py, opts.max_vein_px
    ));

    // Draw finest veins first so majors render on top.
    let mut order: Vec<usize> = (0..veins.edges.len()).collect();
    order.sort_by(|&i, &j| veins.edge_order[j].cmp(&veins.edge_order[i]));
    s.push_str("<g fill=\"none\" stroke-linecap=\"round\">\n");
    for &e in &order {
        let (a, b) = veins.edges[e];
        let ord = veins.edge_order[e];
        let (r, gg, bl) = vein_color(ord);
        let w = vein_width(ord, opts);
        let (x1, y1) = tx(veins.nodes[a]);
        let (x2, y2) = tx(veins.nodes[b]);
        s.push_str(&format!(
            "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"rgb({},{},{})\" stroke-width=\"{:.2}\"/>\n",
            x1, y1, x2, y2, r, gg, bl, w
        ));
    }
    s.push_str("</g>\n</svg>\n");
    s
}
