use anyhow::Result;
use fs_err::File;
use geo::LineString;
use serde::Deserialize;
use std::io::BufReader;
use polars::prelude::*;

pub struct Edge {
    pub id: usize,
    pub osm_id: i64,
    pub start_node: i64,
    pub end_node: i64,
    pub linestring: LineString,
    pub forward: bool,
    pub backward: bool,
}

pub struct SubNode {
    pub start_node: usize,    // start_node
    pub end_node: usize,      // end_node
    pub easting: f64,       // longitude
    pub northing: f64,        // latitude
    pub time_to_start: usize, // time_to_start
    pub time_to_end: usize,   // time_to_end
}

#[derive(Deserialize)]
pub struct Settings {
    pub mode: String,
    pub tag_pairs: Vec<(String, String)>,
    pub speed: f32,           // m/s
    pub ascention_speed: f32, // s/m
    pub descent_speed: f32,   // s/m
}


pub fn read_settings(mode: &str) -> Result<Settings> {
    let inpath = format!("settings/{}.json", mode);
    let file = File::open(inpath)?;
    let reader = BufReader::new(file);
    let settings: Settings = serde_json::from_reader(reader)?;
    Ok(settings)
}

pub fn write_subnodes_parquet(
    network_subnodes: &Vec<SubNode>,
    output_directory: &str,
    mode: &str,
) -> Result<()> {
    let start_nodes: Vec<u32> = network_subnodes
        .iter()
        .map(|x| x.start_node as u32)
        .collect();
    let end_nodes: Vec<u32> = network_subnodes.iter().map(|x| x.end_node as u32).collect();
    let eastings: Vec<f32> = network_subnodes
        .iter()
        .map(|x| x.easting as f32)
        .collect();
    let northings: Vec<f32> = network_subnodes.iter().map(|x| x.northing as f32).collect();
    let time_to_starts: Vec<u32> = network_subnodes
        .iter()
        .map(|x| x.time_to_start as u32)
        .collect();
    let time_to_ends: Vec<u32> = network_subnodes
        .iter()
        .map(|x| x.time_to_end as u32)
        .collect();

    let df: PolarsResult<DataFrame> = df!(
        "start_node" => start_nodes,
        "end_node" => end_nodes,
        "easting" => eastings,
        "northing" => northings,
        "time_to_start" => time_to_starts,
        "time_to_end" => time_to_ends,
    );
    let mut df = df.unwrap();
    let file = File::create(format!("{output_directory}/{mode}_subnodes.parquet"))
        .expect("could not create file");
    ParquetWriter::new(file)
        .finish(&mut df)
        .expect("could not save file");
    Ok(())
}
