use std::collections::HashMap;
use std::io::{BufReader, BufWriter};

use anyhow::Result;
use fs_err::File;
use geo::{Coord, LineString};
use geojson::{Feature, FeatureWriter, Geometry};
use indicatif::{ProgressBar, ProgressStyle};
use osm_reader::{Element, NodeID, WayID};
use serde::Deserialize;


fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        panic!("Call with the input path to an osm.pbf and a GeoTIFF file");
    }

    let settings = read_settings(&args[3]).unwrap();
    run(&args[1], &args[2], settings).unwrap();

}

fn run(osm_path: &str, output_path: &str, settings: Settings) -> Result<()> {
    let edges = scrape_osm(osm_path, settings)?;

    println!("Writing output");
    let progress = ProgressBar::new(edges.len() as u64).with_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap());
    let mut out = FeatureWriter::from_writer(BufWriter::new(File::create(output_path)?));
    for (_, linestring) in edges {
        progress.inc(1);
        let mut f = Feature::from(Geometry::from(&linestring));
        f.set_property("gradient", 1);
        out.write_feature(&f)?;
    }
    progress.finish();

    Ok(())
}

fn scrape_osm(osm_path: &str, settings: Settings) -> Result<Vec<(WayID, LineString)>> {
    let mut node_mapping: HashMap<NodeID, Coord> = HashMap::new();
    let mut highways: Vec<(WayID, Vec<NodeID>)> = Vec::new();
    let mut first_way = true;
    println!("Reading {osm_path}");
    let nodes_progress = ProgressBar::new_spinner().with_style(
        ProgressStyle::with_template("[{elapsed_precise}] {human_len} nodes read ({per_sec})")
            .unwrap(),
    );
    let ways_progress = ProgressBar::new_spinner().with_style(
        ProgressStyle::with_template("[{elapsed_precise}] {human_len} ways read ({per_sec})")
            .unwrap(),
    );
    osm_reader::parse(&fs_err::read(osm_path)?, |elem| match elem {
        Element::Node { id, lon, lat, .. } => {
            nodes_progress.inc(1);
            node_mapping.insert(id, Coord { x: lon, y: lat });
        }
        Element::Way { id, node_ids, tags, .. } => {
            if tags.contains_key("highway")
                // select just ways meeting mode criteria
                && settings.tag_pairs.iter().all(|(k, v)| tags.get(k) != Some(v))
            {
                if first_way {
                    nodes_progress.finish();
                    first_way = false;
                }
                ways_progress.inc(1);
                highways.push((id, node_ids));
            }
        }
        Element::Relation { .. } | Element::Bounds { .. } => {}
    })?;
    ways_progress.finish();

    Ok(split_edges(node_mapping, highways))
}

fn split_edges(
    node_mapping: HashMap<NodeID, Coord>,
    ways: Vec<(WayID, Vec<NodeID>)>,
) -> Vec<(WayID, LineString)> {
    println!("Splitting ways into edges");

    // Count how many ways reference each node
    let mut node_counter: HashMap<NodeID, usize> = HashMap::new();
    for (_, node_ids) in &ways {
        for node in node_ids {
            *node_counter.entry(*node).or_insert(0) += 1;
        }
    }

    // Split each way into edges
    let progress = ProgressBar::new(ways.len() as u64).with_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap());
    let mut edges = Vec::new();
    for (way_id, node_ids) in ways {
        progress.inc(1);
        let mut pts = Vec::new();

        let num_nodes = node_ids.len();
        for (idx, node) in node_ids.into_iter().enumerate() {
            pts.push(node_mapping[&node]);
            // Edges start/end at intersections between two ways. The endpoints of the way also
            // count as intersections.
            let is_endpoint =
                idx == 0 || idx == num_nodes - 1 || *node_counter.get(&node).unwrap() > 1;
            if is_endpoint && pts.len() > 1 {
                edges.push((way_id, LineString::new(std::mem::take(&mut pts))));

                // Start the next edge
                pts.push(node_mapping[&node]);
            }
        }
    }
    progress.finish();
    edges
}

pub fn read_settings(mode: &str) -> Result<Settings> {
    let inpath = format!("../graph_from_pbf/settings/{}.json", mode);
    let file = File::open(inpath)?;
    let reader = BufReader::new(file);
    let settings: Settings = serde_json::from_reader(reader)?;
    Ok(settings)
}

#[derive(Deserialize)]
pub struct Settings {
    pub mode: String,
    pub tag_pairs: Vec<(String, String)>,
    pub speed: f32,           // m/s
    pub ascention_speed: f32, // s/m
    pub descent_speed: f32,   // s/m
}
