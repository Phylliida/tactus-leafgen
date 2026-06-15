//! Vein graph + minor (tertiary) venation via space colonization.
//!
//! The vein system is a graph (nodes + undirected edges), not a tree, because
//! brochidodromous loops and (later) reticulate areoles are cycles. Each node
//! and edge carries a *vein order* (0 = midrib, 1 = secondary/loop,
//! 2+ = successively finer minor veins) which drives stroke width and colour —
//! vein order is the botanical notion of vein hierarchy.

use crate::vec2::Vec2;
use crate::Scalar;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct VeinGraph {
    pub nodes: Vec<Vec2>,
    pub edges: Vec<(usize, usize)>,
    pub edge_order: Vec<u8>,
    pub node_order: Vec<u8>,
    /// Growth parent of each node (the node it sprouted from), or `None` for a
    /// root. Used by anastomosis to avoid closing trivial loops between a tip
    /// and its own ancestors.
    pub parents: Vec<Option<usize>>,
}

impl VeinGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, p: Vec2, order: u8) -> usize {
        self.nodes.push(p);
        self.node_order.push(order);
        self.parents.push(None);
        self.nodes.len() - 1
    }

    pub fn add_edge(&mut self, a: usize, b: usize, order: u8) {
        self.edges.push((a, b));
        self.edge_order.push(order);
    }

    /// Add a node grown from `parent`: creates the node, the connecting edge,
    /// and records the parent link.
    pub fn add_child(&mut self, parent: usize, p: Vec2, order: u8) -> usize {
        let i = self.add_node(p, order);
        self.add_edge(parent, i, order);
        self.parents[i] = Some(parent);
        i
    }

    /// Append another graph's nodes/edges, mapping each node position through
    /// `f` (used to place a leaflet's venation into a compound leaf). Returns
    /// the index offset (= the appended graph's node 0).
    pub fn append_transformed(&mut self, other: &VeinGraph, f: impl Fn(Vec2) -> Vec2) -> usize {
        let off = self.nodes.len();
        for (i, p) in other.nodes.iter().enumerate() {
            self.nodes.push(f(*p));
            self.node_order.push(other.node_order[i]);
            self.parents.push(other.parents[i].map(|x| x + off));
        }
        for (k, &(a, b)) in other.edges.iter().enumerate() {
            self.edges.push((a + off, b + off));
            self.edge_order.push(other.edge_order[k]);
        }
        off
    }

    /// Node degrees (incident edge counts).
    pub fn degrees(&self) -> Vec<u32> {
        let mut deg = vec![0u32; self.nodes.len()];
        for &(a, b) in &self.edges {
            deg[a] += 1;
            deg[b] += 1;
        }
        deg
    }

    /// Write `node` and up to `k` of its ancestors into `buf`; return the count.
    /// Allocation-free (caller supplies a small stack buffer).
    fn ancestry_into(&self, node: usize, k: usize, buf: &mut [usize]) -> usize {
        let mut len = 1;
        buf[0] = node;
        let mut x = node;
        for _ in 0..k {
            if len >= buf.len() {
                break;
            }
            match self.parents[x] {
                Some(p) => {
                    x = p;
                    buf[len] = x;
                    len += 1;
                }
                None => break,
            }
        }
        len
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
            influence_radius: 0.6,
            kill_radius: 0.10,
            step: 0.09,
            seed_clearance: 0.16,
            max_iters: 400,
            max_order: 4,
        }
    }
}

/// Extend `graph` (seeded with the major veins) with minor venation grown by
/// space colonization toward `sources`. New veins attach to the nearest
/// existing vein, so tertiaries sprout off the major veins and fill the blade.
/// Returns (iterations used, sources left unconsumed).
fn grid_key(p: Vec2, cell: Scalar) -> (i64, i64) {
    ((p.x / cell).floor() as i64, (p.y / cell).floor() as i64)
}

fn build_node_grid(nodes: &[Vec2], cell: Scalar) -> HashMap<(i64, i64), Vec<usize>> {
    let mut grid: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (i, n) in nodes.iter().enumerate() {
        grid.entry(grid_key(*n, cell)).or_default().push(i);
    }
    grid
}

/// Nearest node to `s` (searching the 3×3 cell neighbourhood), as
/// (index, squared distance). `cell` must be ≥ the query radius of interest.
fn nearest_in_grid(
    nodes: &[Vec2],
    grid: &HashMap<(i64, i64), Vec<usize>>,
    cell: Scalar,
    s: Vec2,
) -> (usize, Scalar) {
    let (kx, ky) = grid_key(s, cell);
    let mut best = usize::MAX;
    let mut best_d = Scalar::INFINITY;
    for dx in -1..=1 {
        for dy in -1..=1 {
            if let Some(bucket) = grid.get(&(kx + dx, ky + dy)) {
                for &i in bucket {
                    let d = nodes[i].dist_sq(s);
                    if d < best_d {
                        best_d = d;
                        best = i;
                    }
                }
            }
        }
    }
    (best, best_d)
}

