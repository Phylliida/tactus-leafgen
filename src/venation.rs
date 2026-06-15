//! Venation growth via the space-colonization algorithm
//! (Runions, Lane & Prusinkiewicz 2007; the open-venation case of
//! Runions et al. 2005 "Modeling and Visualization of Leaf Venation Patterns").
//!
//! Open venation = a *tree*: veins grow from a root (the petiole) toward
//! scattered auxin sources, consuming them as they arrive. This is the clean
//! first case. Closed/reticulate venation (loops + areoles, a planar
//! subdivision) is a later mode layered on the same machinery.
//!
//! Algorithm per iteration:
//!   1. Each source picks the nearest vein node within `influence_radius`.
//!   2. Each node with ≥1 source attached grows one new node a step of length
//!      `step` toward the average (normalized) direction of its sources.
//!   3. Sources within `kill_radius` of any node are removed (consumed).
//!   4. Stop when sources run out, nothing grew, or `max_iters` is hit.

use crate::blade::Blade;
use crate::vec2::Vec2;
use crate::Scalar;

/// A node in the vein graph. `parent == None` marks the root (petiole).
/// Parent indices are always smaller than the child's index (children are
/// pushed after their parent), giving a ready-made topological order.
#[derive(Clone, Copy, Debug)]
pub struct VeinNode {
    pub pos: Vec2,
    pub parent: Option<usize>,
}

#[derive(Clone, Copy, Debug)]
pub struct VenationParams {
    /// di — a source only influences nodes within this distance.
    pub influence_radius: Scalar,
    /// dk — a source is consumed when a node comes this close.
    pub kill_radius: Scalar,
    /// D — length of each vein step.
    pub step: Scalar,
    pub max_iters: usize,
}

impl Default for VenationParams {
    fn default() -> Self {
        VenationParams {
            influence_radius: 1.6,
            kill_radius: 0.32,
            step: 0.16,
            max_iters: 8000,
        }
    }
}

pub struct Venation {
    pub nodes: Vec<VeinNode>,
    /// Sources left unconsumed when growth stopped (typically near the tips).
    pub leftover_sources: usize,
    pub iters_used: usize,
}

/// Grow open (tree) venation rooted at the blade's petiole (0,0).
pub fn grow_open(_blade: &Blade, sources: Vec<Vec2>, params: &VenationParams) -> Venation {
    let mut nodes: Vec<VeinNode> = Vec::new();
    nodes.push(VeinNode {
        pos: Vec2::new(0.0, 0.0),
        parent: None,
    });

    let mut sources = sources;
    let di_sq = params.influence_radius * params.influence_radius;
    let dk_sq = params.kill_radius * params.kill_radius;
    let mut iters_used = 0usize;

    for _ in 0..params.max_iters {
        if sources.is_empty() {
            break;
        }
        iters_used += 1;

        // (1) Attach each source to its nearest node within the influence
        // radius; accumulate per-node growth directions.
        let mut dir_sum = vec![Vec2::zero(); nodes.len()];
        let mut counts = vec![0u32; nodes.len()];
        for s in &sources {
            let mut best = usize::MAX;
            let mut best_d = Scalar::INFINITY;
            for (i, node) in nodes.iter().enumerate() {
                let d = node.pos.dist_sq(*s);
                if d < best_d {
                    best_d = d;
                    best = i;
                }
            }
            if best != usize::MAX && best_d <= di_sq {
                dir_sum[best] = dir_sum[best].add(s.sub(nodes[best].pos).normalized());
                counts[best] += 1;
            }
        }

        // (2) Grow a new node from each influenced node.
        let mut grew = false;
        let existing = nodes.len();
        for i in 0..existing {
            if counts[i] == 0 {
                continue;
            }
            let n_hat = dir_sum[i].normalized();
            if n_hat.len_sq() <= 1e-18 {
                continue; // opposing sources cancelled out
            }
            let base = nodes[i].pos;
            nodes.push(VeinNode {
                pos: base.add(n_hat.scale(params.step)),
                parent: Some(i),
            });
            grew = true;
        }
        if !grew {
            break; // every remaining source is out of reach
        }

        // (3) Consume sources reached by any node.
        sources.retain(|s| !nodes.iter().any(|n| n.pos.dist_sq(*s) <= dk_sq));
    }

    Venation {
        leftover_sources: sources.len(),
        iters_used,
        nodes,
    }
}

/// Number of nodes in each node's subtree (itself included). Because every
/// parent index is smaller than its child's, a single reverse pass suffices.
/// Used to taper vein stroke width (more descendants → thicker → major vein).
pub fn subtree_sizes(nodes: &[VeinNode]) -> Vec<usize> {
    let mut sizes = vec![1usize; nodes.len()];
    for i in (0..nodes.len()).rev() {
        if let Some(p) = nodes[i].parent {
            sizes[p] += sizes[i];
        }
    }
    sizes
}
