//! Vein graph + minor (tertiary) venation via space colonization.
//!
//! The vein system is a graph (nodes + undirected edges), not a tree, because
//! brochidodromous loops and (later) reticulate areoles are cycles. Each node
//! and edge carries a *vein order* (0 = midrib, 1 = secondary/loop,
//! 2+ = successively finer minor veins) which drives stroke width and colour —
//! vein order is the botanical notion of vein hierarchy.

use crate::vec2::Vec2;
use crate::Scalar;

#[derive(Clone, Debug, Default)]
pub struct VeinGraph {
    pub nodes: Vec<Vec2>,
    pub edges: Vec<(usize, usize)>,
    pub edge_order: Vec<u8>,
    pub node_order: Vec<u8>,
}

impl VeinGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, p: Vec2, order: u8) -> usize {
        self.nodes.push(p);
        self.node_order.push(order);
        self.nodes.len() - 1
    }

    pub fn add_edge(&mut self, a: usize, b: usize, order: u8) {
        self.edges.push((a, b));
        self.edge_order.push(order);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MinorParams {
    /// di — a source only influences nodes within this distance.
    pub influence_radius: Scalar,
    /// dk — a source is consumed when a node comes this close.
    pub kill_radius: Scalar,
    /// D — length of each vein step.
    pub step: Scalar,
    /// Sources starting within this distance of the seed (major) veins are
    /// discarded, so minor veins grow as chains into the gaps rather than as
    /// stubs right next to the majors. Should exceed `kill_radius`.
    pub seed_clearance: Scalar,
    pub max_iters: usize,
    /// Finest vein order minor veins are allowed to reach.
    pub max_order: u8,
}

impl Default for MinorParams {
    fn default() -> Self {
        MinorParams {
            influence_radius: 1.2,
            kill_radius: 0.16,
            step: 0.12,
            seed_clearance: 0.30,
            max_iters: 8000,
            max_order: 4,
        }
    }
}

/// Extend `graph` (seeded with the major veins) with minor venation grown by
/// space colonization toward `sources`. New veins attach to the nearest
/// existing vein, so tertiaries sprout off the major veins and fill the blade.
/// Returns (iterations used, sources left unconsumed).
pub fn grow_minor(graph: &mut VeinGraph, mut sources: Vec<Vec2>, params: &MinorParams) -> (usize, usize) {
    let di_sq = params.influence_radius * params.influence_radius;
    let dk_sq = params.kill_radius * params.kill_radius;
    let clear_sq = params.seed_clearance * params.seed_clearance;

    // Drop sources that already sit near the seed (major) veins, so minor veins
    // grow into the gaps (the future areole interiors) rather than as stubs on
    // top of the majors.
    sources.retain(|s| !graph.nodes.iter().any(|n| n.dist_sq(*s) <= clear_sq));

    let mut iters = 0usize;
    for _ in 0..params.max_iters {
        if sources.is_empty() {
            break;
        }
        iters += 1;

        let mut dir_sum = vec![Vec2::zero(); graph.nodes.len()];
        let mut counts = vec![0u32; graph.nodes.len()];
        for s in &sources {
            let mut best = usize::MAX;
            let mut best_d = Scalar::INFINITY;
            for (i, n) in graph.nodes.iter().enumerate() {
                let d = n.dist_sq(*s);
                if d < best_d {
                    best_d = d;
                    best = i;
                }
            }
            if best != usize::MAX && best_d <= di_sq {
                dir_sum[best] = dir_sum[best].add(s.sub(graph.nodes[best]).normalized());
                counts[best] += 1;
            }
        }

        // Gather growth first (don't mutate the graph while reading it).
        let mut to_add: Vec<(usize, Vec2, u8)> = Vec::new();
        let existing = graph.nodes.len();
        for i in 0..existing {
            if counts[i] == 0 {
                continue;
            }
            let n_hat = dir_sum[i].normalized();
            if n_hat.len_sq() <= 1e-18 {
                continue;
            }
            let order = (graph.node_order[i] + 1).clamp(2, params.max_order);
            to_add.push((i, graph.nodes[i].add(n_hat.scale(params.step)), order));
        }
        if to_add.is_empty() {
            break;
        }
        for (parent, pos, order) in to_add {
            let idx = graph.add_node(pos, order);
            graph.add_edge(parent, idx, order);
        }

        sources.retain(|s| !graph.nodes.iter().any(|n| n.dist_sq(*s) <= dk_sq));
    }

    (iters, sources.len())
}
