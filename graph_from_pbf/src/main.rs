mod angles;
mod edges;
mod graph;
pub mod pt_stops;
pub mod public_transport_graphs;
mod traversal_times;

use anyhow::Result;
use graph_from_pbf::{read_settings, write_json_file, Edge, Settings, read_walk_graph, read_walk_nodes};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        panic!("Call with the input path to an osm.pbf, GeoTIFF, output directory and PT toggle");
    }
    let pt_toggle = args[4].parse::<bool>().unwrap();   
    for mode in ["walk", "cycling"].iter() {
        run(&args[1], &args[2], &args[3], pt_toggle, mode).unwrap();
    }
}

fn run(osm_path: &str, tif_path: &str, output_directory: &str, pt_toggle: bool, mode: &str) -> Result<()> {
    let settings = read_settings(mode)?;

    let (graph_nodes_lookup, edges) = edges::process(osm_path, &settings)?;
    let traversal_times = traversal_times::calculate(&edges, tif_path, &settings);
    let angles = angles::calculate(&edges);
    let (graph, nodes) = graph::process(graph_nodes_lookup, traversal_times, angles, edges);

    write_json_file(format!("{mode}_nodes"), output_directory, &nodes)?;
    write_json_file(format!("{mode}_graph"), output_directory, &graph)?;

    if mode == "walk" && pt_toggle {
        let (pt_graph_walk, pt_graph_routes, pt_graph_routes_reverse) = public_transport_graphs::process(graph, nodes)?;
        write_json_file(format!("pt_graph_walk"), output_directory, &pt_graph_walk)?;
        write_json_file(
            format!("pt_graph_routes"),
            output_directory,
            &pt_graph_routes,
        )?;
        write_json_file(
            format!("pt_graph_routes_reverse"),
            output_directory,
            &pt_graph_routes_reverse,
        )?;
    }
    Ok(())
}
