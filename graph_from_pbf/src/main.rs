mod angles;
mod edges;
mod graph;
mod traversal_times;

use graph_from_pbf::{read_settings, write_json_file, Edge, Settings};
use anyhow::Result;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        panic!("Call with the input path to an osm.pbf, GeoTIFF and output directory");
    }
    for mode in ["walk", "cycling"].iter() {
        run(&args[1], &args[2], &args[3], mode).unwrap();
    }
}

fn run(osm_path: &str, tif_path: &str, output_directory: &str, mode: &str) -> Result<()> {
    let settings = read_settings(mode)?;

    let (graph_nodes_lookup, edges) = edges::process(osm_path, &settings)?;
    let traversal_times = traversal_times::process(&edges, tif_path, &settings);
    let angles = angles::process(&edges);
    let (graph, nodes) = graph::process(graph_nodes_lookup, traversal_times, angles, edges);

    write_json_file(format!("{mode}_nodes"), output_directory, &nodes)?;
    write_json_file(format!("{mode}_graph"), output_directory, &graph)?;

    Ok(())
}
