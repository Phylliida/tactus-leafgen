//! Minimal SVG renderer for a generated leaf (no dependencies).

use crate::blade::Blade;
use crate::venation::{subtree_sizes, VeinNode};
use crate::vec2::Vec2;
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
            target_height_px: 820.0,
            pad_px: 24.0,
            petiole_frac: 0.14,
            min_vein_px: 0.6,
            max_vein_px: 5.0,
        }
    }
}

pub fn render(blade: &Blade, nodes: &[VeinNode], opts: &RenderOpts) -> String {
    let petiole_len = blade.length * opts.petiole_frac;
    let world_h = blade.length + petiole_len;
    let scale = opts.target_height_px / world_h;
    let pad = opts.pad_px;

    let minx = -blade.half_width;
    let maxy = blade.length;
    let svg_w = (2.0 * blade.half_width) * scale + 2.0 * pad;
    let svg_h = world_h * scale + 2.0 * pad;

    // World → screen (flip y so the apex is at the top).
    let tx = |p: Vec2| -> (Scalar, Scalar) {
        ((p.x - minx) * scale + pad, (maxy - p.y) * scale + pad)
    };

    let mut s = String::new();
    s.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{:.0}\" height=\"{:.0}\" \
         viewBox=\"0 0 {:.0} {:.0}\">\n",
        svg_w, svg_h, svg_w, svg_h
    ));
    s.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#fbfdf6\"/>\n");

    // Lamina outline.
    let outline = blade.outline(160);
    s.push_str("<path d=\"");
    for (i, p) in outline.iter().enumerate() {
        let (x, y) = tx(*p);
        s.push_str(&format!("{}{:.2} {:.2} ", if i == 0 { "M" } else { "L" }, x, y));
    }
    s.push_str("Z\" fill=\"#e7f3d4\" stroke=\"#5a7d2a\" stroke-width=\"2\"/>\n");

    // Petiole stub.
    let (bx, by) = tx(Vec2::new(0.0, 0.0));
    let (px, py) = tx(Vec2::new(0.0, -petiole_len));
    s.push_str(&format!(
        "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" \
         stroke=\"#5a7d2a\" stroke-width=\"{:.2}\" stroke-linecap=\"round\"/>\n",
        bx, by, px, py, opts.max_vein_px
    ));

    // Veins, tapered by subtree size.
    let sizes = subtree_sizes(nodes);
    let max_size = sizes.first().copied().unwrap_or(1).max(1) as Scalar;
    s.push_str("<g stroke=\"#3f6418\" fill=\"none\" stroke-linecap=\"round\">\n");
    for (i, node) in nodes.iter().enumerate() {
        let Some(p) = node.parent else { continue };
        let frac = (sizes[i] as Scalar / max_size).powf(0.45);
        let w = opts.min_vein_px + (opts.max_vein_px - opts.min_vein_px) * frac;
        let (x1, y1) = tx(node.pos);
        let (x2, y2) = tx(nodes[p].pos);
        s.push_str(&format!(
            "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke-width=\"{:.2}\"/>\n",
            x1, y1, x2, y2, w
        ));
    }
    s.push_str("</g>\n</svg>\n");
    s
}
