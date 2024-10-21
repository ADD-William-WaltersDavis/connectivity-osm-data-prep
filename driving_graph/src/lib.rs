use fs_err::File;
use std::io::{BufReader, BufWriter, Write};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use geo::Coord;
use std::collections::HashMap;

pub fn read_node_coord_lookup() -> Result<HashMap<OSNodeID, Coord>> {
    let file = File::open("../input/driving_index_coord_lookup.json")?;
    let reader = BufReader::new(file);
    let node_coord_lookup: HashMap<OSNodeID, Coord> = serde_json::from_reader(reader)?;
    Ok(node_coord_lookup)
}

pub fn read_driving_ways() -> Result<HashMap<usize, DrivingWay>> {
    let file = File::open("../input/driving_ways.json")?;
    let reader = BufReader::new(file);
    let driving_ways: HashMap<usize, DrivingWay> = serde_json::from_reader(reader)?;
    Ok(driving_ways)
}

#[derive(Deserialize, Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize)]
pub struct OSNodeID(pub i64);

#[derive(Deserialize, Debug, Clone)]
pub struct DrivingWay {
    pub geometry_length: f32,
    pub directionality: String,
    pub speeds: Speeds,
    pub node_ids: Vec<OSNodeID>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Speeds {
    pub in9to12: f64,
    pub in12to14: f64,
    pub in14to16: f64,
    pub in16to19: f64,
    pub against9to12: f64,
    pub against12to14: f64,
    pub against14to16: f64,
    pub against16to19: f64,
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
