//! Dependency-free software rasterizer + PNG writer.
//!
//! We render our own geometry (anti-aliased thick lines + scanline polygon
//! fill) into an RGB buffer and emit an uncompressed-DEFLATE PNG. No external
//! crates, no system rasterizer needed — so we can always produce a viewable
//! image of a generated leaf.

use crate::svg::{vein_color, vein_width, RenderOpts, Scene};
use crate::vec2::Vec2;
use crate::Scalar;

pub type Rgb = [u8; 3];

pub struct Canvas {
    pub w: usize,
    pub h: usize,
    px: Vec<u8>, // RGB, row-major
}

impl Canvas {
    pub fn new(w: usize, h: usize, bg: Rgb) -> Self {
        let mut px = vec![0u8; w * h * 3];
        for i in 0..w * h {
            px[3 * i] = bg[0];
            px[3 * i + 1] = bg[1];
            px[3 * i + 2] = bg[2];
        }
        Canvas { w, h, px }
    }

    #[inline]
    fn blend(&mut self, x: usize, y: usize, c: Rgb, a: Scalar) {
        if x >= self.w || y >= self.h || a <= 0.0 {
            return;
        }
        let a = a.clamp(0.0, 1.0);
        let i = 3 * (y * self.w + x);
        for k in 0..3 {
            let dst = self.px[i + k] as Scalar;
            self.px[i + k] = (c[k] as Scalar * a + dst * (1.0 - a)).round() as u8;
        }
    }

    /// Anti-aliased line of total width `width` from `p0` to `p1` (screen px).
    pub fn stroke(&mut self, p0: (Scalar, Scalar), p1: (Scalar, Scalar), width: Scalar, c: Rgb) {
        let half = width * 0.5;
        let r = half + 1.0;
        let minx = (p0.0.min(p1.0) - r).floor().max(0.0) as usize;
        let maxx = ((p0.0.max(p1.0) + r).ceil() as i64).clamp(0, self.w as i64) as usize;
        let miny = (p0.1.min(p1.1) - r).floor().max(0.0) as usize;
        let maxy = ((p0.1.max(p1.1) + r).ceil() as i64).clamp(0, self.h as i64) as usize;
        let dx = p1.0 - p0.0;
        let dy = p1.1 - p0.1;
        let len_sq = dx * dx + dy * dy;
        for y in miny..maxy {
            for x in minx..maxx {
                let qx = x as Scalar + 0.5;
                let qy = y as Scalar + 0.5;
                let t = if len_sq > 1e-12 {
                    (((qx - p0.0) * dx + (qy - p0.1) * dy) / len_sq).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let cx = p0.0 + t * dx;
                let cy = p0.1 + t * dy;
                let d = ((qx - cx).powi(2) + (qy - cy).powi(2)).sqrt();
                let cov = (half + 0.5 - d).clamp(0.0, 1.0);
                if cov > 0.0 {
                    self.blend(x, y, c, cov);
                }
            }
        }
    }

    /// Scanline fill of a simple polygon (even-odd rule).
    pub fn fill_polygon(&mut self, pts: &[(Scalar, Scalar)], c: Rgb) {
        if pts.len() < 3 {
            return;
        }
        for y in 0..self.h {
            let yc = y as Scalar + 0.5;
            let mut xs: Vec<Scalar> = Vec::new();
            for i in 0..pts.len() {
                let a = pts[i];
                let b = pts[(i + 1) % pts.len()];
                let (lo, hi) = if a.1 < b.1 { (a, b) } else { (b, a) };
                if yc >= lo.1 && yc < hi.1 {
                    xs.push(lo.0 + (yc - lo.1) / (hi.1 - lo.1) * (hi.0 - lo.0));
                }
            }
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mut i = 0;
            while i + 1 < xs.len() {
                let x0 = xs[i].max(0.0).floor() as usize;
                let x1 = (xs[i + 1].ceil() as i64).clamp(0, self.w as i64) as usize;
                for x in x0..x1 {
                    self.blend(x, y, c, 1.0);
                }
                i += 2;
            }
        }
    }

    /// Extract a sub-rectangle as a new canvas (clamped to bounds).
    pub fn crop(&self, x0: usize, y0: usize, w: usize, h: usize) -> Canvas {
        let w = w.min(self.w.saturating_sub(x0));
        let h = h.min(self.h.saturating_sub(y0));
        let mut out = Canvas::new(w, h, [255, 255, 255]);
        for y in 0..h {
            for x in 0..w {
                let si = 3 * ((y + y0) * self.w + (x + x0));
                let di = 3 * (y * w + x);
                out.px[di] = self.px[si];
                out.px[di + 1] = self.px[si + 1];
                out.px[di + 2] = self.px[si + 2];
            }
        }
        out
    }

    pub fn write_png(&self, path: &str) -> std::io::Result<()> {
        // Raw image: each scanline prefixed by filter byte 0 (none).
        let mut raw = Vec::with_capacity(self.h * (1 + self.w * 3));
        for y in 0..self.h {
            raw.push(0);
            let off = y * self.w * 3;
            raw.extend_from_slice(&self.px[off..off + self.w * 3]);
        }

        let mut png: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&(self.w as u32).to_be_bytes());
        ihdr.extend_from_slice(&(self.h as u32).to_be_bytes());
        ihdr.extend_from_slice(&[8, 2, 0, 0, 0]); // 8-bit, RGB
        chunk(&mut png, b"IHDR", &ihdr);
        chunk(&mut png, b"IDAT", &zlib_store(&raw));
        chunk(&mut png, b"IEND", &[]);
        std::fs::write(path, png)
    }
}

