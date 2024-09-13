use crate::angles::Angles;
use geo::Coord;
use graph_from_pbf::Edge;

use std::collections::HashMap;

pub fn process(
    graph_nodes_lookup: HashMap<i64, (usize, Coord)>,
    traversal_times: HashMap<usize, (usize, usize)>,
    angles: HashMap<usize, Angles>,
    edges: Vec<Edge>,
) -> (
    Vec<Vec<(usize, usize, u16, u16, u32)>>,
    HashMap<usize, Coord>,
) {
    println!("Creating nodes");
    let nodes = convert_graph_nodes_lookup_to_nodes(&graph_nodes_lookup);

    println!("Creating graph");
    let mut all_links: Vec<(usize, usize, usize, u16, u16)> = Vec::new();
    for edge in edges {
        // forward direction
        if edge.forward {
            all_links.push((
                graph_nodes_lookup[&edge.start_node].0,
                graph_nodes_lookup[&edge.end_node].0,
                traversal_times[&edge.id].0,
                angles[&edge.id].forward_departure,
                angles[&edge.id].forward_arrival,
            ));
        }
        // backward direction
        if edge.backward {
            all_links.push((
                graph_nodes_lookup[&edge.end_node].0,
                graph_nodes_lookup[&edge.start_node].0,
                traversal_times[&edge.id].1,
                angles[&edge.id].backward_departure,
                angles[&edge.id].backward_arrival,
            ));
        }
    }
    all_links.sort();

    let graph = group_links_into_graph(all_links);

    (graph, nodes)
}

fn convert_graph_nodes_lookup_to_nodes(
    graph_nodes_lookup: &HashMap<i64, (usize, Coord)>,
) -> HashMap<usize, Coord> {
    let nodes: HashMap<usize, Coord> = graph_nodes_lookup
        .iter()
        .map(|(_, (id, coord))| (id.clone(), coord.clone()))
        .collect();
    nodes
}

fn group_links_into_graph(
    all_links: Vec<(usize, usize, usize, u16, u16)>,
) -> Vec<Vec<(usize, usize, u16, u16, u32)>> {
    let mut link_id: u32 = 0;
    let mut graph: Vec<Vec<(usize, usize, u16, u16, u32)>> = Vec::new();
    for (start_node, end_node, traversal_time, departure_angle, arrival_angle) in all_links {
        if graph.len() <= start_node {
            graph.resize(start_node + 1, Vec::new());
        }
        graph[start_node].push((
            traversal_time,
            end_node,
            departure_angle,
            arrival_angle,
            link_id,
        ));
        link_id += 1;
    }
    graph
}
