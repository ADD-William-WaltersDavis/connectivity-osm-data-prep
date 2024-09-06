mod read;

use anyhow::Result;
use serde::Serialize;
use fs_err::File;
use std::io::{BufWriter, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        panic!("Call with the input path to an osm.pbf and output directory");
    }
    run(&args[1], &args[2]).unwrap();
}

fn run(osm_path: &str, output_directory: &str) -> Result<()> {

    let destinations = read::process(osm_path)?;
    write_json_file("destinations".to_string(), output_directory, destinations)?;
    Ok(())
}


pub fn write_json_file<T: Serialize>(
    file_name: String,
    output_directory: &str,
    data: T,
) -> Result<()> {
    let path = format!("{output_directory}/{file_name}.json");
    println!("Writing to {path}");
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &data)?;
    writer.flush()?;
    Ok(())
}
