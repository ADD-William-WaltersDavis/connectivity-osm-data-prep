mod edges;
mod subnodes;

use anyhow::Result;
use subnodes_from_pbf::{read_settings, write_json_file, Edge, Settings, SubNode};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 6 {
        panic!("Call with the input path to each E/W/S osm.pbf, GeoTIFF, output directory");
    }
    let osm_paths: Vec<&str> = vec![&args[1], &args[2], &args[3]];
    for mode in ["walk", "cycling"].iter() {
        run(osm_paths.clone(), &args[4], &args[5], mode).unwrap();
    }
}

fn run(osm_paths: Vec<&str>, tif_path: &str, output_directory: &str, mode: &str) -> Result<()> {
    let settings = read_settings(mode)?;

    let (graph_nodes_lookup, edges) = edges::process(osm_paths, &settings)?;
    let network_subnodes = subnodes::process(&edges, tif_path, &settings, graph_nodes_lookup);

    write_json_file(format!("{mode}_subnodes"), output_directory, &network_subnodes)?;

    Ok(())
}
