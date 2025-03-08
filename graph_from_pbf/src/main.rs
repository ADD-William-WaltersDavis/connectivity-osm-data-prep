mod angles;
mod edges;
mod graph;
pub mod pt_stops;
pub mod public_transport_graphs;
mod subnodes;
mod traversal_times;

use anyhow::Result;
use connectivity::io::write_json_file;
use graph_from_pbf::{read_settings, write_subnodes_parquet, Edge, Settings, SubNode};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 10 {
        panic!("Call with the input path to each E/W/S osm.pbf, GeoTIFF, output directory, PT toggle, PT stops path, and PT routes path");
    }
    let osm_paths: Vec<&str> = vec![&args[1], &args[2], &args[3]];
    let pt_toggle = args[6].parse::<bool>().unwrap();
    let subnodes_toggle = args[9].parse::<bool>().unwrap(); // this requires a larger machine
    let pt_paths: Vec<&str> = vec![&args[7], &args[8]];
    for mode in ["walk", "cycling"].iter() {
        run(
            osm_paths.clone(),
            &args[4],
            &args[5],
            pt_toggle,
            subnodes_toggle,
            mode,
            pt_paths.clone(),
        )
        .unwrap();
    }
}

fn run(
    osm_paths: Vec<&str>,
    tif_path: &str,
    output_directory: &str,
    pt_toggle: bool,
    subnodes_toggle: bool,
    mode: &str,
    pt_paths: Vec<&str>,
) -> Result<()> {
    let settings = read_settings(mode)?;

    let (graph_nodes_lookup, edges) = edges::process(osm_paths, &settings)?;
    let traversal_times = traversal_times::calculate(&edges, tif_path, &settings);
    if subnodes_toggle {
        let network_subnodes = subnodes::process(&edges, tif_path, &settings, &graph_nodes_lookup);
        write_subnodes_parquet(&network_subnodes, output_directory, mode)?;
    }
    let angles = angles::calculate(&edges);
    let (graph, nodes) = graph::process(graph_nodes_lookup, traversal_times, angles, edges);

    write_json_file(format!("{mode}_nodes"), output_directory, &nodes)?;
    write_json_file(format!("{mode}_graph"), output_directory, &graph)?;

    if mode == "walk" && pt_toggle {
        let (pt_graph_walk, pt_graph_routes, pt_graph_routes_reverse) =
            public_transport_graphs::process(graph, nodes, pt_paths)?;
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