fn chunk(out: &mut Vec<u8>, typ: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(typ);
    out.extend_from_slice(data);
    let mut crc_buf = Vec::with_capacity(4 + data.len());
    crc_buf.extend_from_slice(typ);
    crc_buf.extend_from_slice(data);
    out.extend_from_slice(&crc32(&crc_buf).to_be_bytes());
}

fn crc32(buf: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in buf {
        crc ^= b as u32;
        for _ in 0..8 {
            let m = if crc & 1 != 0 { 0xEDB8_8320 } else { 0 };
            crc = (crc >> 1) ^ m;
        }
    }
    !crc
}

fn adler32(buf: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &x in buf {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

/// zlib stream using only uncompressed (stored) DEFLATE blocks.
fn zlib_store(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    let mut i = 0;
    if data.is_empty() {
        out.extend_from_slice(&[0x01, 0x00, 0x00, 0xFF, 0xFF]);
    }
    while i < data.len() {
        let n = core::cmp::min(65535, data.len() - i);
        let last = i + n >= data.len();
        out.push(if last { 1 } else { 0 });
        out.extend_from_slice(&(n as u16).to_le_bytes());
        out.extend_from_slice(&(!(n as u16)).to_le_bytes());
        out.extend_from_slice(&data[i..i + n]);
        i += n;
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

/// Render a leaf from its lamina polygons + vein graph (shape-agnostic; mirrors
/// `svg::render`). The petiole runs from (0,0) down by `petiole_len`.
pub fn render(scene: &Scene, opts: &RenderOpts) -> Canvas {
    let (minx, miny, maxx, maxy) = crate::svg::scene_bounds(scene);
    let world_h = (maxy - miny).max(1e-6);
    let world_w = (maxx - minx).max(1e-6);
    let scale = opts.target_height_px / world_h;
    let pad = opts.pad_px;
    let w = (world_w * scale + 2.0 * pad).ceil() as usize;
    let h = (world_h * scale + 2.0 * pad).ceil() as usize;

    let tx = |p: Vec2| -> (Scalar, Scalar) {
        ((p.x - minx) * scale + pad, (maxy - p.y) * scale + pad)
    };

    let mut cv = Canvas::new(w, h, [251, 253, 246]);

    // Petiole/stem first (behind everything).
    cv.stroke(tx(Vec2::new(0.0, 0.0)), tx(Vec2::new(0.0, -scene.petiole_len)), opts.max_vein_px, scene.petiole_color);

    for lam in &scene.laminae {
        let outline: Vec<(Scalar, Scalar)> = lam.points.iter().map(|p| tx(*p)).collect();
        cv.fill_polygon(&outline, lam.fill);
        for i in 0..outline.len() {
            cv.stroke(outline[i], outline[(i + 1) % outline.len()], 2.0, lam.stroke);
        }
    }

    // Finest veins first so majors render on top.
    let veins = &scene.veins;
    let mut order: Vec<usize> = (0..veins.edges.len()).collect();
    order.sort_by(|&i, &j| veins.edge_order[j].cmp(&veins.edge_order[i]));
    for &e in &order {
        let (a, b) = veins.edges[e];
        let ord = veins.edge_order[e];
        cv.stroke(tx(veins.nodes[a]), tx(veins.nodes[b]), vein_width(ord, opts), vein_color(scene.vein_base, ord));
    }
    cv
}