pub fn grow_minor(graph: &mut VeinGraph, mut sources: Vec<Vec2>, params: &MinorParams) -> (usize, usize) {
    let di_sq = params.influence_radius * params.influence_radius;
    let dk_sq = params.kill_radius * params.kill_radius;
    let clear_sq = params.seed_clearance * params.seed_clearance;
    // Cell must cover the largest query radius so 3×3 search is exhaustive.
    let cell = params
        .influence_radius
        .max(params.seed_clearance)
        .max(params.kill_radius)
        .max(1e-6);

    // One spatial grid, grown incrementally as veins are added (rebuilding it
    // every iteration was the dominant cost).
    let mut grid = build_node_grid(&graph.nodes, cell);

    // Drop sources near the seed (major) veins, so minor veins grow into the
    // gaps (the future areole interiors) rather than as stubs on the majors.
    sources.retain(|s| {
        let (b, d) = nearest_in_grid(&graph.nodes, &grid, cell, *s);
        !(b != usize::MAX && d <= clear_sq)
    });

    let mut dir_sum: Vec<Vec2> = Vec::new();
    let mut counts: Vec<u32> = Vec::new();
    let mut iters = 0usize;
    for _ in 0..params.max_iters {
        if sources.is_empty() {
            break;
        }
        iters += 1;

        let n = graph.nodes.len();
        dir_sum.clear();
        dir_sum.resize(n, Vec2::zero());
        counts.clear();
        counts.resize(n, 0);
        for s in &sources {
            let (b, d) = nearest_in_grid(&graph.nodes, &grid, cell, *s);
            if b != usize::MAX && d <= di_sq {
                dir_sum[b] = dir_sum[b].add(s.sub(graph.nodes[b]).normalized());
                counts[b] += 1;
            }
        }

        // Grow one new node per influenced node, inserting it into the grid.
        let mut grew = false;
        for i in 0..n {
            if counts[i] == 0 {
                continue;
            }
            let n_hat = dir_sum[i].normalized();
            if n_hat.len_sq() <= 1e-18 {
                continue;
            }
            let order = (graph.node_order[i] + 1).clamp(2, params.max_order);
            let pos = graph.nodes[i].add(n_hat.scale(params.step));
            let idx = graph.add_child(i, pos, order);
            grid.entry(grid_key(pos, cell)).or_default().push(idx);
            grew = true;
        }
        if !grew {
            break;
        }

        sources.retain(|s| {
            let (b, d) = nearest_in_grid(&graph.nodes, &grid, cell, *s);
            !(b != usize::MAX && d <= dk_sq)
        });
    }

    (iters, sources.len())
}

#[derive(Clone, Copy, Debug)]
pub struct AnastomoseParams {
    /// Two unrelated tips closer than this are joined into a loop.
    pub radius: Scalar,
    /// How many ancestor levels count as "related" (skip to avoid trivial
    /// loops between a tip and its own branch).
    pub ancestor_depth: usize,
}

impl Default for AnastomoseParams {
    fn default() -> Self {
        AnastomoseParams {
            radius: 0.17,
            ancestor_depth: 4,
        }
    }
}

/// Close the open minor venation into a reticulate mesh. Each minor vein *tip*
/// (degree-1, order ≥ 2) is joined to the nearest existing vein node within
/// `radius` that is not on its own branch (no shared ancestor within
/// `ancestor_depth`) — a vein-to-vein junction. Each such edge closes a loop:
/// an areole. A node may receive several junctions, so the result is a
/// connected polygonal network; tips with no eligible neighbour remain as
/// free-ending veinlets inside the cells. Returns the number of anastomoses.
pub fn anastomose(graph: &mut VeinGraph, params: &AnastomoseParams) -> usize {
    let r_sq = params.radius * params.radius;
    let deg = graph.degrees();
    let tips: Vec<usize> = (0..graph.nodes.len())
        .filter(|&i| graph.node_order[i] >= 2 && deg[i] == 1)
        .collect();

    // Spatial hash over ALL nodes so a tip can find the nearest vein to land on.
    let cell = params.radius.max(1e-6);
    let grid = build_node_grid(&graph.nodes, cell);

    // A tip is consumed once it lands; target nodes are NOT consumed, so a vein
    // can be a junction for several incoming veinlets.
    let mut count = 0usize;
    let depth = params.ancestor_depth.min(7);
    let mut at = [0usize; 8];
    let mut au = [0usize; 8];
    for &t in &tips {
        let pt = graph.nodes[t];
        let (kx, ky) = grid_key(pt, cell);
        let atn = graph.ancestry_into(t, depth, &mut at);
        let mut best = usize::MAX;
        let mut best_d = r_sq;
        for dx in -1..=1 {
            for dy in -1..=1 {
                let Some(bucket) = grid.get(&(kx + dx, ky + dy)) else {
                    continue;
                };
                for &u in bucket {
                    if u == t || at[..atn].contains(&u) {
                        continue;
                    }
                    let d = graph.nodes[u].dist_sq(pt);
                    if d >= best_d {
                        continue;
                    }
                    // Reject if u is on t's branch (shared recent ancestor).
                    let aun = graph.ancestry_into(u, depth, &mut au);
                    if at[..atn].iter().any(|x| au[..aun].contains(x)) {
                        continue;
                    }
                    best_d = d;
                    best = u;
                }
            }
        }
        if best != usize::MAX {
            let order = graph.node_order[t].max(graph.node_order[best]);
            graph.add_edge(t, best, order);
            count += 1;
        }
    }
    count
}
