mod edges;
mod graph;

use driving_graph::{read_driving_ways, read_node_coord_lookup, write_json_file};

fn main() {
    println!("Creating driving graph from OS");

    let node_coord_lookup = read_node_coord_lookup().unwrap();
    println!("Number of nodes: {}", node_coord_lookup.len());

    let driving_ways = read_driving_ways().unwrap();
    println!("Number of driving ways: {}", driving_ways.len());

    let (driving_edges, osid_to_graph_id) = edges::split_ways_into_edges(&node_coord_lookup, &driving_ways);
    println!("Number of driving edges: {}", driving_edges.len());

    for time_group in vec!["9to12", "12to14", "14to16", "16to19"] {
            let driving_graph = graph::create_driving_graph(&driving_edges, &osid_to_graph_id, time_group);
        write_json_file(format!("driving_graph_{}", time_group), "../data", driving_graph).unwrap();
    }
}