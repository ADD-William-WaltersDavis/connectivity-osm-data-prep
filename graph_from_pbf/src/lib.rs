use geo::LineString;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::io::{BufWriter, Write};
use fs_err::File;

#[derive(Serialize)]
pub struct Edge {
    pub id: usize,
    pub osm_id: i64,
    pub start_node: i64,
    pub end_node: i64,
    pub linestring: LineString,
}

#[derive(Serialize)]
pub struct Angles {
    pub forward_arrival: u16,
    pub forward_departure: u16,
    pub backward_arrival: u16,
    pub backward_departure: u16,
}

#[derive(Deserialize)]
pub struct Settings {
    pub mode: String,
    pub tag_pairs: Vec<(String, String)>,
    pub speed: f32, // m/s
    pub ascention_speed: f32, // s/m
    pub descent_speed: f32, // s/m
}

pub fn write_json_file<T: Serialize>(file_name: String, output_directory: &str, data: T) -> Result<()> {
    let path = format!("{output_directory}/{file_name}.json");
    println!("Writing to {path}");
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &data)?;
    writer.flush()?;
    Ok(())
}

pub fn read_settings(mode: &str) -> Result<Settings> {
    let inpath = format!("settings/{}.json", mode);
    let file = File::open(inpath)?;
    let reader = std::io::BufReader::new(file);
    let settings: Settings = serde_json::from_reader(reader)?;
    Ok(settings)
}