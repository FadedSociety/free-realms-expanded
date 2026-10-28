//! FabledRealms road graph: the client's own `FabledRealms.map`, for 98/1 path replies.
//!
//! The client ships its road graph and routes click-to-move over it; the breadcrumb and
//! Take Me There reply now follows the same roads instead of a straight line. The derived
//! data is `data/content/road_graph.json`:
//! - asset: the client's `FabledRealms.map`, sha256
//!   `edd298b4341a367a5d0e3e19d3e2de04f63911de4c12bc5dcacb1e75bae75565`, zone 0
//!   (FabledRealms; `scripts/build_terrain.py:49`);
//! - decoded at `lab/reports/stage7-route-geometry` (2,019 nodes, 2,151 undirected edges);
//! - shape: `{schema_version, source, zone, snap_radius_m, max_route_m,
//!   nodes: [[id, x, y, z, [neighbour ids]]]}`; node ids are `0..n` in array order.
//!
//! At load we recompute every edge length from the positions and refuse a non-finite
//! position or length, a missing neighbour, or a one-way edge. The two tuned values must
//! stay inside their bounds. (R) The server's route is a reconstruction: each end snaps to
//! the nearest node by planar x/z distance within `snap_radius_m`, A* runs over the 3D
//! edge lengths, and the caller falls back to a straight line when either end has no node
//! in range, no path exists, or the whole route is longer than `max_route_m`.
use serde::Deserialize;
use std::{
    cmp::Ordering,
    collections::BinaryHeap,
    fs,
    io::{self, Read},
    path::Path,
};

pub const CONTENT_FILE: &str = "road_graph.json";
/// The derived file stays under 1 MiB; the shipped graph is about 98 KiB.
const MAX_FILE: u64 = 1 << 20;
/// (R) Bounds on the two tuned values in the data.
const SNAP_RADIUS_M: std::ops::RangeInclusive<f32> = 5.0..=200.0;
const MAX_ROUTE_M: std::ops::RangeInclusive<f32> = 50.0..=5000.0;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContentFile {
    schema_version: u32,
    /// Provenance for readers; never gameplay input.
    #[serde(default, rename = "source")]
    _source: Option<serde_json::Value>,
    zone: u32,
    snap_radius_m: f32,
    max_route_m: f32,
    /// `[id, x, y, z, [neighbour ids]]`; the id is the row's index.
    nodes: Vec<(u32, f32, f32, f32, Vec<u32>)>,
}

#[derive(Debug)]
struct Node {
    position: [f32; 3],
    /// Neighbour index and its 3D length, recomputed at load.
    edges: Vec<(u32, f32)>,
}

/// The zone's road graph. Content, not character state: loaded once per session and never
/// saved. The pets walkability work reuses [`RoadGraph::load`] rather than reading the map
/// a second way.
#[derive(Debug)]
pub struct RoadGraph {
    zone: u32,
    snap_radius_m: f32,
    max_route_m: f32,
    nodes: Vec<Node>,
}

/// Queue entry ordered by f-score; ties break on the lower node index so runs repeat
/// exactly across sessions and machines.
struct Queue {
    estimate: f32,
    node: u32,
}

impl PartialEq for Queue {
    fn eq(&self, other: &Self) -> bool {
        self.estimate == other.estimate && self.node == other.node
    }
}

impl Eq for Queue {}

impl PartialOrd for Queue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Queue {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .estimate
            .total_cmp(&self.estimate)
            .then_with(|| other.node.cmp(&self.node))
    }
}

