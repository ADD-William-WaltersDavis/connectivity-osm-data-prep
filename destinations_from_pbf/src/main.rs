mod destinations;
mod values;

use anyhow::Result;
use destinations_from_pbf::write_json_file;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        panic!("Call with the input path to an osm.pbf and output directory");
    }
    for mode in ["walk", "cycling"].iter() {
        run(&args[1], &args[2], mode).unwrap();
    }
}

fn run(osm_path: &str, output_directory: &str, mode: &str) -> Result<()> {
    let destinations = destinations::process(osm_path)?;
    let graph_values = values::process(&destinations, mode)?;
    write_json_file("destinations".to_string(), output_directory, destinations)?;
    write_json_file(
        format!("{}_graph_values", mode),
        output_directory,
        graph_values,
    )?;
    Ok(())
}
