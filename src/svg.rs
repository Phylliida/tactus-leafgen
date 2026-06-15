//! Minimal SVG renderer for a generated leaf (no dependencies).

use crate::blade::Blade;
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

pub fn render(blade: &Blade, veins: &VeinGraph, opts: &RenderOpts) -> String {
    let petiole_len = blade.length * opts.petiole_frac;
    let world_h = blade.length + petiole_len;
    let scale = opts.target_height_px / world_h;
    let pad = opts.pad_px;
    let ext = blade.half_extent();
    let minx = -ext;
    let maxy = blade.length;
    let svg_w = (2.0 * ext) * scale + 2.0 * pad;
    let svg_h = world_h * scale + 2.0 * pad;

    let tx = |p: Vec2| -> (Scalar, Scalar) { ((p.x - minx) * scale + pad, (maxy - p.y) * scale + pad) };

    let mut s = String::new();
    s.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{:.0}\" height=\"{:.0}\" viewBox=\"0 0 {:.0} {:.0}\">\n",
        svg_w, svg_h, svg_w, svg_h
    ));
    s.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#fbfdf6\"/>\n");

    let outline = blade.outline(700);
    s.push_str("<path d=\"");
    for (i, p) in outline.iter().enumerate() {
        let (x, y) = tx(*p);
        s.push_str(&format!("{}{:.2} {:.2} ", if i == 0 { "M" } else { "L" }, x, y));
    }
    s.push_str("Z\" fill=\"#e7f3d4\" stroke=\"#5a7d2a\" stroke-width=\"2\"/>\n");

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
