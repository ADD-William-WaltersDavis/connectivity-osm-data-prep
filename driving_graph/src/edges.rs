use driving_graph::{DrivingWay, OSNodeID, Speeds};
use std::collections::HashMap;
use geo::{Coord, LineString};
use indicatif::{ProgressBar, ProgressStyle};


pub struct DrivingEdge {
    pub start_node: OSNodeID,
    pub end_node: OSNodeID,
    pub linestring: LineString,
    pub forward: bool,
    pub backward: bool,
    pub speeds: Speeds,
}

pub fn split_ways_into_edges(
    node_mapping: &HashMap<OSNodeID, Coord>,
    ways: &HashMap<usize, DrivingWay>,
) -> (Vec<DrivingEdge>, HashMap<OSNodeID, usize>) {
    println!("Splitting ways into edges");
    // Count how many ways reference each node
    let mut node_counter: HashMap<OSNodeID, usize> = HashMap::new();
    // loop over ways hashmap
    for (_, driving_way) in ways {
        for node in &driving_way.node_ids {
            *node_counter.entry(*node).or_insert(0) += 1;
        }
    }
    // Split each way into edges
    let progress = ProgressBar::new(ways.len() as u64).with_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap());
    let mut edges: Vec<DrivingEdge> = Vec::new();
    // Create mapping of OSNodeID to graph node ID
    let mut osid_to_graph_id: HashMap<OSNodeID, usize> = HashMap::new();
    let mut graph_id: usize = 0;
    for (_, driving_way) in ways {
        progress.inc(1);
        let mut pts = Vec::new();
        let mut start_node = driving_way.node_ids[0].clone();
        let (forward, backward) = determine_directionality(driving_way);
        let node_ids = driving_way.node_ids.clone();
        let num_nodes = node_ids.len();
        for (idx, node) in node_ids.into_iter().enumerate() {
            pts.push(node_mapping[&node]);
            // Edges start/end at intersections between two ways. The endpoints of the way also
            // count as intersections.
            let is_endpoint =
                idx == 0 || idx == num_nodes - 1 || *node_counter.get(&node).unwrap() > 1;
            if is_endpoint && pts.len() > 1 {
                edges.push(DrivingEdge {
                    start_node: start_node,
                    end_node: node,
                    linestring: LineString::new(std::mem::take(&mut pts)),
                    forward: forward,
                    backward: backward,
                    speeds: driving_way.speeds,
                });
                start_node = node;
                // Start the next edge
                pts.push(node_mapping[&node]);
            }
            // if first time node is an endpoint, add to graph_id
            if is_endpoint && osid_to_graph_id.get(&node).is_none() {
                osid_to_graph_id.insert(node, graph_id);
                graph_id += 1;
            }
        }
    }
    progress.finish();
    println!("New edges found {}", edges.len()-ways.len());
    (edges, osid_to_graph_id)
}

fn determine_directionality(driving_way: &DrivingWay) -> (bool, bool) {
    let forward = driving_way.directionality == "Both Directions" || driving_way.directionality == "In Direction";
    let backward = driving_way.directionality == "Both Directions" || driving_way.directionality == "In Opposite Direction";
    (forward, backward)
}
