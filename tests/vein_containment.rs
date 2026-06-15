//! Every vein of a single-lamina leaf must lie inside that lamina. This is the
//! runtime regression test for the containment bugs we hit by eye (veins arcing
//! past a narrow margin, veins floating in an apical notch) — and a preview of
//! the invariant the eventual tactus/Lean proof will assert statically.

use tactus_leafgen::gallery;
use tactus_leafgen::vec2::Vec2;

/// Crossing-number point-in-polygon test.
fn point_in_polygon(p: Vec2, poly: &[Vec2]) -> bool {
    let n = poly.len();
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (a, b) = (poly[i], poly[j]);
        if (a.y > p.y) != (b.y > p.y) {
            let xint = a.x + (p.y - a.y) / (b.y - a.y) * (b.x - a.x);
            if p.x < xint {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

fn dist_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f64 {
    let ab = b.sub(a);
    let len2 = ab.dot(ab);
    let t = if len2 > 1e-12 {
        (p.sub(a).dot(ab) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    p.sub(a.add(ab.scale(t))).len()
}

fn dist_to_polygon(p: Vec2, poly: &[Vec2]) -> f64 {
    let n = poly.len();
    let mut best = f64::INFINITY;
    for i in 0..n {
        let d = dist_to_segment(p, poly[i], poly[(i + 1) % n]);
        best = best.min(d);
    }
    best
}

fn bbox_extent(poly: &[Vec2]) -> f64 {
    let (mut mnx, mut mny, mut mxx, mut mxy) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    for p in poly {
        mnx = mnx.min(p.x);
        mny = mny.min(p.y);
        mxx = mxx.max(p.x);
        mxy = mxy.max(p.y);
    }
    (mxx - mnx).max(mxy - mny)
}

#[test]
fn all_veins_inside_their_lamina() {
    let leaves = gallery::all();
    assert!(leaves.len() >= 25, "expected the full gallery, got {}", leaves.len());

    let mut checked_single = 0;
    let mut total_nodes = 0;
    let mut failures: Vec<String> = Vec::new();

    for leaf in &leaves {
        if !leaf.single_lamina {
            continue; // compound rachis legitimately lies between leaflets
        }
        checked_single += 1;
        let poly = &leaf.laminae[0];
        // boundary/anchor nodes sit on the margin; allow a small slack.
        let eps = 0.015 * bbox_extent(poly);

        for (i, &node) in leaf.veins.nodes.iter().enumerate() {
            total_nodes += 1;
            let inside = point_in_polygon(node, poly) || dist_to_polygon(node, poly) <= eps;
            if !inside {
                let d = dist_to_polygon(node, poly);
                failures.push(format!(
                    "{}: node {} at ({:.2},{:.2}) outside lamina by {:.3} (eps {:.3})",
                    leaf.name, i, node.x, node.y, d, eps
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} vein(s) outside their lamina:\n{}",
        failures.len(),
        failures.iter().take(20).cloned().collect::<Vec<_>>().join("\n")
    );

    eprintln!(
        "vein containment OK: {} single-lamina templates, {} vein nodes checked",
        checked_single, total_nodes
    );
}
