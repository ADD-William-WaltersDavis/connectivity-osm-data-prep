use anyhow::Result;
use fs_err::File;
use geo::{Coord, LineString};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Write};
use polars::prelude::*;

#[derive(Serialize)]
pub struct Edge {
    pub id: usize,
    pub osm_id: i64,
    pub start_node: i64,
    pub end_node: i64,
    pub linestring: LineString,
    pub forward: bool,
    pub backward: bool,
}

#[derive(Serialize)]
pub struct SubNode {
    pub start_node: usize,    // start_node
    pub end_node: usize,      // end_node
    pub longitude: f64,       // longitude
    pub latitude: f64,        // latitude
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

#[derive(Deserialize)]
pub struct InputTimetable {
    pub pt_stop_node: usize,
    pub next_node: Option<usize>,
    pub timetable: Option<Timetable>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Timetable(pub Vec<(usize, usize)>);

impl Timetable {
    pub fn reverse(&mut self) {
        self.0.reverse();
    }
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

pub fn read_settings(mode: &str) -> Result<Settings> {
    let inpath = format!("settings/{}.json", mode);
    let file = File::open(inpath)?;
    let reader = BufReader::new(file);
    let settings: Settings = serde_json::from_reader(reader)?;
    Ok(settings)
}

pub fn read_timetables(path: &str) -> Result<Vec<InputTimetable>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let timetables: Vec<InputTimetable> = serde_json::from_reader(reader)?;
    Ok(timetables)
}

pub fn read_pt_stops(path: &str) -> Result<Vec<(usize, Coord)>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let pt_stops: Vec<(usize, Coord)> = serde_json::from_reader(reader)?;
    Ok(pt_stops)
}

pub fn read_walk_nodes() -> Result<HashMap<usize, Coord>> {
    let file = File::open("../data/walk_nodes.json")?;
    let reader = BufReader::new(file);
    let walk_nodes: HashMap<usize, Coord> = serde_json::from_reader(reader)?;
    Ok(walk_nodes)
}

pub fn read_walk_graph() -> Result<Vec<Vec<(usize, usize, u16, u16, u32)>>> {
    let file = File::open("../data/walk_graph.json")?;
    let reader = BufReader::new(file);
    let walk_graph: Vec<Vec<(usize, usize, u16, u16, u32)>> = serde_json::from_reader(reader)?;
    Ok(walk_graph)
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
    let longitudes: Vec<f32> = network_subnodes
        .iter()
        .map(|x| x.longitude as f32)
        .collect();
    let latitudes: Vec<f32> = network_subnodes.iter().map(|x| x.latitude as f32).collect();
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
        "longitude" => longitudes,
        "latitude" => latitudes,
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
