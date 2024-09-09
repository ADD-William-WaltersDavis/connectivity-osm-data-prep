use anyhow::Result;
use fs_err::File;
use geo::{Coord, Geometry, Point};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufWriter, Write};

#[derive(Serialize)]
pub struct Destination {
    pub id: usize,
    pub name: String,
    pub purpose: String,
    pub subpurpose: String,
    pub geometry: Geometry,
    pub centroid: Point,
}

// #[derive(Deserialize)]
// pub struct Nodes (
//     pub HashMap<usize, Coord>
// );

pub fn read_nodes(mode: &str) -> Result<HashMap<usize, Coord>> {
    let inpath = format!("../data/{}_nodes.json", mode);
    let file = File::open(inpath)?;
    let reader = std::io::BufReader::new(file);
    let nodes: HashMap<usize, Coord> = serde_json::from_reader(reader)?;
    Ok(nodes)
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