fn invalid(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

/// Euclidean 3D length of a road edge, recomputed from its end positions.
fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Planar x/z distance: the route graph is walked on the ground, and the request's y may
/// be a camera height or an authored node height.
fn planar(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

impl RoadGraph {
    /// `road_graph.json` in `directory`, `Ok(None)` when the content set has no graph.
    /// A malformed or out-of-bounds file refuses the content load.
    pub fn load(directory: &Path) -> io::Result<Option<Self>> {
        let mut text = String::new();
        match fs::File::open(directory.join(CONTENT_FILE)) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            file => file?.take(MAX_FILE + 1).read_to_string(&mut text)?,
        };
        if text.len() as u64 > MAX_FILE {
            return Err(invalid("road graph content exceeds 1 MiB"));
        }
        let file: ContentFile = serde_json::from_str(&text).map_err(|e| invalid(e.to_string()))?;
        Self::from_content(file).map(Some)
    }

    fn from_content(file: ContentFile) -> io::Result<Self> {
        if file.schema_version != 1 {
            return Err(invalid("unsupported road graph content version"));
        }
        if !SNAP_RADIUS_M.contains(&file.snap_radius_m) {
            return Err(invalid("road graph snap radius is outside 5..=200 m"));
        }
        if !MAX_ROUTE_M.contains(&file.max_route_m) {
            return Err(invalid("road graph route limit is outside 50..=5000 m"));
        }
        let mut positions = Vec::with_capacity(file.nodes.len());
        let mut adjacency = Vec::with_capacity(file.nodes.len());
        for (index, (id, x, y, z, neighbours)) in file.nodes.into_iter().enumerate() {
            if id as usize != index {
                return Err(invalid("road graph node ids must be 0..n in array order"));
            }
            if ![x, y, z].iter().all(|v| v.is_finite()) {
                return Err(invalid(format!(
                    "road graph node {id} has a non-finite position"
                )));
            }
            positions.push([x, y, z]);
            adjacency.push(neighbours);
        }
        let mut nodes: Vec<Node> = positions
            .iter()
            .map(|position| Node {
                position: *position,
                edges: Vec::new(),
            })
            .collect();
        for (index, neighbours) in adjacency.iter().enumerate() {
            for &neighbour in neighbours {
                let Some(target) = positions.get(neighbour as usize) else {
                    return Err(invalid(format!(
                        "road graph node {index} names missing neighbour {neighbour}"
                    )));
                };
                let length = distance(positions[index], *target);
                if !length.is_finite() {
                    return Err(invalid(format!(
                        "road graph edge {index}-{neighbour} has a non-finite length"
                    )));
                }
                nodes[index].edges.push((neighbour, length));
            }
        }
        for (index, node) in nodes.iter().enumerate() {
            for &(neighbour, _) in &node.edges {
                if !nodes[neighbour as usize]
                    .edges
                    .iter()
                    .any(|&(back, _)| back as usize == index)
                {
                    return Err(invalid(format!(
                        "road graph edge {index}-{neighbour} is one-way"
                    )));
                }
            }
        }
        Ok(Self {
            zone: file.zone,
            snap_radius_m: file.snap_radius_m,
            max_route_m: file.max_route_m,
            nodes,
        })
    }

    pub fn zone(&self) -> u32 {
        self.zone
    }

    /// The node positions from the node nearest `from` to the node nearest `to`, both
    /// within `snap_radius_m`, or `None` for no node in range, no path, or a route longer
    /// than `max_route_m` (from and to legs included).
    pub fn route(&self, from: [f32; 3], to: [f32; 3]) -> Option<Vec<[f32; 3]>> {
        let (start, start_leg) = self.nearest(from)?;
        let (goal, goal_leg) = self.nearest(to)?;
        let (path, length) = self.astar(start, goal)?;
        if start_leg + length + goal_leg > self.max_route_m {
            return None;
        }
        Some(
            path.into_iter()
                .map(|node| self.nodes[node as usize].position)
                .collect(),
        )
    }

    /// The nearest node by planar x/z distance, if it is within the snap radius.
    /// Lowest index wins a tie, so the result is deterministic.
    fn nearest(&self, point: [f32; 3]) -> Option<(u32, f32)> {
        let mut best: Option<(u32, f32)> = None;
        for (index, node) in self.nodes.iter().enumerate() {
            let length = planar(node.position, point);
            if best.is_none_or(|(_, shortest)| length < shortest) {
                best = Some((index as u32, length));
            }
        }
        best.filter(|(_, length)| *length <= self.snap_radius_m)
    }

    fn heuristic(&self, node: u32, goal: u32) -> f32 {
        distance(
            self.nodes[node as usize].position,
            self.nodes[goal as usize].position,
        )
    }

    /// A* over the recomputed edge lengths; `(nodes from start to goal, length)`.
    fn astar(&self, start: u32, goal: u32) -> Option<(Vec<u32>, f32)> {
        let mut cost = vec![f32::INFINITY; self.nodes.len()];
        let mut came_from = vec![u32::MAX; self.nodes.len()];
        let mut closed = vec![false; self.nodes.len()];
        let mut queue = BinaryHeap::new();
        cost[start as usize] = 0.0;
        queue.push(Queue {
            estimate: self.heuristic(start, goal),
            node: start,
        });
        while let Some(entry) = queue.pop() {
            let node = entry.node;
            if node == goal {
                break;
            }
            if std::mem::replace(&mut closed[node as usize], true) {
                continue;
            }
            for &(next, length) in &self.nodes[node as usize].edges {
                let candidate = cost[node as usize] + length;
                if candidate < cost[next as usize] {
                    cost[next as usize] = candidate;
                    came_from[next as usize] = node;
                    queue.push(Queue {
                        estimate: candidate + self.heuristic(next, goal),
                        node: next,
                    });
                }
            }
        }
        if !cost[goal as usize].is_finite() {
            return None;
        }
        let mut path = vec![goal];
        while *path.last()? != start {
            let previous = came_from[*path.last()? as usize];
            if previous == u32::MAX {
                return None;
            }
            path.push(previous);
        }
        path.reverse();
        Some((path, cost[goal as usize]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content(nodes: Vec<(u32, f32, f32, f32, Vec<u32>)>) -> ContentFile {
        ContentFile {
            schema_version: 1,
            _source: None,
            zone: 0,
            snap_radius_m: 40.0,
            max_route_m: 600.0,
            nodes,
        }
    }

    #[test]
    fn non_finite_positions_and_lengths_are_refused() {
        // JSON cannot carry NaN or infinity, so this rule is tested on the parsed form.
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(
                RoadGraph::from_content(content(vec![
                    (0, bad, 0.0, 0.0, vec![1]),
                    (1, 0.0, 0.0, 0.0, vec![0]),
                ]))
                .is_err(),
                "{bad}"
            );
        }
        // Finite positions whose recomputed edge length overflows.
        assert!(
            RoadGraph::from_content(content(vec![
                (0, 1e38, 0.0, 0.0, vec![1]),
                (1, -1e38, 0.0, 0.0, vec![0]),
            ]))
            .is_err()
        );
    }
}
