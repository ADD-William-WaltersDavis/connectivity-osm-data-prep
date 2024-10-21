mod edges;
mod graph;

use driving_graph::{read_driving_ways, read_node_coord_lookup, write_json_file, OSNodeID};
use std::collections::HashMap;
use geo::Coord;

fn main() {
    println!("Creating driving graph from OS");

    let node_coord_lookup = read_node_coord_lookup().unwrap();
    println!("Number of nodes: {}", node_coord_lookup.len());

    let driving_ways = read_driving_ways().unwrap();
    println!("Number of driving ways: {}", driving_ways.len());

    let (driving_edges, osid_to_graph_id) = edges::split_ways_into_edges(&node_coord_lookup, &driving_ways);
    println!("Number of driving edges: {}", driving_edges.len());

    // for time_group in vec!["9to12", "12to14", "14to16", "16to19"] {
    //         let driving_graph = graph::create_driving_graph(&driving_edges, &osid_to_graph_id, time_group);
    //     write_json_file(format!("driving_graph_{}", time_group), "../data", driving_graph).unwrap();
    // }
    let graphnode_coord_lookup = create_graphnode_coord_lookup(&node_coord_lookup, &osid_to_graph_id);
    write_json_file("graphnode_coord_lookup".to_string(), "../data", graphnode_coord_lookup).unwrap();
}

fn create_graphnode_coord_lookup(
    node_coord_lookup: &HashMap<OSNodeID, Coord>,
    osid_to_graph_id: &HashMap<OSNodeID, usize>,
) -> HashMap<usize, Coord> {
    let mut graphnode_coord_lookup: HashMap<usize, Coord> = HashMap::new();
    for (osid, graph_id) in osid_to_graph_id {
        graphnode_coord_lookup.insert(*graph_id, node_coord_lookup[osid]);
    }
    graphnode_coord_lookup
}